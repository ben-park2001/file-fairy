import re
from llama_cpp import Llama
from pathlib import Path
from typing import Optional


class AIKeywordExtractor:
    """
    Llama-cpp를 사용한 AI 키워드 추출 및 파일 분류 클래스
    """
    
    def __init__(self, model_path: str, prompt_template: Optional[str] = None):
        """
        AI 모델을 초기화합니다.
        
        Args:
            model_path: GGUF 모델 파일 경로 또는 디렉토리 경로
            prompt_template: 사용자 정의 프롬프트 템플릿
        """
        # 모델 경로 처리 - 디렉토리인 경우 GGUF 파일을 찾음
        model_path_obj = Path(model_path)
        if model_path_obj.is_dir():
            # 디렉토리에서 .gguf 파일 찾기
            gguf_files = list(model_path_obj.glob("*.gguf"))
            if gguf_files:
                self.model_path = gguf_files[0]  # 첫 번째 gguf 파일 사용
            else:
                # 기본 경로 사용
                self.model_path = Path(r"E:\Downloads\Models\unsloth\gemma-3n-E2B-it-GGUF\gemma-3n-E2B-it-Q4_K_S.gguf")
        else:
            self.model_path = model_path_obj
            
        self.prompt_template = prompt_template or self._get_default_prompt_template()
        self.model = None
        
        # 모델 초기화
        self._load_model()
    
    def _get_default_prompt_template(self) -> str:
        """기본 프롬프트 템플릿을 반환합니다."""
        return """You are an expert file naming and classification specialist. Analyze the original filename and file content to generate optimal results.


TASK: 
1. Evaluate if the original filename accurately represents the file content
2. Generate a new descriptive filename (3-8 words, use underscores, include extension)
3. Suggest an appropriate folder name in Korean

GUIDELINES:
- If original filename is descriptive and accurate, use it as reference for the new filename
- If original filename is generic or unclear, create a completely new descriptive filename
- New filename should be concise but comprehensive
- Use clear, common terminology
- Folder name should be in Korean and categorize the file appropriately

EXAMPLES:
Original: document1.pdf, Content: quarterly sales report Q3 2024
→ 키워드: Q3_2024_quarterly_sales_report.pdf
→ 폴더: 매출보고서

Original: notes.txt, Content: project meeting discussing timeline and budget
→ 키워드: project_meeting_timeline_budget.txt  
→ 폴더: 회의록

OUTPUT FORMAT:
키워드: [new_filename_with_extension]
폴더: [Korean_folder_name]

Original filename: {file_name}
File content:
---
{file_content}
---"""

    def _load_model(self):
        """Llama-cpp 모델을 로드합니다."""
        try:
            print(f"🔄 Llama-cpp 모델 로딩 중: {self.model_path}")
            
            # 모델 파일 존재 확인
            if not self.model_path.exists():
                raise FileNotFoundError(f"모델 파일을 찾을 수 없습니다: {self.model_path}")
            
            # Llama 모델 로드 (에러 처리 개선)
            try:
                self.model = Llama(
                    model_path=str(self.model_path),
                    n_ctx=4096,  # Context window size
                    n_gpu_layers=-1,  # Use all GPU layers (-1 for all)
                    verbose=False,
                    chat_format="gemma"  # Gemma 모델용 chat format
                )
            except Exception as model_error:
                # chat_format 없이 재시도
                print("⚠️  Gemma chat format 실패, 기본 설정으로 재시도...")
                self.model = Llama(
                    model_path=str(self.model_path),
                    n_ctx=4096,
                    n_gpu_layers=-1,
                    verbose=False
                )
            
            print("✅ Llama-cpp 모델 로딩 완료!")
            
        except Exception as e:
            print(f"❌ 모델 로딩 실패: {e}")
            raise RuntimeError(f"모델 로딩 실패: {e}")
    
    def _generate_response(self, prompt: str, max_new_tokens: int = 64) -> str:
        """
        주어진 프롬프트에 대해 AI 응답을 생성합니다.
        
        Args:
            prompt: 입력 프롬프트
            max_new_tokens: 최대 생성 토큰 수
            
        Returns:
            생성된 응답 텍스트
        """
        try:
            # Llama-cpp를 사용한 chat completion
            response = self.model.create_chat_completion(
                messages=[
                    {"role": "user", "content": prompt}
                ],
                max_tokens=max_new_tokens,
                temperature=0,  # 낮은 temperature로 일관성 있는 결과
                stop=["<eos>", "</s>", "\n\n"]  # 적절한 중단점 설정
            )
            
            result = response['choices'][0]['message']['content'].strip()
            return result
            
        except Exception as e:
            print(f"응답 생성 중 오류: {e}")
            return "응답_생성_실패"
    
    def generate_response(self, prompt: str, max_tokens: int = 64) -> str:
        """
        외부에서 직접 호출할 수 있는 응답 생성 메서드
        """
        return self._generate_response(prompt, max_tokens)
    
    def process_file_content(self, file_name: str, file_content: str) -> str:
        """
        Process file content to get AI suggestions for naming and categorization.
        
        Args:
            file_name: Original filename
            file_content: File content to analyze
            
        Returns:
            AI response with keywords and folder suggestions
        """
        try:
            # Use the default prompt template to get structured response
            prompt = self.prompt_template.format(
                file_name=file_name,
                file_content=file_content[:2000]  # Limit content length
            )
            
            return self.generate_response(prompt)
            
        except Exception as e:
            print(f"파일 내용 처리 중 오류: {e}")
            return "키워드: 분석실패\n폴더: 기타"
    
    def extract_keywords(self, file_content: str, file_name: str, image_path: Optional[str] = None, audio_path: Optional[str] = None) -> str:
        """
        파일 내용으로부터 키워드를 추출합니다.
        
        Args:
            file_content: 파일 내용
            file_name: 파일명
            image_path: 이미지 파일 경로 (현재 텍스트만 지원)
            audio_path: 오디오 파일 경로 (현재 텍스트만 지원)
            
        Returns:
            추출된 키워드 (밑줄로 연결된 형태)
        """
        try:
            # 멀티모달 파일이 있을 경우 경고 메시지
            if image_path or audio_path:
                print("⚠️  이미지/오디오 파일은 현재 지원되지 않습니다. 텍스트 내용만 분석합니다.")
            
            # 파일 내용이 너무 길면 자르기 (토큰 제한)
            if len(file_content) > 2000:
                file_content = file_content[:2000] + "..."
            
            # 키워드 추출용 프롬프트
            prompt = f"""파일 내용을 분석하여 키워드를 추출해주세요.

원본 파일명: {file_name}
파일 내용:
---
{file_content}
---

요청사항:
1. 파일 내용을 요약하는 한국어 키워드 3개 (밑줄로 연결: 예시_키워드_형태)
2. 구체적이고 의미있는 키워드를 생성해주세요

응답 형식:
키워드: [키워드1_키워드2_키워드3]"""
            
            # AI 응답 생성
            response = self._generate_response(prompt, max_new_tokens=80)
            
            # 키워드 추출
            keywords = self._parse_keywords_from_response(response)
            return keywords
            
        except Exception as e:
            print(f"키워드 추출 중 오류: {e}")
            return "키워드추출실패"
    
    def classify_folder(self, file_content: str, file_name: str, image_path: Optional[str] = None, audio_path: Optional[str] = None) -> str:
        """
        파일 내용을 분석하여 적절한 폴더명을 제안합니다.
        
        Args:
            file_content: 파일 내용
            file_name: 파일명
            image_path: 이미지 파일 경로 (현재 텍스트만 지원)
            audio_path: 오디오 파일 경로 (현재 텍스트만 지원)
            
        Returns:
            제안된 폴더명
        """
        try:
            # 멀티모달 파일이 있을 경우 경고 메시지
            if image_path or audio_path:
                print("⚠️  이미지/오디오 파일은 현재 지원되지 않습니다. 텍스트 내용만 분석합니다.")
            
            # 파일 내용이 너무 길면 자르기
            if len(file_content) > 2000:
                file_content = file_content[:2000] + "..."
            
            # 폴더 분류용 프롬프트
            prompt = f"""파일 내용을 분석하여 적절한 폴더명을 제안해주세요.

원본 파일명: {file_name}
파일 내용:
---
{file_content}
---

요청사항:
1. 파일 내용을 바탕으로 이 파일이 속할 적절한 폴더명 1개 (한국어)
2. 구체적이고 의미있는 폴더명을 제안해주세요

응답 형식:
폴더: [폴더명]"""
            
            # AI 응답 생성
            response = self._generate_response(prompt, max_new_tokens=60)
            
            # 폴더명 추출
            folder_name = self._parse_folder_from_response(response)
            return folder_name
            
        except Exception as e:
            print(f"폴더 분류 중 오류: {e}")
            return "기타"
    
    def suggest_filename(self, file_content: str, original_name: str, extension: str, image_path: Optional[str] = None, audio_path: Optional[str] = None) -> str:
        """
        파일 내용을 분석하여 적절한 파일명을 제안합니다.
        
        Args:
            file_content: 파일 내용
            original_name: 원본 파일명 (확장자 제외)
            extension: 파일 확장자
            image_path: 이미지 파일 경로 (현재 텍스트만 지원)
            audio_path: 오디오 파일 경로 (현재 텍스트만 지원)
            
        Returns:
            제안된 파일명 (확장자 포함)
        """
        try:
            # 멀티모달 파일이 있을 경우 경고 메시지
            if image_path or audio_path:
                print("⚠️  이미지/오디오 파일은 현재 지원되지 않습니다. 텍스트 내용만 분석합니다.")
            
            # 파일 내용이 너무 길면 자르기 (토큰 제한)
            if len(file_content) > 2000:
                file_content = file_content[:2000] + "..."
            
            original_filename = original_name + extension
            
            # 파일명 제안용 프롬프트
            prompt = f"""You are an expert file naming specialist. Analyze the original filename and content to generate an optimal new filename.

Original filename: {original_filename}
File content:
---
{file_content}
---

TASK: Generate a new descriptive filename that represents the file content.

GUIDELINES:
- 3-8 words maximum, use underscores
- Include original extension: {extension}
- If original filename is descriptive, reference it
- If original filename is generic (like document1 or file), create a completely new descriptive name
- Be specific and descriptive

OUTPUT FORMAT:
키워드: [new_filename{extension}]"""
            
            # AI 응답 생성
            response = self._generate_response(prompt, max_new_tokens=80)
            
            # 새로운 파일명 추출
            new_filename = self._parse_keywords_from_response(response)
            
            # 확장자가 없다면 추가
            if not new_filename.endswith(extension):
                # 키워드에서 확장자 제거 후 올바른 확장자 추가
                if '.' in new_filename:
                    new_filename = new_filename.rsplit('.', 1)[0]
                new_filename += extension
            
            # 파일명으로 적합하지 않은 문자 제거
            safe_filename = re.sub(r'[<>:"/\\|?*]', '_', new_filename)
            safe_filename = re.sub(r'_+', '_', safe_filename).strip('_')
            
            # 파일명이 너무 길면 자르기
            name_part = safe_filename.replace(extension, '')
            if len(name_part) > 50:
                name_part = name_part[:50].rstrip('_')
                safe_filename = name_part + extension
            
            # 생성에 실패했거나 의미없는 결과인 경우 원본 사용
            if not safe_filename or safe_filename in [f"키워드추출실패{extension}", extension]:
                return original_filename
                
            return safe_filename
            
        except Exception as e:
            print(f"파일명 제안 중 오류: {e}")
            return f"{original_name}{extension}"
    
    def _parse_keywords_from_response(self, response: str) -> str:
        """AI 응답에서 키워드를 파싱합니다."""
        try:
            # 무의미한 키워드 필터링 목록
            meaningless_keywords = {"없음", "비어있음", "알수없음", "모름", "정보없음", "내용없음", "빈내용"}
            
            # "키워드:" 라벨 찾기 - 새로운 파일명 형식 지원
            keyword_match = re.search(r'키워드\s*:\s*\[?([^\]\n]+)\]?', response, re.IGNORECASE)
            if keyword_match:
                keywords = keyword_match.group(1).strip()
                # 특수문자 정리 (대괄호 제거, 파일명에 적합하지 않은 문자)
                keywords = re.sub(r'[<>:"/\\|?*\[\]]', '', keywords)
                
                # 완전한 파일명 형식인지 확인 (확장자 포함)
                if '.' in keywords and not keywords.startswith('.'):
                    # 이미 완전한 파일명 형식
                    return keywords
                
                # 키워드 형식인 경우 무의미한 키워드 필터링
                keyword_parts = [k.strip() for k in keywords.split('_') if k.strip()]
                filtered_parts = [k for k in keyword_parts if k not in meaningless_keywords]
                
                if len(filtered_parts) >= 1:  # 최소 1개 이상의 의미있는 키워드가 있을 때 반환
                    return '_'.join(filtered_parts)
            
            # 대체 패턴들 시도
            lines = response.split('\n')
            for line in lines:
                if '키워드' in line and ':' in line:
                    keywords = line.split(':', 1)[1].strip()
                    keywords = re.sub(r'[<>:"/\\|?*\[\]]', '', keywords)
                    if keywords:
                        # 완전한 파일명 형식인지 확인
                        if '.' in keywords and not keywords.startswith('.'):
                            return keywords
                        
                        # 키워드 형식인 경우 무의미한 키워드 필터링
                        keyword_parts = [k.strip() for k in keywords.split('_') if k.strip()]
                        filtered_parts = [k for k in keyword_parts if k not in meaningless_keywords]
                        
                        if len(filtered_parts) >= 1:
                            return '_'.join(filtered_parts)
            
            # 응답에서 파일명 패턴을 직접 찾기 시도
            filename_pattern = re.search(r'([a-zA-Z가-힣0-9_]+\.[a-zA-Z0-9]+)', response)
            if filename_pattern:
                return filename_pattern.group(1)
            
            return "키워드추출실패"
            
        except Exception:
            return "키워드추출실패"
    
    def _parse_folder_from_response(self, response: str) -> str:
        """AI 응답에서 폴더명을 파싱합니다."""
        try:
            # "폴더:" 라벨 찾기
            folder_match = re.search(r'폴더\s*:\s*\[?([^\]\n]+)\]?', response, re.IGNORECASE)
            if folder_match:
                folder_name = folder_match.group(1).strip()
                # 폴더명으로 적합하지 않은 문자 제거
                folder_name = re.sub(r'[<>:"/\\|?*\[\]]', '', folder_name)
                return folder_name
            
            # 대체 패턴들 시도
            lines = response.split('\n')
            for line in lines:
                if '폴더' in line and ':' in line:
                    folder_name = line.split(':', 1)[1].strip()
                    folder_name = re.sub(r'[<>:"/\\|?*\[\]]', '', folder_name)
                    if folder_name:
                        return folder_name
            
            return "기타"
            
        except Exception:
            return "기타"
