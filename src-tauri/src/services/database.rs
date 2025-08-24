use super::OllamaService;
use crate::error::{AppError, AppResult};
use crate::types::FileChunkSchema;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use arrow::array::types::Float32Type;
use arrow::array::{
    FixedSizeListArray, Float32Array, RecordBatch, RecordBatchIterator, StringArray, UInt64Array,
};
use arrow::datatypes::{DataType, Field, Schema};
use futures::stream::TryStreamExt;
use lancedb::connection::Connection;
use lancedb::index::scalar::{FtsIndexBuilder, FullTextSearchQuery};
use lancedb::index::Index;
use lancedb::query::{QueryBase, QueryExecutionOptions};
use lancedb::{connect, Result as LanceResult, Table as LanceDbTable};

const DB_PATH: &str = "data/file_fairy.db";
const TABLE_NAME: &str = "file_chunks";
const EMBEDDING_DIMENSION: usize = 1024;

pub struct DatabaseService {
    db_path: PathBuf,
    table_name: String,
    conn: Arc<Mutex<Option<Connection>>>,
    ollama: OllamaService,
}

impl DatabaseService {
    pub fn default() -> Self {
        Self {
            db_path: PathBuf::from(DB_PATH),
            table_name: TABLE_NAME.to_string(),
            conn: Arc::new(Mutex::new(None)),
            ollama: OllamaService::new(),
        }
    }

    /// Initialize the database connection and create table if it doesn't exist
    pub async fn initialize(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Create directory if it doesn't exist
        if let Some(parent) = self.db_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        // Connect to LanceDB
        let uri = self.db_path.to_string_lossy().to_string();
        let db = connect(&uri).execute().await?;

        // Check if table exists, create if not
        let table_names = db.table_names().execute().await?;

        if !table_names.contains(&self.table_name) {
            self.create_table(&db).await?;
        }

        // Store the connection
        {
            let mut connection = self.conn.lock().map_err(|_| "Failed to lock connection")?;
            *connection = Some(db);
        }

        Ok(())
    }

    /// Create the document embeddings table
    async fn create_table(&self, db: &Connection) -> LanceResult<LanceDbTable> {
        let schema = Arc::new(Schema::new(vec![
            Field::new("chunk_id", DataType::UInt64, false),
            Field::new("created_at", DataType::UInt64, false),
            Field::new("file_path", DataType::Utf8, false),
            Field::new("file_name", DataType::Utf8, false),
            Field::new("text", DataType::Utf8, false),
            Field::new(
                "vector",
                DataType::FixedSizeList(
                    Arc::new(Field::new("item", DataType::Float32, true)),
                    EMBEDDING_DIMENSION as i32,
                ),
                false,
            ),
        ]));

        let table = db
            .create_empty_table(&self.table_name, schema)
            .execute()
            .await?;

        // Note: FTS index will be created after first data insertion
        // Vector index will be created after first data insertion

        Ok(table)
    }

    /// Get the table handle
    async fn get_table(&self) -> AppResult<LanceDbTable> {
        // Clone the connection from the mutex to avoid holding the lock across await points
        let connection_opt = {
            let connection = self.conn.lock().map_err(|_| "Failed to lock connection")?;
            connection.clone()
        };

        let db = if let Some(conn) = connection_opt {
            conn
        } else {
            // Try to initialize if not already done
            self.initialize()
                .await
                .map_err(|e| AppError::Other(format!("Failed to initialize database: {}", e)))?;

            // Try to get the connection again after initialization
            let connection = self.conn.lock().map_err(|_| "Failed to lock connection")?;
            if let Some(conn) = connection.clone() {
                conn
            } else {
                return Err(AppError::Other(
                    "Database connection is still not initialized after initialization attempt"
                        .into(),
                ));
            }
        };

        let table = db.open_table(&self.table_name).execute().await?;

        Ok(table)
    }

    pub async fn remove_file(&self, file_path: &str) -> AppResult<()> {
        let table = self.get_table().await?;

        // Delete any existing embedding for this file path
        let delete_query = format!("file_path = '{}'", file_path);
        let _ = table.delete(&delete_query).await; // Ignore errors if no rows to delete

        Ok(())
    }

    pub async fn upsert_file(&self, file_path: &str, content: &str) -> AppResult<()> {
        let table = self.get_table().await?;

        // First, delete any existing embedding for this file path
        self.remove_file(file_path).await?;

        // Split the content into chunks
        let text_chunks = self.ollama.chunk_text(&content).await;

        // Generate embeddings for each chunk
        let embeddings = self
            .ollama
            .generate_embeddings(&text_chunks)
            .await
            .map_err(|e| AppError::Ollama(e))?;

        // Insert the chunks and their embeddings into the database
        let file_chunks = text_chunks
            .into_iter()
            .zip(embeddings)
            .enumerate()
            .map(|(i, (chunk, embedding))| {
                FileChunkSchema::new(i as u64, file_path.to_string(), chunk, embedding)
            })
            .collect::<Vec<_>>();

        // Create record batch from the embedding
        let batch = self.create_record_batch(file_chunks)?;
        let batch_iterator =
            RecordBatchIterator::new(vec![batch].into_iter().map(Ok), self.get_schema());

        // Insert the new embedding
        table.add(batch_iterator).execute().await?;

        // Ensure indices exist after adding data
        self.ensure_indices_exist(&table).await?;

        Ok(())
    }

