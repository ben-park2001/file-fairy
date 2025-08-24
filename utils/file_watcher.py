from watchdog.observers import Observer
from watchdog.events import FileSystemEventHandler
import os
from .file_parser import extract_text


class FileEventHandler(FileSystemEventHandler):
    def __init__(self, db, ollama):
        self.db = db
        self.ollama = ollama
        self.extract_text = extract_text
    
    def on_created(self, event):
        if not event.is_directory:
            self._process_file(event.src_path, "created")
    
    def on_modified(self, event):
        if not event.is_directory:
            self._process_file(event.src_path, "modified")
    
    def on_deleted(self, event):
        if not event.is_directory:
            print(f"파일 삭제됨: {event.src_path}")
            self._remove_file_from_db(event.src_path)
    
    def _process_file(self, file_path, action):
        try:
            # 텍스트 추출
            text_content = self.extract_text(file_path)
            if text_content and text_content.get('content'):
                content = text_content['content']
                # 임베딩 생성
                embedding = self.ollama.get_embedding([content])
                
                # DB에 저장 (기존 내용은 자동으로 덮어쓰기됨)
                folder_path = os.path.dirname(file_path)
                file_name = os.path.basename(file_path)
                
                self.db.add_file_content(
                    folder_path=folder_path,
                    file_name=file_name,
                    texts=[content],
                    embeddings=embedding
                )
                print(f"DB 업데이트 완료: {file_name}")
        except Exception as e:
            print(f"파일 처리 중 오류: {e}")
    
    def _remove_file_from_db(self, file_path):
        """파일 삭제 시 DB에서도 제거"""
        try:
            folder_path = os.path.dirname(file_path)
            file_name = os.path.basename(file_path)
            
            self.db.delete_file(folder_path, file_name)
            print(f"DB에서 파일 삭제 완료: {file_name}")
        except Exception as e:
            print(f"DB 파일 삭제 중 오류: {e}")


class FileWatcher:
    def __init__(self, db, ollama, extract_text_func, watch_directories=None):
        self.db = db
        self.ollama = ollama
        self.extract_text = extract_text_func
        self.observer = Observer()
        self.watch_directories = watch_directories or []
        self._start_watching()
    
    def _start_watching(self):
        """내부적으로 파일 시스템 감시 시작"""
        # 이벤트 핸들러 설정
        event_handler = FileEventHandler(self.db, self.ollama)
        
        # 각 폴더에 대해 감시 설정
        for folder_path in self.watch_directories:
            if os.path.exists(folder_path):
                self.observer.schedule(event_handler, folder_path, recursive=True)
                print(f"감시 시작: {folder_path}")
            else:
                print(f"폴더가 존재하지 않습니다: {folder_path}")
        
        # 감시 시작
        self.observer.start()
        print("파일 감시가 시작되었습니다.")
    
    def add_watch_directory(self, directory):
        """감시 디렉토리 추가 및 즉시 감시 시작"""
        if os.path.exists(directory):
            if directory not in self.watch_directories:
                self.watch_directories.append(directory)
                
                # 새 디렉토리에 대해 즉시 감시 시작
                event_handler = FileEventHandler(self.db, self.ollama)
                self.observer.schedule(event_handler, directory, recursive=True)
                
                print(f"폴더가 추가되고 감시가 시작되었습니다: {directory}")
                return True
            else:
                print("이미 추가된 폴더입니다.")
                return False
        else:
            print("존재하지 않는 폴더 경로입니다.")
            return False
    
    def remove_watch_directory(self, directory):
        """감시 디렉토리 제거 및 감시 중지"""
        if directory in self.watch_directories:
            self.watch_directories.remove(directory)
            
            # 해당 디렉토리의 감시 중지를 위해 observer 재시작
            self.observer.stop()
            self.observer.join()
            
            # 새로운 observer로 남은 디렉토리들 다시 감시
            self.observer = Observer()
            if self.watch_directories:
                self._start_watching()
            
            print(f"폴더가 제거되고 감시가 중지되었습니다: {directory}")
            return True
        else:
            print("감시 목록에 없는 폴더입니다.")
            return False
    
    def get_watch_directories(self):
        """현재 감시 중인 디렉토리 목록 반환"""
        return self.watch_directories.copy()
