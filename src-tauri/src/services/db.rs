//! Vector database service using LanceDB for document embeddings and semantic search
//!
//! This module provides CRUD operations for document embeddings using LanceDB,
//! enabling efficient semantic search capabilities for the file organization system.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use arrow::array::types::Float32Type;
use arrow::array::{
    FixedSizeListArray, Float32Array, RecordBatch, RecordBatchIterator, StringArray, UInt64Array,
};
use arrow::datatypes::{DataType, Field, Schema};
use futures::stream::TryStreamExt;
use lancedb::connection::Connection;
use lancedb::index::Index;
use lancedb::query::{ExecutableQuery, QueryBase};
use lancedb::{connect, Result as LanceResult, Table as LanceDbTable};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// The dimension size for document embeddings
/// Using 384 dimensions which is common for sentence transformers
const EMBEDDING_DIMENSION: usize = 384;

/// Represents a document embedding stored in the vector database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentEmbedding {
    /// Unique identifier for the document
    pub id: String,
    /// Full file path of the document
    pub file_path: String,
    /// Name of the file
    pub file_name: String,
    /// Directory containing the file
    pub directory: String,
    /// File extension
    pub file_extension: String,
    /// Unix timestamp when the embedding was created
    pub created_at: u64,
    /// Extracted text content from the document
    pub content: String,
    /// Vector embedding of the document content
    pub embedding: Vec<f32>,
}

impl DocumentEmbedding {
    /// Create a new document embedding
    pub fn new(
        file_path: String,
        file_name: String,
        directory: String,
        file_extension: String,
        content: String,
        embedding: Vec<f32>,
    ) -> Self {
        let id = Uuid::new_v4().to_string();
        let created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            id,
            file_path,
            file_name,
            directory,
            file_extension,
            created_at,
            content,
            embedding,
        }
    }

    /// Create a placeholder embedding with zeros (for testing/development)
    pub fn with_placeholder_embedding(
        file_path: String,
        file_name: String,
        directory: String,
        file_extension: String,
        content: String,
    ) -> Self {
        let embedding = vec![0.0; EMBEDDING_DIMENSION];
        Self::new(
            file_path,
            file_name,
            directory,
            file_extension,
            content,
            embedding,
        )
    }
}

/// LanceDB-based vector database service for document embeddings
#[derive(Clone)]
pub struct VectorDbService {
    db_path: PathBuf,
    table_name: String,
    connection: Arc<Mutex<Option<Connection>>>,
}

impl VectorDbService {
    /// Create a new vector database service with default path
    pub fn default() -> Self {
        let default_path = PathBuf::from("data/file_fairy_embeddings");
        Self::new(default_path)
    }

    /// Create a new vector database service with custom path
    pub fn new(db_path: PathBuf) -> Self {
        Self {
            db_path,
            table_name: "document_embeddings".to_string(),
            connection: Arc::new(Mutex::new(None)),
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
            let mut connection = self
                .connection
                .lock()
                .map_err(|_| "Failed to lock connection")?;
            *connection = Some(db);
        }
        Ok(())
    }

