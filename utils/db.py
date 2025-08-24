import chromadb

class FilefairyDB:
    def __init__(self, db_path="./chroma_db"):
        self.client = chromadb.PersistentClient(path=db_path)
        self.collection = self.client.get_or_create_collection(
            name="file_fairy",
            metadata={"description": "파일 시스템 기반 텍스트 임베딩 저장소"}
        )
    
    def add_file_content(self, folder_path, file_name, texts, embeddings):
        if len(texts) != len(embeddings):
            raise ValueError("텍스트와 임베딩의 개수가 일치하지 않습니다")
        
        # 각 텍스트 조각에 대한 고유 ID 생성
        ids = [f"{folder_path}::{file_name}::{i}" for i in range(len(texts))]
        
        # 메타데이터 생성
        metadatas = [
            {
                "folder_path": folder_path,
                "file_name": file_name,
                "chunk_index": i
            }
            for i in range(len(texts))
        ]
        
        self.collection.add(
            ids=ids,
            documents=texts,
            embeddings=embeddings,
            metadatas=metadatas
        )
    
    def search_similar(self, query_embedding, top_k=5):
        results = self.collection.query(
            query_embeddings=[query_embedding],
            n_results=top_k
        )
        return results
    
    def get_all_folders(self):
        results = self.collection.get()
        folders = set()
        for metadata in results['metadatas']:
            folders.add(metadata['folder_path'])
        return list(folders)
    
    def get_files_in_folder(self, folder_path):
        results = self.collection.get(where={"folder_path": folder_path})
        files = set()
        for metadata in results['metadatas']:
            files.add(metadata['file_name'])
        return list(files)
    
    def delete_folder(self, folder_path):
        self.collection.delete(where={"folder_path": folder_path})
    
    def delete_file(self, folder_path, file_name):
        """특정 파일의 모든 청크 삭제"""
        self.collection.delete(where={
            "folder_path": folder_path,
            "file_name": file_name
        })
    
    def get_stats(self):
        all_data = self.collection.get()
        total_chunks = len(all_data['ids'])
        folders = self.get_all_folders()
        total_folders = len(folders)
        total_files = sum(len(self.get_files_in_folder(folder)) for folder in folders)
        
        return {
            "총_텍스트_조각": total_chunks,
            "총_폴더": total_folders,
            "총_파일": total_files
        }
