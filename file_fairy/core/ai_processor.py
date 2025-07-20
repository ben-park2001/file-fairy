import re
import onnxruntime
import numpy as np
from transformers import AutoConfig, AutoProcessor
from pathlib import Path
from typing import Optional


class AIKeywordExtractor:
    """
    Gemma ONNX 모델을 사용한 AI 키워드 추출 및 파일 분류 클래스
    """
    
    def __init__(self, model_path: str, prompt_template: Optional[str] = None):
        """
        AI 모델을 초기화합니다.
        
        Args:
            model_path: ONNX 모델들이 있는 디렉토리 경로
            prompt_template: 사용자 정의 프롬프트 템플릿
        """
        self.model_dir = Path(model_path)
        self.prompt_template = prompt_template or self._get_default_prompt_template()
        
        # 모델 관련 변수들
        self.processor = None
        self.config = None
        self.embed_session = None
        self.decoder_session = None
        self.vision_session = None
        self.audio_session = None
        self.eos_token_id = 106
        self.image_token_id = None
        self.audio_token_id = None
        
        # 모델 초기화
        self._load_models()
    
    def _get_default_prompt_template(self) -> str:
        """기본 프롬프트 템플릿을 반환합니다."""
        return """You are an expert file naming and classification specialist. Analyze the original filename and file content to generate optimal results.

Original filename: {file_name}
File content:
---
{file_content}
---

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
폴더: [Korean_folder_name]"""

    def _load_models(self):
        """ONNX 모델들을 로드합니다."""
        try:
            print("🔄 Processor와 Config 로딩 중...")
            
            # 첫 번째 시도: 로컬 파일만 사용
            try:
                self.processor = AutoProcessor.from_pretrained(
                    str(self.model_dir),
                    local_files_only=True
                )
                self.config = AutoConfig.from_pretrained(
                    str(self.model_dir),
                    local_files_only=True
                )
                print("✅ 로컬 파일로 Config/Processor 로드 성공")
            except Exception as local_e:
                print(f"⚠️  로컬 파일 로드 실패, 온라인 fallback 시도: {local_e}")
                # Fallback: 온라인에서 로드
                model_id = "google/gemma-3n-E2B-it"
                self.processor = AutoProcessor.from_pretrained(model_id)
                self.config = AutoConfig.from_pretrained(model_id)
                print("✅ 온라인으로 Config/Processor 로드 성공")
            
            # ONNX 모델 경로들
            embed_model_path = self.model_dir / "onnx/embed_tokens_quantized.onnx"
            decoder_model_path = self.model_dir / "onnx/decoder_model_merged_q4.onnx"
            vision_model_path = self.model_dir / "onnx/vision_encoder_quantized.onnx"
            audio_model_path = self.model_dir / "onnx/audio_encoder_quantized.onnx"
            
            print(f"🔍 모델 파일 경로 확인:")
            print(f"   Embed: {embed_model_path} (존재: {embed_model_path.exists()})")
            print(f"   Decoder: {decoder_model_path} (존재: {decoder_model_path.exists()})")
            print(f"   Vision: {vision_model_path} (존재: {vision_model_path.exists()})")
            print(f"   Audio: {audio_model_path} (존재: {audio_model_path.exists()})")
            
            # 모델 파일 존재 확인
            if not embed_model_path.exists():
                raise FileNotFoundError(f"Embed model not found: {embed_model_path}")
            if not decoder_model_path.exists():
                raise FileNotFoundError(f"Decoder model not found: {decoder_model_path}")
            if not vision_model_path.exists():
                print(f"⚠️  Vision model not found: {vision_model_path}")
                print("    Vision 기능이 비활성화됩니다.")
                self.vision_session = None
            if not audio_model_path.exists():
                print(f"⚠️  Audio model not found: {audio_model_path}")
                print("    Audio 기능이 비활성화됩니다.")
                self.audio_session = None
            
            # Provider 설정 (CUDA 사용 가능 여부 확인)
            available_providers = onnxruntime.get_available_providers()
            print(f"🔧 사용 가능한 providers: {available_providers}")
            
            if 'CUDAExecutionProvider' in available_providers:
                providers = ['CUDAExecutionProvider', 'CPUExecutionProvider']
                print("✅ CUDA Provider 사용 가능")
            else:
                providers = ['CPUExecutionProvider']
                print("⚠️  CUDA 사용 불가, CPU만 사용")
                
            providers= ['CUDAExecutionProvider', 'CPUExecutionProvider']
            
            # ONNX 세션 생성
            print("🔄 ONNX 세션 생성 중...")
            
            # Embed 모델
            try:
                self.embed_session = onnxruntime.InferenceSession(str(embed_model_path), providers=providers)
                embed_provider = self.embed_session.get_providers()[0]
                print(f"✅ Embed model using: {embed_provider}")
            except Exception as e:
                raise RuntimeError(f"Embed model 로드 실패: {e}")
            
            # Decoder 모델  
            try:
                self.decoder_session = onnxruntime.InferenceSession(str(decoder_model_path), providers=providers)
                decoder_provider = self.decoder_session.get_providers()[0]
                print(f"✅ Decoder model using: {decoder_provider}")
            except Exception as e:
                raise RuntimeError(f"Decoder model 로드 실패: {e}")
            
            # Vision 모델 (선택적)
            if vision_model_path.exists():
                try:
                    self.vision_session = onnxruntime.InferenceSession(str(vision_model_path), providers=providers)
                    vision_provider = self.vision_session.get_providers()[0]
                    print(f"✅ Vision model using: {vision_provider}")
                    
                    # Vision 모델의 입력/출력 정보 확인
                    print(f"🔍 Vision model 입력 정보:")
                    for input_meta in self.vision_session.get_inputs():
                        print(f"   이름: {input_meta.name}, 타입: {input_meta.type}, 형태: {input_meta.shape}")
                    
                    print(f"🔍 Vision model 출력 정보:")
                    for output_meta in self.vision_session.get_outputs():
                        print(f"   이름: {output_meta.name}, 타입: {output_meta.type}, 형태: {output_meta.shape}")
                        
                except Exception as e:
                    print(f"⚠️  Vision model 로드 실패: {e}")
                    print("    텍스트 전용 모드로 작동합니다.")
                    self.vision_session = None
            
            # Audio 모델 (선택적)
            if audio_model_path.exists():
                try:
                    self.audio_session = onnxruntime.InferenceSession(str(audio_model_path), providers=providers)
                    audio_provider = self.audio_session.get_providers()[0]
                    print(f"✅ Audio model using: {audio_provider}")
                    
                    # Audio 모델의 입력/출력 정보 확인
                    print(f"🔍 Audio model 입력 정보:")
                    for input_meta in self.audio_session.get_inputs():
                        print(f"   이름: {input_meta.name}, 타입: {input_meta.type}, 형태: {input_meta.shape}")
                    
                    print(f"🔍 Audio model 출력 정보:")
                    for output_meta in self.audio_session.get_outputs():
                        print(f"   이름: {output_meta.name}, 타입: {output_meta.type}, 형태: {output_meta.shape}")
                        
                except Exception as e:
                    print(f"⚠️  Audio model 로드 실패: {e}")
                    print("    오디오 처리 기능이 비활성화됩니다.")
                    self.audio_session = None
            
            # 설정값들
            self.num_key_value_heads = self.config.text_config.num_key_value_heads
            self.head_dim = self.config.text_config.head_dim
            self.num_hidden_layers = self.config.text_config.num_hidden_layers
            self.image_token_id = getattr(self.config, 'image_token_id', 262145)
            self.audio_token_id = getattr(self.config, 'audio_token_id', None)
            
            print(f"🔧 Config 값들:")
            print(f"   Key-Value heads: {self.num_key_value_heads}")
            print(f"   Head dimension: {self.head_dim}")
            print(f"   Hidden layers: {self.num_hidden_layers}")
            print(f"   Image token ID: {self.image_token_id}")
            print(f"   Audio token ID: {self.audio_token_id}")
            
            print("✅ Gemma ONNX 모델 로딩 완료!")
            
        except Exception as e:
            print(f"❌ 모델 로딩 실패 상세 정보:")
            print(f"   오류: {e}")
            print(f"   타입: {type(e).__name__}")
            import traceback
            traceback.print_exc()
            raise RuntimeError(f"모델 로딩 실패: {e}")
    
    def _generate_response(self, prompt: str, max_new_tokens: int = 64, image_path: Optional[str] = None, audio_path: Optional[str] = None) -> str:
        """
        주어진 프롬프트에 대해 AI 응답을 생성합니다.
        
        Args:
            prompt: 입력 프롬프트
            max_new_tokens: 최대 생성 토큰 수
            image_path: 이미지 파일 경로 (선택사항)
            audio_path: 오디오 파일 경로 (선택사항)
            
        Returns:
            생성된 응답 텍스트
        """
        try:
            # 메시지 형태로 변환
            content = [{"type": "text", "text": prompt}]
            if image_path and Path(image_path).exists():
                content.append({"type": "image", "image": str(image_path)})
            if audio_path and Path(audio_path).exists():
                content.append({"type": "audio", "audio": str(audio_path)})
            
            messages = [{"role": "user", "content": content}]
            
            # 토크나이저로 입력 처리
            inputs = self.processor.apply_chat_template(
                messages,
                add_generation_prompt=True,
                tokenize=True,
                return_dict=True,
                return_tensors="pt",
            )
            
            input_ids = inputs["input_ids"].numpy().astype(np.int64)
            attention_mask = inputs["attention_mask"].numpy().astype(np.int64)
            position_ids = np.cumsum(attention_mask, axis=-1) - 1
            
            # 이미지 데이터 처리
            pixel_values = inputs.get("pixel_values")
            if pixel_values is not None:
                pixel_values = pixel_values.numpy()
            
            # 오디오 데이터 처리
            input_features = inputs.get("input_features")
            input_features_mask = inputs.get("input_features_mask")
            if input_features is not None:
                input_features = input_features.numpy().astype(np.float32)
            if input_features_mask is not None:
                input_features_mask = input_features_mask.numpy()
            
            # KV 캐시 초기화
            batch_size = input_ids.shape[0]
            past_key_values = {
                f"past_key_values.{layer}.{kv}": np.zeros([batch_size, self.num_key_value_heads, 0, self.head_dim], dtype=np.float32)
                for layer in range(self.num_hidden_layers)
                for kv in ("key", "value")
            }
            
            # 생성 루프
            generated_tokens = np.array([[]], dtype=np.int64)
            image_features = None
            audio_features = None
            
            for i in range(max_new_tokens):
                # 임베딩 생성
                inputs_embeds, per_layer_inputs = self.embed_session.run(None, {"input_ids": input_ids})
                
                # 이미지 처리 (첫 번째 반복에서만)
                if image_features is None and pixel_values is not None and self.vision_session is not None and self.image_token_id is not None:
                    image_features = self.vision_session.run(
                        ["image_features"],
                        {"pixel_values": pixel_values}
                    )[0]
                    
                    # 이미지 토큰을 이미지 특징으로 교체
                    mask = (input_ids == self.image_token_id).reshape(-1)
                    flat_embeds = inputs_embeds.reshape(-1, inputs_embeds.shape[-1])
                    flat_embeds[mask] = image_features.reshape(-1, image_features.shape[-1])
                    inputs_embeds = flat_embeds.reshape(inputs_embeds.shape)
                
                # 오디오 처리 (첫 번째 반복에서만)
                if audio_features is None and input_features is not None and input_features_mask is not None and self.audio_session is not None and self.audio_token_id is not None:
                    audio_features = self.audio_session.run(
                        ["audio_features"],
                        {
                            "input_features": input_features,
                            "input_features_mask": input_features_mask,
                        }
                    )[0]
                    
                    # 오디오 토큰을 오디오 특징으로 교체
                    mask = (input_ids == self.audio_token_id).reshape(-1)
                    flat_embeds = inputs_embeds.reshape(-1, inputs_embeds.shape[-1])
                    flat_embeds[mask] = audio_features.reshape(-1, audio_features.shape[-1])
                    inputs_embeds = flat_embeds.reshape(inputs_embeds.shape)
                
                # 디코더 실행
                logits, *present_key_values = self.decoder_session.run(None, dict(
                    inputs_embeds=inputs_embeds,
                    per_layer_inputs=per_layer_inputs,
                    position_ids=position_ids,
                    **past_key_values,
                ))
                
                # 다음 토큰 선택
                input_ids = logits[:, -1].argmax(-1, keepdims=True)
                attention_mask = np.ones_like(input_ids)
                position_ids = position_ids[:, -1:] + 1
                
                # KV 캐시 업데이트
                for j, key in enumerate(past_key_values):
                    past_key_values[key] = present_key_values[j]
                
                generated_tokens = np.concatenate([generated_tokens, input_ids], axis=-1)
                
                # EOS 토큰 확인
                if (input_ids == self.eos_token_id).all():
                    break
            
            # 결과 디코딩
            response = self.processor.batch_decode(generated_tokens, skip_special_tokens=True)[0]
            return response.strip()
            
        except Exception as e:
            print(f"응답 생성 중 오류: {e}")
            return "응답_생성_실패"
    
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
            image_path: 이미지 파일 경로 (선택사항)
            audio_path: 오디오 파일 경로 (선택사항)
            
        Returns:
            추출된 키워드 (밑줄로 연결된 형태)
        """
        try:
            # 파일 내용이 너무 길면 자르기 (토큰 제한)
            if len(file_content) > 2000:
                file_content = file_content[:2000] + "..."
            
            # 프롬프트 생성
            if image_path or audio_path:
                content_desc = []
                if image_path:
                    content_desc.append("이미지")
                if audio_path:
                    content_desc.append("오디오")
                content_type = "와 ".join(content_desc)
                
                if file_content.strip():
                    prompt = f"""{content_type}와 텍스트 내용을 분석하여 다음을 제공해주세요:

