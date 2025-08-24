import webview
import os
from cli import FileFairyCli


class FileFairyAPI:
    """FileFairy GUI API - CLI와 웹 인터페이스를 연결"""
    
    def __init__(self):
        self.cli = FileFairyCli()
        self.current_tab = "watch"  # 현재 활성 탭
    
    def get_watch_folders(self):
        """감시 중인 폴더 목록 반환"""
        try:
            folders = self.cli.get_watch_folders()
            return {"success": True, "data": folders}
        except Exception as e:
            return {"success": False, "error": str(e)}
    
    def add_watch_folder(self, folder_path):
        """감시 폴더 추가"""
        try:
            result = self.cli.add_watch_folder(folder_path)
            if result:
                return {"success": True, "message": f"폴더 '{folder_path}' 감시 시작"}
            else:
                return {"success": False, "error": "폴더 추가 실패"}
        except Exception as e:
            return {"success": False, "error": str(e)}
    
    def remove_watch_folder(self, folder_path):
        """감시 폴더 제거"""
        try:
            result = self.cli.remove_watch_folder(folder_path)
            if result:
                return {"success": True, "message": f"폴더 '{folder_path}' 감시 중지"}
            else:
                return {"success": False, "error": "폴더 제거 실패"}
        except Exception as e:
            return {"success": False, "error": str(e)}
    
    def search_files(self, query):
        """파일 검색 - 강화된 버전"""
        try:
            if not query.strip():
                return {"success": False, "error": "검색어를 입력해주세요"}
            
            results = self.cli.searcher(query)
            
            # 결과 정렬 및 추가 정보
            enhanced_results = []
            for result in results:
                enhanced_result = {
                    **result,
                    "file_path": os.path.join(result['folder_path'], result['file_name']),
                    "exists": os.path.exists(os.path.join(result['folder_path'], result['file_name'])),
                    "size": self._get_file_size(result['folder_path'], result['file_name']),
                    "modified": self._get_file_modified(result['folder_path'], result['file_name'])
                }
                enhanced_results.append(enhanced_result)
            
            return {
                "success": True, 
                "data": {
                    "results": enhanced_results,
                    "total_count": len(enhanced_results),
                    "query": query
                }
            }
        except Exception as e:
            return {"success": False, "error": f"검색 실패: {str(e)}"}
    
    def get_filename_suggestions(self, file_paths):
        """파일명 추천 - 향상된 에러 처리"""
        try:
            if not file_paths:
                return {"success": False, "error": "파일을 선택해주세요"}
            
            # 유효한 파일 경로만 필터링
            valid_paths = []
            invalid_paths = []
            
            for path in file_paths:
                if os.path.exists(path) and os.path.isfile(path):
                    valid_paths.append(path)
                else:
                    invalid_paths.append(path)
            
            if not valid_paths:
                return {
                    "success": False, 
                    "error": f"유효한 파일이 없습니다. 확인된 경로: {invalid_paths}"
                }
            
            # CLI를 통해 파일명 추천 실행
            results = self.cli.filename_changer(valid_paths)
            
            # 결과에 추가 정보 포함
            enhanced_results = []
            for result in results:
                enhanced_result = {
                    **result,
                    "file_size": os.path.getsize(result['file_path']) if os.path.exists(result['file_path']) else 0,
                    "file_extension": os.path.splitext(result['file_path'])[1],
                    "processing_status": "success"
                }
                enhanced_results.append(enhanced_result)
            
            response_data = {
                "results": enhanced_results,
                "processed_count": len(valid_paths),
                "skipped_count": len(invalid_paths),
                "skipped_files": invalid_paths
            }
            
            return {"success": True, "data": response_data}
            
        except Exception as e:
            return {
                "success": False, 
                "error": f"파일명 추천 처리 중 오류 발생: {str(e)}"
            }
    
    def get_system_info(self):
        """시스템 정보 반환"""
        try:
            import platform
            return {
                "success": True,
                "data": {
                    "platform": platform.system(),
                    "python_version": platform.python_version(),
                    "cwd": os.getcwd(),
                    "supported_extensions": ['.txt', '.md', '.pdf', '.docx', '.doc', 
                                           '.xlsx', '.xls', '.pptx', '.ppt', '.hwp', 
                                           '.hwpx', '.csv', '.log']
                }
            }
        except Exception as e:
            return {"success": False, "error": str(e)}
    
    def _get_file_size(self, folder_path, file_name):
        """파일 크기 반환"""
        try:
            file_path = os.path.join(folder_path, file_name)
            return os.path.getsize(file_path) if os.path.exists(file_path) else 0
        except:
            return 0
    
    def apply_filename_change(self, file_path, new_filename):
        """파일명 실제 변경"""
        try:
            if not os.path.exists(file_path):
                return {"success": False, "error": "파일이 존재하지 않습니다"}
            
            directory = os.path.dirname(file_path)
            new_path = os.path.join(directory, new_filename)
            
            if os.path.exists(new_path):
                return {"success": False, "error": "동일한 이름의 파일이 이미 존재합니다"}
            
            os.rename(file_path, new_path)
            return {
                "success": True, 
                "message": f"파일명이 변경되었습니다",
                "old_path": file_path,
                "new_path": new_path
            }
        except Exception as e:
            return {"success": False, "error": f"파일명 변경 실패: {str(e)}"}
    
    def get_watch_folder_stats(self):
        """감시 폴더 통계"""
        try:
            folders = self.cli.get_watch_folders()
            stats = []
            
            for folder in folders:
                if os.path.exists(folder):
                    file_count = sum(len(files) for _, _, files in os.walk(folder))
                    stats.append({
                        "path": folder,
                        "file_count": file_count,
                        "exists": True
                    })
                else:
                    stats.append({
                        "path": folder,
                        "file_count": 0,
                        "exists": False
                    })
            
            return {"success": True, "data": stats}
        except Exception as e:
            return {"success": False, "error": str(e)}
    
    def handle_drop(self, file_info):
        """드래그앤드롭 처리 - 향상된 버전"""
        try:
            # file_info에서 경로 추출
            if isinstance(file_info, dict):
                path = file_info.get('path', '')
                name = file_info.get('name', '')
                size = file_info.get('size', 0)
                file_type = file_info.get('type', '')
            elif isinstance(file_info, str):
                path = file_info
                name = os.path.basename(file_info) if file_info else ''
                size = 0
                file_type = ''
            else:
                path = str(file_info)
                name = os.path.basename(path) if path else ''
                size = 0
                file_type = ''

            # 경로가 없거나 상대경로인 경우 처리
            if not path or path == name:
                # 현재 작업 디렉토리 기준으로 절대 경로 생성 시도
                if name:
                    potential_paths = [
                        os.path.join(os.getcwd(), name),
                        os.path.join(os.path.expanduser("~"), "Desktop", name),
                        os.path.join(os.path.expanduser("~"), "Downloads", name)
                    ]
                    
                    for potential_path in potential_paths:
                        if os.path.exists(potential_path):
                            path = potential_path
                            break
                
                if not path or not os.path.exists(path):
                    return {
                        "success": False, 
                        "error": f"파일 경로를 찾을 수 없습니다: {name}. 브라우저 보안상 일부 파일의 경로에 접근할 수 없습니다."
                    }

            # 절대 경로로 변환
            abs_path = os.path.abspath(path)
            
            if not os.path.exists(abs_path):
                return {
                    "success": False, 
                    "error": f"파일이 존재하지 않습니다: {abs_path}"
                }

            # 파일 정보 수집
            is_directory = os.path.isdir(abs_path)
            actual_size = size
            
            # 실제 파일 크기 확인 (size가 0이거나 없는 경우)
            if not actual_size and not is_directory:
                try:
                    actual_size = os.path.getsize(abs_path)
                except:
                    actual_size = 0

            # 지원되는 파일 타입 확인
            supported_extensions = ['.txt', '.md', '.pdf', '.docx', '.doc', '.xlsx', '.xls', 
                                  '.pptx', '.ppt', '.hwp', '.hwpx', '.csv', '.log']
            
            file_extension = os.path.splitext(abs_path)[1].lower()
            is_supported = is_directory or file_extension in supported_extensions

            return {
                "success": True, 
                "data": {
                    "path": abs_path,
                    "name": os.path.basename(abs_path),
                    "size": actual_size,
                    "type": file_type,
                    "extension": file_extension,
                    "is_directory": is_directory,
                    "is_supported": is_supported,
                    "readable": os.access(abs_path, os.R_OK),
                    "last_modified": os.path.getmtime(abs_path) if os.path.exists(abs_path) else None
                }
            }
            
        except Exception as e:
            return {
                "success": False, 
                "error": f"파일 처리 중 오류 발생: {str(e)}"
            }
    
    def set_current_tab(self, tab_name):
        """현재 활성 탭 설정"""
        self.current_tab = tab_name
        return {"success": True, "current_tab": tab_name}


def main():
    """메인 GUI 실행"""
    api = FileFairyAPI()
    
    # 웹뷰 창 생성
    window = webview.create_window(
        title='FileFairy - 파일 관리 도우미',
        url='web/index.html',  # HTML 파일 경로
        js_api=api,
        width=1000,
        height=700,
        min_size=(800, 600),
        resizable=True
    )
    
    # 웹뷰 시작
    webview.start(debug=True)


if __name__ == '__main__':
    main()
