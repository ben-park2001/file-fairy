from utils.clustering import get_top_cluster_representatives
from utils.db import FilefairyDB
from utils.file_parser import extract_text
from utils.ollama import Ollama
from utils.file_watcher import FileWatcher
import os


class FileFairyCli:
    def __init__(self):
        self.db = FilefairyDB()
        self.ollama = Ollama()
        self.watch_directory = self.db.get_all_folders()
        self.file_watcher = FileWatcher(self.db, self.ollama, extract_text, self.watch_directory)
    

    def searcher(self, query_text):
        query_embedding = self.ollama.get_embedding([query_text])
        search_results = self.db.search_similar(query_embedding)
        
        results = []
        for i in range(len(search_results['ids'])):
            result_item = {
                'file_name': search_results['metadatas'][i]['file_name'],
                'folder_path': search_results['metadatas'][i]['folder_path'],
                'document': search_results['documents'][i]
            }
            results.append(result_item)
        
        return results


    def get_watch_folders(self):
        """현재 감시 중인 폴더 목록 반환"""
        return self.watch_directory
    
    def add_watch_folder(self, folder_path):
        """감시 폴더 추가 및 기존 파일 스캔"""
        if self.file_watcher.add_watch_directory(folder_path):
            if folder_path not in self.watch_directory:
                self.watch_directory.append(folder_path)
                self._scan_folder(folder_path)   # 기존 파일들 스캔
                return True
        return False
    
    def remove_watch_folder(self, folder_path):
        """감시 폴더 제거 및 DB에서 파일 삭제"""
        if folder_path in self.watch_directory:
            self.file_watcher.remove_watch_directory(folder_path)
            self.watch_directory.remove(folder_path)
            self.db.delete_folder(folder_path)  # DB에서 폴더와 파일들 제거
            return True
        return False
    
    def _scan_folder(self, folder_path):
        """폴더의 모든 파일을 스캔하여 DB에 추가"""
        for root, dirs, files in os.walk(folder_path):
            for file in files:
                file_path = os.path.join(root, file)
                try:
                    text_content = extract_text(file_path)
                    if text_content and text_content.get('content'):
                        content = text_content['content']
                        embedding = self.ollama.get_embedding([content])
                        self.db.add_file_content(
                            folder_path=os.path.dirname(file_path),
                            file_name=file,
                            texts=[content],
                            embeddings=embedding
                        )
                except Exception as e:
                    print(f"파일 스캔 오류 ({file_path}): {e}")
                    continue


    def filename_changer(self, file_folder_directories):
        # 1단계: 파일 목록 수집
        all_files = []
        
        for path in file_folder_directories:
            if os.path.isfile(path):
                all_files.append(path)
            elif os.path.isdir(path):
                for root, dirs, files in os.walk(path):
                    for file in files:
                        file_path = os.path.join(root, file)
                        all_files.append(file_path)
        
        # 2단계: 각 파일의 텍스트 추출
        file_texts = []
        for file_path in all_files:
            try:
                text_content = extract_text(file_path)
                if text_content and text_content.get('content'):
                    file_texts.append((file_path, text_content['content']))
            except Exception as e:
                print(f"텍스트 추출 오류 ({file_path}): {e}")
                continue
        
        # 3단계: 텍스트를 정해진 길이로 자름
        chunk_size = 300
        results = []
        
        for file_path, text_content in file_texts:
            chunks = [text_content[i:i + chunk_size] for i in range(0, len(text_content), chunk_size)]
            
            # 4단계: 청크들을 임베딩으로 변환
            embeddings = self.ollama.get_embedding(chunks)
            
            # 5단계: 대표 텍스트 인덱스 찾기
            n_clusters = min(len(chunks), 5)  # 청크 수와 5 중 작은 값
            top_n = min(n_clusters, 3)  # 상위 3개 클러스터
            representative_indices = get_top_cluster_representatives(embeddings, n_clusters, top_n)
            
            # 6단계: 대표 텍스트들로 key_chunks 생성
            key_chunks = [chunks[idx] for idx in representative_indices]
            
            # 7단계: 새로운 파일명 생성
            original_filename = os.path.splitext(os.path.basename(file_path))[0]
            file_extension = os.path.splitext(file_path)[1][1:]  # 확장자에서 . 제거
            new_filename = self.ollama.get_filename(key_chunks, original_filename, file_extension)
            
            # 8단계: 결과 구성
            results.append({
                'original_filename': os.path.basename(file_path),
                'key_chunks': key_chunks,
                'new_filename': new_filename,
                'file_path': file_path
            })
        
        return results