Original filename: {file_name}
텍스트 내용:
---
{file_content}
---

요청사항:
1. {content_type}와 텍스트 내용을 종합하여 요약하는 한국어 키워드 3개 (밑줄로 연결: 예시_키워드_형태)

응답 형식:
키워드: [키워드1_키워드2_키워드3]"""
                else:
                    prompt = f"""{content_type}를 분석하여 다음을 제공해주세요:

Original filename: {file_name}

요청사항:
1. {content_type}의 내용을 요약하는 한국어 키워드 3개 (밑줄로 연결: 예시_키워드_형태)
2. 구체적이고 의미있는 키워드를 생성해주세요

응답 형식:
키워드: [키워드1_키워드2_키워드3]"""
            else:
                prompt = self.prompt_template.format(
                    file_content=file_content,
                    file_name=file_name
                )
            
            # AI 응답 생성
            response = self._generate_response(prompt, image_path=image_path, audio_path=audio_path)
            
            # 모델이 없고 미디어만 있는 경우 기본 키워드 반환
            if image_path and not self.vision_session and not file_content.strip():
                return "이미지_분석불가_비전모델없음"
            if audio_path and not self.audio_session and not file_content.strip():
                return "오디오_분석불가_오디오모델없음"
            
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
            image_path: 이미지 파일 경로 (선택사항)
            audio_path: 오디오 파일 경로 (선택사항)
            
        Returns:
            제안된 폴더명
        """
        try:
            # 파일 내용이 너무 길면 자르기
            if len(file_content) > 2000:
                file_content = file_content[:2000] + "..."
            
            # 프롬프트 생성
            if image_path or audio_path:
                content_desc = []
                if image_path:
                    content_desc.append("이미지")
                if audio_path:
                    content_desc.append("오디오")
                content_type = "와 ".join(content_desc)
                
                if file_content.strip():
                    prompt = f"""{content_type}와 텍스트 내용을 분석하여 다음을 제공해주세요:

텍스트 내용:
---
{file_content}
---

요청사항:
1. {content_type}와 텍스트 내용을 종합하여 이 파일이 속할 적절한 폴더명 1개 (한국어)

응답 형식:
폴더: [폴더명]"""
                else:
                    prompt = f"""{content_type}를 분석하여 다음을 제공해주세요:

요청사항:
1. {content_type}의 내용을 바탕으로 이 파일이 속할 적절한 폴더명 1개 (한국어)
2. 구체적이고 의미있는 폴더명을 제안해주세요

응답 형식:
폴더: [폴더명]"""
            else:
                prompt = self.prompt_template.format(
                    file_content=file_content,
                    file_name=file_name
                )
            
            # AI 응답 생성
            response = self._generate_response(prompt, image_path=image_path, audio_path=audio_path)
            
            # 모델이 없고 미디어만 있는 경우 기본 폴더명 반환
            if image_path and not self.vision_session and not file_content.strip():
                return "이미지"
            if audio_path and not self.audio_session and not file_content.strip():
                return "오디오"
            
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
            image_path: 이미지 파일 경로 (선택사항)
            audio_path: 오디오 파일 경로 (선택사항)
            
        Returns:
            제안된 파일명 (확장자 포함)
        """
        try:
            # 파일 내용이 너무 길면 자르기 (토큰 제한)
            if len(file_content) > 2000:
                file_content = file_content[:2000] + "..."
            
            original_filename = original_name + extension
            
            # 프롬프트 생성 - 기존 제목과 내용을 함께 분석
            if image_path or audio_path:
                content_desc = []
                if image_path:
                    content_desc.append("이미지")
                if audio_path:
                    content_desc.append("오디오")
                content_type = "와 ".join(content_desc)
                
                if file_content.strip():
                    prompt = f"""You are an expert file naming specialist for multimedia files. Analyze the original filename and content to generate an optimal new filename.

