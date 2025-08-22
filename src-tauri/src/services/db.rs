use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use lancedb::{Connection, Table, Result as LanceResult};
use arrow_array::{RecordBatch, StringArray, ListArray, Float32Array};
use arrow_schema::{Schema, Field, DataType};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct File {
    pub directory: String,
    pub embeddings: Vec<Vec<f32>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Folder {
    pub directory: String,
    pub files: Vec<File>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WatchItem {
    File(File),
    Folder(Folder),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchingArray {
    pub items: Vec<WatchItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchDatabase {
    pub is_watching: bool,
    pub watching_array: WatchingArray,
}

pub struct DatabaseService {
    connection: Arc<Connection>,
    watch_state: Arc<RwLock<bool>>,
}

impl DatabaseService {
    pub async fn new(db_path: &str) -> LanceResult<Self> {
        let connection = Arc::new(lancedb::connect(db_path).await?);
        
        let service = Self {
            connection,
            watch_state: Arc::new(RwLock::new(false)),
        };
        
        service.init_tables().await?;
        Ok(service)
    }

    async fn init_tables(&self) -> LanceResult<()> {
        let schema = Schema::new(vec![
            Field::new("id", DataType::Utf8, false),
            Field::new("item_type", DataType::Utf8, false),
            Field::new("directory", DataType::Utf8, false),
            Field::new("metadata", DataType::Utf8, true),
        ]);

        let empty_batch = RecordBatch::new_empty(Arc::new(schema));
        
        if !self.connection.table_names().await?.contains(&"watch_items".to_string()) {
            self.connection
                .create_table("watch_items", vec![empty_batch])
                .await?;
        }

        Ok(())
    }

    // isWatching CRUD
    pub async fn read_is_watching(&self) -> bool {
        *self.watch_state.read().await
    }

    pub async fn update_is_watching(&self, is_watching: bool) {
        *self.watch_state.write().await = is_watching;
    }

    // File CRUD
    pub async fn create_file(&self, file: File) {
        if let Ok(table) = self.connection.open_table("watch_items").await {
            if let Ok(batch) = self.file_to_record_batch(&file) {
                let _ = table.add(vec![batch]).await;
            }
        }
    }

    pub async fn read_file(&self, directory: &str) -> Option<File> {
        let table = self.connection.open_table("watch_items").await.ok()?;
        
        let results = table
            .query()
            .filter(&format!("directory = '{}' AND item_type = 'file'", directory))
            .execute()
            .await
            .ok()?;

        if results.is_empty() {
            return None;
        }

        self.record_batch_to_file(&results[0]).ok()
    }

    pub async fn delete_file(&self, directory: &str) -> bool {
        if let Ok(table) = self.connection.open_table("watch_items").await {
            if let Ok(deleted) = table
                .delete(&format!("directory = '{}' AND item_type = 'file'", directory))
                .await 
            {
                return deleted > 0;
            }
        }
        false
    }

    // Folder CRUD
    pub async fn create_folder(&self, folder: Folder) {
        if let Ok(table) = self.connection.open_table("watch_items").await {
            if let Ok(batch) = self.folder_to_record_batch(&folder) {
                let _ = table.add(vec![batch]).await;
            }
        }
    }

    pub async fn read_folder(&self, directory: &str) -> Option<Folder> {
        let table = self.connection.open_table("watch_items").await.ok()?;
        
        let results = table
            .query()
            .filter(&format!("directory = '{}' AND item_type = 'folder'", directory))
            .execute()
            .await
            .ok()?;

        if results.is_empty() {
            return None;
        }

        self.record_batch_to_folder(&results[0]).ok()
    }

    pub async fn delete_folder(&self, directory: &str) -> bool {
        if let Ok(table) = self.connection.open_table("watch_items").await {
            if let Ok(deleted) = table
                .delete(&format!("directory = '{}' AND item_type = 'folder'", directory))
                .await 
            {
                return deleted > 0;
            }
        }
        false
    }

    // Folder의 File 관리
    pub async fn create_file_in_folder(&self, folder_directory: &str, file: File) -> bool {
        if let Some(mut folder) = self.read_folder(folder_directory).await {
            folder.files.push(file);
            self.delete_folder(folder_directory).await;
            self.create_folder(folder).await;
            return true;
        }
        false
    }

    pub async fn read_file_in_folder(&self, folder_directory: &str, file_directory: &str) -> Option<File> {
        let folder = self.read_folder(folder_directory).await?;
        folder.files.into_iter().find(|file| file.directory == file_directory)
    }

    pub async fn delete_file_in_folder(&self, folder_directory: &str, file_directory: &str) -> bool {
        if let Some(mut folder) = self.read_folder(folder_directory).await {
            let initial_len = folder.files.len();
            folder.files.retain(|file| file.directory != file_directory);
            if folder.files.len() != initial_len {
                self.delete_folder(folder_directory).await;
                self.create_folder(folder).await;
                return true;
            }
        }
        false
    }

    // 파일 수정 시 delete + create 패턴
    pub async fn recreate_file(&self, old_directory: &str, new_file: File) -> bool {
        if self.delete_file(old_directory).await {
            self.create_file(new_file).await;
            true
        } else {
            false
        }
    }

    pub async fn recreate_file_in_folder(&self, folder_directory: &str, old_file_directory: &str, new_file: File) -> bool {
        if self.delete_file_in_folder(folder_directory, old_file_directory).await {
            self.create_file_in_folder(folder_directory, new_file).await
        } else {
            false
        }
    }

    // 전체 데이터 조회
    pub async fn read_all_items(&self) -> Vec<WatchItem> {
        let Ok(table) = self.connection.open_table("watch_items").await else { 
            return Vec::new(); 
        };
        
        let Ok(results) = table.query().execute().await else { 
            return Vec::new(); 
        };
        
        let mut items = Vec::new();
        for batch in results {
            let Some(item_type_array) = batch
                .column_by_name("item_type")
                .and_then(|col| col.as_any().downcast_ref::<StringArray>()) else { 
                continue; 
            };
                
            for (i, item_type) in item_type_array.iter().enumerate() {
                match item_type {
                    Some("file") => {
                        if let Ok(file) = self.record_batch_row_to_file(&batch, i) {
                            items.push(WatchItem::File(file));
                        }
                    }
                    Some("folder") => {
                        if let Ok(folder) = self.record_batch_row_to_folder(&batch, i) {
                            items.push(WatchItem::Folder(folder));
                        }
                    }
                    _ => {}
                }
            }
        }
        
        items
    }

    pub async fn read_watch_database(&self) -> WatchDatabase {
        WatchDatabase {
            is_watching: self.read_is_watching().await,
            watching_array: WatchingArray {
                items: self.read_all_items().await,
            },
        }
    }

    // Helper methods
    fn file_to_record_batch(&self, file: &File) -> LanceResult<RecordBatch> {
        let id = format!("file_{}", file.directory);
        
        let batch = RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new("id", DataType::Utf8, false),
                Field::new("item_type", DataType::Utf8, false),
                Field::new("directory", DataType::Utf8, false),
                Field::new("metadata", DataType::Utf8, true),
            ])),
            vec![
                Arc::new(StringArray::from(vec![id])),
                Arc::new(StringArray::from(vec!["file"])),
                Arc::new(StringArray::from(vec![file.directory.clone()])),
                Arc::new(StringArray::from(vec![serde_json::to_string(file).unwrap_or_default()])),
            ],
        )?;
        
        Ok(batch)
    }

    fn folder_to_record_batch(&self, folder: &Folder) -> LanceResult<RecordBatch> {
        let id = format!("folder_{}", folder.directory);
        
        let batch = RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new("id", DataType::Utf8, false),
                Field::new("item_type", DataType::Utf8, false),
                Field::new("directory", DataType::Utf8, false),
                Field::new("metadata", DataType::Utf8, true),
            ])),
            vec![
                Arc::new(StringArray::from(vec![id])),
                Arc::new(StringArray::from(vec!["folder"])),
                Arc::new(StringArray::from(vec![folder.directory.clone()])),
                Arc::new(StringArray::from(vec![serde_json::to_string(folder).unwrap_or_default()])),
            ],
        )?;
        
        Ok(batch)
    }

    fn record_batch_to_file(&self, batch: &RecordBatch) -> LanceResult<File> {
        let metadata_array = batch
            .column_by_name("metadata")
            .and_then(|col| col.as_any().downcast_ref::<StringArray>())
            .ok_or_else(|| lancedb::Error::InvalidInput("No metadata column".into()))?;
            
        let metadata_str = metadata_array.value(0);
        let file: File = serde_json::from_str(metadata_str)
            .map_err(|e| lancedb::Error::InvalidInput(format!("JSON parse error: {}", e)))?;
            
        Ok(file)
    }

    fn record_batch_to_folder(&self, batch: &RecordBatch) -> LanceResult<Folder> {
        let metadata_array = batch
            .column_by_name("metadata")
            .and_then(|col| col.as_any().downcast_ref::<StringArray>())
            .ok_or_else(|| lancedb::Error::InvalidInput("No metadata column".into()))?;
            
        let metadata_str = metadata_array.value(0);
        let folder: Folder = serde_json::from_str(metadata_str)
            .map_err(|e| lancedb::Error::InvalidInput(format!("JSON parse error: {}", e)))?;
            
        Ok(folder)
    }

    fn record_batch_row_to_file(&self, batch: &RecordBatch, row: usize) -> LanceResult<File> {
        let metadata_array = batch
            .column_by_name("metadata")
            .and_then(|col| col.as_any().downcast_ref::<StringArray>())
            .ok_or_else(|| lancedb::Error::InvalidInput("No metadata column".into()))?;
            
        let metadata_str = metadata_array.value(row);
        let file: File = serde_json::from_str(metadata_str)
            .map_err(|e| lancedb::Error::InvalidInput(format!("JSON parse error: {}", e)))?;
            
        Ok(file)
    }

    fn record_batch_row_to_folder(&self, batch: &RecordBatch, row: usize) -> LanceResult<Folder> {
        let metadata_array = batch
            .column_by_name("metadata")
            .and_then(|col| col.as_any().downcast_ref::<StringArray>())
            .ok_or_else(|| lancedb::Error::InvalidInput("No metadata column".into()))?;
            
        let metadata_str = metadata_array.value(row);
        let folder: Folder = serde_json::from_str(metadata_str)
            .map_err(|e| lancedb::Error::InvalidInput(format!("JSON parse error: {}", e)))?;
            
        Ok(folder)
    }
}