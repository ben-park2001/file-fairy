"""File organizer with AI-powered categorization and naming."""

import logging
from pathlib import Path
from typing import List, Dict, Optional, Tuple, Any

from .config import AppConfig
from .file_utils import FileUtils
from .ai_processor import AIKeywordExtractor
from ..message_creator import InputMessageCreator


class FileOrganizer:
    """Main file organizer class with AI capabilities."""
    
    def __init__(self, use_ai: bool = True, model_path: Optional[str] = None, config: Optional[AppConfig] = None):
        """
        Initialize the File Organizer.
        
        Args:
            use_ai: Whether to enable AI features
            model_path: Path to AI model directory
            config: Application configuration
        """
        self.config = config or AppConfig.load()
        self.use_ai = use_ai
        self.model_path = model_path or self.config.ai.model_path
        self.ai_extractor: Optional[AIKeywordExtractor] = None
        self.message_creator = InputMessageCreator()
        self.file_utils = FileUtils(self.config)
        
        # Setup logging
        self._setup_logging()
        
        if self.use_ai:
            self._initialize_ai()
    
    def _setup_logging(self) -> None:
        """Setup logging configuration."""
        logging.basicConfig(
            level=logging.INFO,
            format='%(asctime)s - %(levelname)s - %(message)s',
            handlers=[
                logging.FileHandler(self.config.log_file, encoding='utf-8'),
                logging.StreamHandler()
            ]
        )
        self.logger = logging.getLogger(__name__)
    
    def _initialize_ai(self) -> bool:
        """
        Initialize AI model for keyword extraction and classification.
        
        Returns:
            True if successful, False otherwise
        """
        try:
            model_path_obj = Path(self.model_path)
            if not model_path_obj.exists():
                self.logger.warning(f"AI model directory not found: {model_path_obj}")
                self.logger.warning("AI features will be disabled. Only basic organization will be available.")
                self.use_ai = False
                return False
            
            self.logger.info("🧠 Initializing Gemma ONNX model...")
            self.ai_extractor = AIKeywordExtractor(model_path=str(model_path_obj))
            self.logger.info("✅ AI model loaded successfully!")
            return True
            
        except Exception as e:
            self.logger.warning(f"Failed to initialize AI model: {e}")
            self.logger.warning("AI features will be disabled. Only basic organization will be available.")
            self.use_ai = False
            return False
    
    def organize_directory(
        self, 
        source_dir: Path, 
        target_dir: Path,
        dry_run: bool = False
    ) -> Dict[str, Any]:
        """
        Organize files in a directory.
        
        Args:
            source_dir: Source directory to organize
            target_dir: Target directory for organized files
            dry_run: If True, only show what would be done
            
        Returns:
            Dictionary with organization results
        """
        if not source_dir.exists():
            raise FileNotFoundError(f"Source directory not found: {source_dir}")
        
        self.logger.info(f"Starting organization of: {source_dir}")
        
        files = self.file_utils.get_files_in_directory(source_dir, recursive=True)
        results = {
            'total_files': len(files),
            'processed_files': 0,
            'skipped_files': 0,
            'errors': [],
            'moved_files': []
        }
        
        for file_path in files:
            try:
                result = self._organize_single_file(file_path, target_dir, dry_run)
                if result['success']:
                    results['processed_files'] += 1
                    results['moved_files'].append(result)
                else:
                    results['skipped_files'] += 1
                    if result.get('error'):
                        results['errors'].append(result['error'])
            except Exception as e:
                error_msg = f"Error processing {file_path}: {e}"
                self.logger.error(error_msg)
                results['errors'].append(error_msg)
                results['skipped_files'] += 1
        
        self.logger.info(f"Organization complete. Processed: {results['processed_files']}, "
                        f"Skipped: {results['skipped_files']}, Errors: {len(results['errors'])}")
        
        return results
    
    def _organize_single_file(
        self, 
        file_path: Path, 
        target_dir: Path, 
        dry_run: bool = False
    ) -> Dict[str, Any]:
        """
        Organize a single file.
        
        Args:
            file_path: Path to the file to organize
            target_dir: Target directory for organized files
            dry_run: If True, only show what would be done
            
        Returns:
            Dictionary with organization result
        """
        try:
            # Get category and new filename
            if self.use_ai and self.ai_extractor:
                category, new_filename = self._get_ai_suggestions(file_path)
            else:
                category = self._get_basic_category(file_path)
                new_filename = file_path.name
            
            # Sanitize filename
            new_filename = FileUtils.sanitize_filename(new_filename)
            
            # Create target path
            target_category_dir = target_dir / category
            target_file_path = target_category_dir / new_filename
            
            result = {
                'original_path': str(file_path),
                'target_path': str(target_file_path),
                'category': category,
                'new_filename': new_filename,
                'success': True
            }
            
            if dry_run:
                self.logger.info(f"[DRY RUN] Would move {file_path} -> {target_file_path}")
                return result
            
            # Actually move the file
            success = FileUtils.move_file(file_path, target_file_path)
            result['success'] = success
            
            if success:
                self.logger.info(f"Moved: {file_path} -> {target_file_path}")
            else:
                result['error'] = f"Failed to move {file_path}"
                self.logger.error(result['error'])
            
            return result
            
        except Exception as e:
            return {
                'original_path': str(file_path),
                'success': False,
                'error': str(e)
            }
    
    def _get_ai_suggestions(self, file_path: Path) -> Tuple[str, str]:
        """
        Get AI-powered category and filename suggestions.
        
        Args:
            file_path: Path to the file
            
        Returns:
            Tuple of (category, suggested_filename)
        """
        try:
            # Extract file content if supported
            if self.message_creator.is_supported_format(str(file_path)):
                # Get file content for AI analysis
                file_content = self._extract_file_content(file_path)
                # Get AI suggestions
                suggestions = self.ai_extractor.process_file_content(
                    file_name=file_path.name,
                    file_content=file_content
                )
                
                # Parse AI response
                category, filename = self._parse_ai_suggestions(suggestions)
                return category, filename
            
        except Exception as e:
            self.logger.warning(f"AI processing failed for {file_path}: {e}")
        
        # Fallback to basic categorization
        return self._get_basic_category(file_path), file_path.name
    
    def _extract_file_content(self, file_path: Path) -> str:
        """
        Extract content from a file for AI analysis.
        
        Args:
            file_path: Path to the file
            
        Returns:
            Extracted content as string
        """
        try:
            message = self.message_creator.create_message(str(file_path))
            # Extract text content from message
            texts = [item['text'] for item in message[0]['content'] 
                    if item.get('type') == 'text' and 'text' in item]
            return texts[1] if len(texts) > 1 else ""
        except Exception as e:
            self.logger.warning(f"Content extraction failed for {file_path}: {e}")
            return ""
    
    def _parse_ai_suggestions(self, ai_response: str) -> Tuple[str, str]:
        """
        Parse AI response to extract category and filename.
        
        Args:
            ai_response: Raw AI response
            
        Returns:
            Tuple of (category, filename)
        """
        # Parse the AI response format:
        # 키워드: filename.ext
        # 폴더: category
        
        lines = ai_response.strip().split('\n')
        filename = None
        category = None
        
        for line in lines:
            line = line.strip()
            if line.startswith('키워드:'):
                filename = line.replace('키워드:', '').strip()
            elif line.startswith('폴더:'):
                category = line.replace('폴더:', '').strip()
        
        # Fallback values
        if not filename:
            filename = "untitled_file"
        if not category or category not in self.config.categories.categories:
            category = "기타"
        
        return category, filename
    
    def _get_basic_category(self, file_path: Path) -> str:
        """
        Get basic category based on file extension.
        
        Args:
            file_path: Path to the file
            
        Returns:
            Category name
        """
        extension = file_path.suffix.lower()
        return self.config.get_category_for_extension(extension)
    
    def get_organization_preview(self, source_dir: Path) -> List[Dict[str, Any]]:
        """
        Get a preview of how files would be organized without actually moving them.
        
        Args:
            source_dir: Source directory to analyze
            
        Returns:
            List of preview results
        """
        files = self.file_utils.get_files_in_directory(source_dir, recursive=True)
        preview = []
        
        for file_path in files[:10]:  # Limit preview to first 10 files
            try:
                if self.use_ai and self.ai_extractor:
                    category, new_filename = self._get_ai_suggestions(file_path)
                else:
                    category = self._get_basic_category(file_path)
                    new_filename = file_path.name
                
                preview.append({
                    'original_name': file_path.name,
                    'new_name': FileUtils.sanitize_filename(new_filename),
                    'category': category,
                    'path': str(file_path)
                })
                
                print(f"🔥 Preview: {file_path.name} -> {new_filename} in category '{category}'")
                
            except Exception as e:
                self.logger.warning(f"Preview failed for {file_path}: {e}")
        
        return preview