Original filename: {original_filename}
Content type: {content_type}
Text content:
---
{file_content}
---

TASK: Generate a new descriptive filename that represents both the {content_type} and text content.

GUIDELINES:
- 3-8 words maximum, use underscores
- Include original extension: {extension}
- If original filename is descriptive, reference it
- If original filename is generic (like IMG_001 or audio_file), create a completely new name
- Be specific and descriptive

OUTPUT FORMAT:
키워드: [new_filename{extension}]"""
                else:
                    prompt = f"""You are an expert file naming specialist. Analyze the original filename and {content_type} to generate an optimal new filename.

Original filename: {original_filename}
Content type: {content_type}

TASK: Generate a new descriptive filename for this {content_type} file.

GUIDELINES:
- 3-8 words maximum, use underscores
- Include original extension: {extension}
- If original filename is descriptive, reference it
- If original filename is generic, create a completely new descriptive name

OUTPUT FORMAT:
키워드: [new_filename{extension}]"""
            else:
                # 텍스트 파일의 경우 기본 템플릿 사용
                prompt = self.prompt_template.format(
                    file_content=file_content,
                    file_name=original_filename
                )
            
            # AI 응답 생성
            response = self._generate_response(prompt, image_path=image_path, audio_path=audio_path)
            
            # 모델이 없고 미디어만 있는 경우 기본 파일명 반환
            if image_path and not self.vision_session and not file_content.strip():
                return f"image_content{extension}"
            if audio_path and not self.audio_session and not file_content.strip():
                return f"audio_content{extension}"
            
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