    /// Create the document embeddings table
    async fn create_table(&self, db: &Connection) -> LanceResult<LanceDbTable> {
        let schema = Arc::new(Schema::new(vec![
            Field::new("id", DataType::Utf8, false),
            Field::new("file_path", DataType::Utf8, false),
            Field::new("file_name", DataType::Utf8, false),
            Field::new("directory", DataType::Utf8, false),
            Field::new("file_extension", DataType::Utf8, false),
            Field::new("created_at", DataType::UInt64, false),
            Field::new("content", DataType::Utf8, false),
            Field::new(
                "embedding",
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

        // Create vector index for similarity search
        table
            .create_index(&["embedding"], Index::Auto)
            .execute()
            .await?;

        Ok(table)
    }

    /// Get the table handle
    async fn get_table(&self) -> Result<LanceDbTable, Box<dyn std::error::Error + Send + Sync>> {
        // Clone the connection from the mutex to avoid holding the lock across await points
        let connection_opt = {
            let connection = self
                .connection
                .lock()
                .map_err(|_| "Failed to lock connection")?;
            connection.clone()
        };

        let db = if let Some(conn) = connection_opt {
            conn
        } else {
            return Err("Database not initialized. Call initialize() first.".into());
        };

        let table = db.open_table(&self.table_name).execute().await?;
        Ok(table)
    }

    /// Insert or update a document embedding
    pub async fn upsert_embedding(
        &self,
        embedding: DocumentEmbedding,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let table = self.get_table().await?;

        // First, delete any existing embedding for this file path
        let delete_query = format!("file_path = '{}'", embedding.file_path);
        let _ = table.delete(&delete_query).await; // Ignore errors if no rows to delete

        // Create record batch from the embedding
        let batch = self.create_record_batch(vec![embedding])?;
        let batch_iterator =
            RecordBatchIterator::new(vec![batch].into_iter().map(Ok), self.get_schema());

        // Insert the new embedding
        table.add(batch_iterator).execute().await?;

        Ok(())
    }

    /// Delete an embedding by file path
    pub async fn delete_embedding(
        &self,
        file_path: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let table = self.get_table().await?;

        let delete_query = format!("file_path = '{}'", file_path);
        table.delete(&delete_query).await?;

        Ok(())
    }

    /// Get an embedding by file path
    pub async fn get_embedding(
        &self,
        file_path: &str,
    ) -> Result<Option<DocumentEmbedding>, Box<dyn std::error::Error + Send + Sync>> {
        let table = self.get_table().await?;

        // For now, get all results and filter in memory
        // In a real implementation, you'd use proper SQL filtering
        let mut stream = table
            .query()
            .limit(1000) // Reasonable limit for filtering
            .execute()
            .await?;

        let mut embeddings = Vec::new();
        while let Some(batch) = stream.try_next().await? {
            let parsed = self.parse_record_batches(vec![batch])?;
            embeddings.extend(parsed);
        }

        // Filter by file path
        let result = embeddings.into_iter().find(|e| e.file_path == file_path);

        Ok(result)
    }

    /// Search for similar documents using vector similarity
    pub async fn search_similar(
        &self,
        query_embedding: Vec<f32>,
        limit: usize,
    ) -> Result<Vec<DocumentEmbedding>, Box<dyn std::error::Error + Send + Sync>> {
        let table = self.get_table().await?;

        if query_embedding.len() != EMBEDDING_DIMENSION {
            return Err("Query embedding dimension mismatch".into());
        }

        let mut stream = table
            .query()
            .nearest_to(query_embedding.as_slice())?
            .limit(limit)
            .execute()
            .await?;

        let mut embeddings = Vec::new();
        while let Some(batch) = stream.try_next().await? {
            let parsed = self.parse_record_batches(vec![batch])?;
            embeddings.extend(parsed);
        }

        Ok(embeddings)
    }

    /// Get all embeddings for files in a specific directory
    pub async fn get_embeddings_by_directory(
        &self,
        directory_path: &str,
    ) -> Result<Vec<DocumentEmbedding>, Box<dyn std::error::Error + Send + Sync>> {
        let table = self.get_table().await?;

        // Get all results and filter in memory
        let mut stream = table.query().execute().await?;

        let mut embeddings = Vec::new();
        while let Some(batch) = stream.try_next().await? {
            let parsed = self.parse_record_batches(vec![batch])?;
            embeddings.extend(parsed);
        }

        // Filter by directory
        let filtered: Vec<DocumentEmbedding> = embeddings
            .into_iter()
            .filter(|e| e.directory == directory_path)
            .collect();

        Ok(filtered)
    }

    /// Get all embeddings in the database
    pub async fn get_all_embeddings(
        &self,
    ) -> Result<Vec<DocumentEmbedding>, Box<dyn std::error::Error + Send + Sync>> {
        let table = self.get_table().await?;

        let mut stream = table.query().execute().await?;

        let mut embeddings = Vec::new();
        while let Some(batch) = stream.try_next().await? {
            let parsed = self.parse_record_batches(vec![batch])?;
            embeddings.extend(parsed);
        }

        Ok(embeddings)
    }

    /// Clean up embeddings for files that no longer exist
    pub async fn cleanup_stale_embeddings(
        &self,
    ) -> Result<u64, Box<dyn std::error::Error + Send + Sync>> {
        let embeddings = self.get_all_embeddings().await?;
        let mut deleted_count = 0u64;

        for embedding in embeddings {
            let path = PathBuf::from(&embedding.file_path);
            if !path.exists() {
                self.delete_embedding(&embedding.file_path).await?;
                deleted_count += 1;
            }
        }

        Ok(deleted_count)
    }

    /// Get database statistics
    pub async fn get_stats(
        &self,
    ) -> Result<HashMap<String, u64>, Box<dyn std::error::Error + Send + Sync>> {
        let table = self.get_table().await?;

        // Get total count
        let mut stream = table.query().execute().await?;

        let mut total_count = 0u64;
        let mut all_embeddings = Vec::new();

        while let Some(batch) = stream.try_next().await? {
            total_count += batch.num_rows() as u64;
            let parsed = self.parse_record_batches(vec![batch])?;
            all_embeddings.extend(parsed);
        }

        // Get counts by file extension
        let mut extension_counts: HashMap<String, u64> = HashMap::new();

        for embedding in &all_embeddings {
            *extension_counts
                .entry(embedding.file_extension.clone())
                .or_insert(0) += 1;
        }

        let mut stats = HashMap::new();
        stats.insert("total_documents".to_string(), total_count);

        for (ext, count) in extension_counts {
            stats.insert(format!("documents_{}", ext), count);
        }

        Ok(stats)
    }

    /// Create a record batch from document embeddings
    fn create_record_batch(
        &self,
        embeddings: Vec<DocumentEmbedding>,
    ) -> Result<RecordBatch, Box<dyn std::error::Error + Send + Sync>> {
        let schema = self.get_schema();

        let ids: Vec<String> = embeddings.iter().map(|e| e.id.clone()).collect();
        let file_paths: Vec<String> = embeddings.iter().map(|e| e.file_path.clone()).collect();
        let file_names: Vec<String> = embeddings.iter().map(|e| e.file_name.clone()).collect();
        let directories: Vec<String> = embeddings.iter().map(|e| e.directory.clone()).collect();
        let file_extensions: Vec<String> = embeddings
            .iter()
            .map(|e| e.file_extension.clone())
            .collect();
        let created_at: Vec<u64> = embeddings.iter().map(|e| e.created_at).collect();
        let contents: Vec<String> = embeddings.iter().map(|e| e.content.clone()).collect();

        let embedding_vectors: Vec<Option<Vec<Option<f32>>>> = embeddings
            .iter()
            .map(|e| Some(e.embedding.iter().map(|&v| Some(v)).collect()))
            .collect();

        let batch = RecordBatch::try_new(
            schema,
            vec![
                Arc::new(StringArray::from(ids)),
                Arc::new(StringArray::from(file_paths)),
                Arc::new(StringArray::from(file_names)),
                Arc::new(StringArray::from(directories)),
                Arc::new(StringArray::from(file_extensions)),
                Arc::new(UInt64Array::from(created_at)),
                Arc::new(StringArray::from(contents)),
                Arc::new(
                    FixedSizeListArray::from_iter_primitive::<Float32Type, _, _>(
                        embedding_vectors,
                        EMBEDDING_DIMENSION as i32,
                    ),
                ),
            ],
        )?;

        Ok(batch)
    }

    /// Parse record batches into document embeddings
    fn parse_record_batches(
        &self,
        batches: Vec<RecordBatch>,
    ) -> Result<Vec<DocumentEmbedding>, Box<dyn std::error::Error + Send + Sync>> {
        let mut embeddings = Vec::new();

        for batch in batches {
            let num_rows = batch.num_rows();

            let ids = batch
                .column(0)
                .as_any()
                .downcast_ref::<StringArray>()
                .unwrap();
            let file_paths = batch
                .column(1)
                .as_any()
                .downcast_ref::<StringArray>()
                .unwrap();
            let file_names = batch
                .column(2)
                .as_any()
                .downcast_ref::<StringArray>()
                .unwrap();
            let directories = batch
                .column(3)
                .as_any()
                .downcast_ref::<StringArray>()
                .unwrap();
            let file_extensions = batch
                .column(4)
                .as_any()
                .downcast_ref::<StringArray>()
                .unwrap();
            let created_at = batch
                .column(5)
                .as_any()
                .downcast_ref::<UInt64Array>()
                .unwrap();
            let contents = batch
                .column(6)
                .as_any()
                .downcast_ref::<StringArray>()
                .unwrap();
            let embedding_array = batch
                .column(7)
                .as_any()
                .downcast_ref::<FixedSizeListArray>()
                .unwrap();

            for row in 0..num_rows {
                let embedding_values = embedding_array.value(row);
                let float_array = embedding_values
                    .as_any()
                    .downcast_ref::<Float32Array>()
                    .unwrap();
                let embedding: Vec<f32> = (0..float_array.len())
                    .map(|i| float_array.value(i))
                    .collect();

                let doc_embedding = DocumentEmbedding {
                    id: ids.value(row).to_string(),
                    file_path: file_paths.value(row).to_string(),
                    file_name: file_names.value(row).to_string(),
                    directory: directories.value(row).to_string(),
                    file_extension: file_extensions.value(row).to_string(),
                    created_at: created_at.value(row),
                    content: contents.value(row).to_string(),
                    embedding,
                };

                embeddings.push(doc_embedding);
            }
        }

        Ok(embeddings)
    }

    /// Get the Arrow schema for the document embeddings table
    fn get_schema(&self) -> Arc<Schema> {
        Arc::new(Schema::new(vec![
            Field::new("id", DataType::Utf8, false),
            Field::new("file_path", DataType::Utf8, false),
            Field::new("file_name", DataType::Utf8, false),
            Field::new("directory", DataType::Utf8, false),
            Field::new("file_extension", DataType::Utf8, false),
            Field::new("created_at", DataType::UInt64, false),
            Field::new("content", DataType::Utf8, false),
            Field::new(
                "embedding",
                DataType::FixedSizeList(
                    Arc::new(Field::new("item", DataType::Float32, true)),
                    EMBEDDING_DIMENSION as i32,
                ),
                false,
            ),
        ]))
    }
}

impl Default for VectorDbService {
    fn default() -> Self {
        Self::default()
    }
}