    /// Ensure that necessary indices exist on the table
    async fn ensure_indices_exist(&self, table: &LanceDbTable) -> AppResult<()> {
        // Try to create FTS index if it doesn't exist
        if let Err(e) = table
            .create_index(&["text"], Index::FTS(FtsIndexBuilder::default()))
            .execute()
            .await
        {
            // Ignore error if index already exists
            let error_msg = e.to_string().to_lowercase();
            if !error_msg.contains("already exists") && !error_msg.contains("duplicate") {
                eprintln!("Warning: Failed to create FTS index: {}", e);
            }
        }

        // Try to create vector index if it doesn't exist
        if let Err(e) = table.create_index(&["vector"], Index::Auto).execute().await {
            // Ignore error if index already exists
            let error_msg = e.to_string().to_lowercase();
            if !error_msg.contains("already exists") && !error_msg.contains("duplicate") {
                eprintln!("Warning: Failed to create vector index: {}", e);
            }
        }

        Ok(())
    }

    pub async fn search_files(&self, query: &str, limit: usize) -> AppResult<Vec<FileChunkSchema>> {
        let table = self.get_table().await?;

        let query_embedding = self
            .ollama
            .generate_embedding(query)
            .await
            .map_err(|e| AppError::Ollama(e))?;

        let mut results = table
            .query()
            .full_text_search(FullTextSearchQuery::new(query.to_owned()))
            .nearest_to(query_embedding)?
            .limit(limit)
            .execute_hybrid(QueryExecutionOptions::default())
            .await?;

        let mut file_chunks = Vec::new();
        while let Some(batch) = results.try_next().await? {
            let chunks = self.parse_record_batches(batch)?;
            file_chunks.extend(chunks);
        }

        Ok(file_chunks)
    }

    /// Create a record batch from document embeddings
    fn create_record_batch(&self, file_chunks: Vec<FileChunkSchema>) -> AppResult<RecordBatch> {
        let schema = self.get_schema();

        let chunk_ids: Vec<u64> = file_chunks.iter().map(|f| f.chunk_id.clone()).collect();
        let file_paths: Vec<String> = file_chunks.iter().map(|f| f.file_path.clone()).collect();
        let file_names: Vec<String> = file_chunks.iter().map(|f| f.file_name.clone()).collect();
        let created_at: Vec<u64> = file_chunks.iter().map(|f| f.created_at).collect();
        let text: Vec<String> = file_chunks.iter().map(|f| f.text.clone()).collect();

        let vectors: Vec<Option<Vec<Option<f32>>>> = file_chunks
            .iter()
            .map(|f| Some(f.vector.iter().map(|&v| Some(v)).collect()))
            .collect();

        let batch = RecordBatch::try_new(
            schema,
            vec![
                Arc::new(UInt64Array::from(chunk_ids)),
                Arc::new(UInt64Array::from(created_at)),
                Arc::new(StringArray::from(file_paths)),
                Arc::new(StringArray::from(file_names)),
                Arc::new(StringArray::from(text)),
                Arc::new(
                    FixedSizeListArray::from_iter_primitive::<Float32Type, _, _>(
                        vectors,
                        EMBEDDING_DIMENSION as i32,
                    ),
                ),
            ],
        )?;

        Ok(batch)
    }

    /// Parse record batches into document embeddings
    fn parse_record_batches(&self, batch: RecordBatch) -> AppResult<Vec<FileChunkSchema>> {
        let mut file_chunks = Vec::new();

        let num_rows = batch.num_rows();

        let chunk_ids = batch
            .column(0)
            .as_any()
            .downcast_ref::<UInt64Array>()
            .unwrap();
        let created_at = batch
            .column(1)
            .as_any()
            .downcast_ref::<UInt64Array>()
            .unwrap();
        let file_paths = batch
            .column(2)
            .as_any()
            .downcast_ref::<StringArray>()
            .unwrap();
        let file_names = batch
            .column(3)
            .as_any()
            .downcast_ref::<StringArray>()
            .unwrap();
        let texts = batch
            .column(4)
            .as_any()
            .downcast_ref::<StringArray>()
            .unwrap();
        let vectors = batch
            .column(5)
            .as_any()
            .downcast_ref::<FixedSizeListArray>()
            .unwrap();

        for row in 0..num_rows {
            let chunk_id = chunk_ids.value(row);
            let created_at = created_at.value(row);
            let file_path = file_paths.value(row).to_string();
            let file_name = file_names.value(row).to_string();
            let text = texts.value(row).to_string();

            let vector_array = vectors.value(row);
            let float_array = vector_array
                .as_any()
                .downcast_ref::<Float32Array>()
                .unwrap();
            let vector: Vec<f32> = (0..float_array.len())
                .map(|i| float_array.value(i))
                .collect();

            file_chunks.push(FileChunkSchema {
                chunk_id,
                created_at,
                file_path,
                file_name,
                text,
                vector,
            });
        }

        Ok(file_chunks)
    }

    /// Get the Arrow schema for the document embeddings table
    fn get_schema(&self) -> Arc<Schema> {
        Arc::new(Schema::new(vec![
            Field::new("chunk_id", DataType::UInt64, false),
            Field::new("created_at", DataType::UInt64, false),
            Field::new("file_path", DataType::Utf8, false),
            Field::new("file_name", DataType::Utf8, false),
            Field::new("text", DataType::Utf8, false),
            Field::new(
                "vector",
                DataType::FixedSizeList(
                    Arc::new(Field::new("item", DataType::Float32, true)),
                    EMBEDDING_DIMENSION as i32,
                ),
                false,
            ),
        ]))
    }
}
