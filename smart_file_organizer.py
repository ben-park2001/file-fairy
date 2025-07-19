#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
Smart File Organizer - AI-powered file organization tool
Integrates fileio_utils package with AI keyword extraction and classification
"""

import sys
import os
import argparse
import datetime
import re
from pathlib import Path
from typing import List, Dict, Optional, Tuple, Any
import traceback

# Add fileio_utils to path if not already available
try:
    from fileio_utils import InputMessageCreator, create_message_from_file, is_supported_file
    from fileio_utils.constants import SupportedFormats, MessageTypes
except ImportError as e:
    print(f"Error: Failed to import fileio_utils package: {e}")
    print("Please ensure fileio_utils is available in your Python path.")
    sys.exit(1)

# Import AI module
try:
    from ai_module import AIKeywordExtractor
except ImportError:
    print("Error: 'ai_module.py' file not found. Please ensure it's in the same directory.")
    sys.exit(1)

# --- Configuration ---
DEFAULT_MODEL_PATH = "gemma-3n-E2B-it-ONNX"
LOG_FILE_NAME = "smart_organizer.log"

# File categories with Korean names
FILE_CATEGORIES = {
    '문서': SupportedFormats.DOCUMENTS + ['.hwp'],
    '이미지': SupportedFormats.IMAGES + ['.bmp', '.tiff'],
    '데이터': SupportedFormats.DATA + ['.xls'],
    '음악': SupportedFormats.AUDIO + ['.ogg'],
    '동영상': ['.mp4', '.avi', '.mkv', '.mov', '.wmv'],
    '압축파일': ['.zip', '.7z', '.rar', '.tar', '.gz'],
    '코드': ['.py', '.js', '.html', '.css', '.java', '.cpp', '.c'],
    '기타': 'others'
}

# Files to exclude from processing
EXCLUDE_PATTERNS = [
    '.git', '.DS_Store', 'node_modules', 'venv', '__pycache__',
    'Thumbs.db', '.vscode', '.idea', 'smart_organizer.log'
]

# Global AI extractor instance (lazy loading)
AI_EXTRACTOR: Optional[AIKeywordExtractor] = None
FILE_MESSAGE_CREATOR: Optional[InputMessageCreator] = None


class SmartFileOrganizer:
    """Smart file organizer with AI capabilities and fileio_utils integration."""
    
    def __init__(self, model_path: str = None, use_ai: bool = True):
        """
        Initialize the Smart File Organizer.
        
        Args:
            model_path: Path to AI model directory
            use_ai: Whether to enable AI features
        """
        self.model_path = model_path or DEFAULT_MODEL_PATH
        self.use_ai = use_ai
        self.ai_extractor = None
        self.file_creator = InputMessageCreator()
        
        if self.use_ai:
            self._initialize_ai()
    
    def _initialize_ai(self) -> bool:
        """Initialize AI model for keyword extraction and classification."""
        try:
            model_path_obj = Path(self.model_path)
            if not model_path_obj.exists():
                print(f"Warning: AI model directory not found: {model_path_obj}")
                print("AI features will be disabled. Only basic organization will be available.")
                self.use_ai = False
                return False
            
            print("🧠 Initializing Gemma ONNX model...")
            self.ai_extractor = AIKeywordExtractor(
                model_path=str(model_path_obj)
            )
            print("✅ AI model loaded successfully!")
            return True
            
        except Exception as e:
            print(f"Warning: Failed to initialize AI model: {e}")
            print("AI features will be disabled. Only basic organization will be available.")
            self.use_ai = False
            return False
    
    def extract_file_content(self, file_path: Path) -> Tuple[str, Optional[str], Optional[str]]:
        """
        Extract content from file using fileio_utils package.
        
        Args:
            file_path: Path to the file
            
        Returns:
            Tuple of (text_content, image_path, audio_path)
        """
        try:
            file_ext = file_path.suffix.lower()
            text_content = ""
            image_path = None
            audio_path = None
            
            # Check if file is supported by fileio_utils
            if is_supported_file(str(file_path)):
                try:
                    # Create structured message from file
                    message = create_message_from_file(str(file_path))
                    
                    # Extract content from the structured message
                    if message and len(message) > 0:
                        content_list = message[0].get('content', [])
                        
                        for content_item in content_list:
                            content_type = content_item.get('type')
                            
                            if content_type == MessageTypes.TEXT:
                                text_part = content_item.get('text', '')
                                if text_part != file_path.name:  # Skip filename
                                    text_content += text_part + "\n"
                            
                            elif content_type == MessageTypes.IMAGE:
                                image_path = content_item.get('path')
                            
                            elif content_type == MessageTypes.AUDIO:
                                audio_path = content_item.get('path')
                    
                except Exception as e:
                    print(f"Warning: Error processing {file_path.name} with fileio_utils: {e}")
                    # Fallback to basic text reading for text files
                    if file_ext in ['.txt', '.md']:
                        try:
                            text_content = file_path.read_text(encoding='utf-8', errors='ignore')
                        except:
                            text_content = ""
            
            # Handle files not supported by fileio_utils
            else:
                if file_ext in SupportedFormats.IMAGES:
                    image_path = str(file_path)
                elif file_ext in SupportedFormats.AUDIO:
                    audio_path = str(file_path)
                elif file_ext in ['.txt', '.md']:
                    try:
                        text_content = file_path.read_text(encoding='utf-8', errors='ignore')
                    except:
                        text_content = ""
            
            return text_content.strip(), image_path, audio_path
            
        except Exception as e:
            print(f"Error extracting content from {file_path.name}: {e}")
            return "", None, None
    
    def get_ai_keywords(self, file_path: Path) -> str:
        """Extract AI-powered keywords from file content."""
        if not self.use_ai or not self.ai_extractor:
            return "AI_비활성화"
        
        try:
            print(f"🧠 AI: Analyzing '{file_path.name}'...")
            
            text_content, image_path, audio_path = self.extract_file_content(file_path)
            
            # Use AI extractor with the extracted content
            keywords = self.ai_extractor.extract_keywords(
                text_content, 
                file_path.name, 
                image_path=image_path, 
                audio_path=audio_path
            )
            
            # Clean up keywords for filename use
            safe_keywords = re.sub(r'[<>:"/\\|?*]', '_', keywords)
            safe_keywords = re.sub(r'_+', '_', safe_keywords).strip('_')
            
            if not safe_keywords or safe_keywords in ["키워드추출실패", "내용없음", "지원하지않는형식"]:
                # Fallback to filename-based keywords
                name_parts = re.findall(r'[가-힣A-Za-z0-9]+', file_path.stem)
                if name_parts:
                    safe_keywords = "_".join(name_parts[:3])
                else:
                    safe_keywords = "내용분석불가"
            
            return safe_keywords
            
        except Exception as e:
            print(f"Error extracting keywords from {file_path.name}: {e}")
            return "키워드추출오류"
    
    def get_ai_filename_suggestion(self, file_path: Path) -> str:
        """Get AI-powered filename suggestion."""
        if not self.use_ai or not self.ai_extractor:
            return file_path.name
        
        try:
            print(f"🤖 AI: Generating filename suggestion for '{file_path.name}'...")
            
            text_content, image_path, audio_path = self.extract_file_content(file_path)
            
            suggested_name = self.ai_extractor.suggest_filename(
                text_content,
                file_path.stem,
                file_path.suffix,
                image_path=image_path,
                audio_path=audio_path
            )
            
            return suggested_name
            
        except Exception as e:
            print(f"Error generating filename suggestion for {file_path.name}: {e}")
            return file_path.name
    
    def get_ai_folder_classification(self, file_path: Path) -> str:
        """Get AI-powered folder classification."""
        if not self.use_ai or not self.ai_extractor:
            return "기타"
        
        try:
            print(f"📁 AI: Classifying folder for '{file_path.name}'...")
            
            text_content, image_path, audio_path = self.extract_file_content(file_path)
            
            folder_name = self.ai_extractor.classify_folder(
                text_content,
                file_path.name,
                image_path=image_path,
                audio_path=audio_path
            )
            
            # Clean folder name
            safe_folder_name = re.sub(r'[<>:"/\\|?*]', '_', folder_name)
            safe_folder_name = re.sub(r'[_\s]+', '_', safe_folder_name).strip('_')
            
            return safe_folder_name if safe_folder_name else "기타"
            
        except Exception as e:
            print(f"Error classifying folder for {file_path.name}: {e}")
            return "기타"
    
    def get_file_category(self, file_ext: str) -> str:
        """Get file category based on extension."""
        file_ext = file_ext.lower()
        for category, extensions in FILE_CATEGORIES.items():
            if category == '기타':
                continue
            if file_ext in extensions:
                return category
        return '기타'
    
    def should_exclude_file(self, file_path: Path) -> bool:
        """Check if file should be excluded from processing."""
        # Check filename
        if file_path.name in EXCLUDE_PATTERNS:
            return True
        
        # Check path parts
        for part in file_path.parts:
            if part in EXCLUDE_PATTERNS:
                return True
        
        return False
    
    def organize_files(self, target_path: Path, options: Dict[str, Any]) -> None:
        """
        Organize files in the target directory.
        
        Args:
            target_path: Directory to organize
            options: Organization options
        """
        print(f"🔍 Scanning '{target_path.resolve()}'...")
        
        # Find files to organize
        files_to_organize = []
        file_iterator = target_path.rglob('*') if options.get('recursive') else target_path.glob('*')
        
        for item in file_iterator:
            if item.is_file() and not self.should_exclude_file(item):
                files_to_organize.append(item)
        
        if not files_to_organize:
            print("No files to organize.")
            return
        
        print(f"Found {len(files_to_organize)} files to process.")
        
        # Create organization plan
        change_plan = []
        for file_path in files_to_organize:
            new_name = file_path.name
            new_dir = file_path.parent
            
            # Generate new filename based on options
            if options.get('ai_filename'):
                new_name = self.get_ai_filename_suggestion(file_path)
            elif options.get('ai_keyword'):
                keywords = self.get_ai_keywords(file_path)
                if keywords and keywords not in ["AI_비활성화", "키워드추출오류", "내용분석불가"]:
                    name_stem = file_path.stem
                    extension = file_path.suffix
                    new_name = f"{name_stem}_{keywords}{extension}"
            else:
                # Basic naming with date
                ext = file_path.suffix.lower()
                date_created = datetime.datetime.fromtimestamp(file_path.stat().st_ctime).strftime('%Y-%m-%d')
                
                if ext in ['.txt', '.md']:
                    new_name = f"{date_created}_{file_path.stem}{ext}"
                elif ext in SupportedFormats.IMAGES:
                    new_name = f"IMG_{date_created}_{file_path.stem}{ext}"
                elif ext in SupportedFormats.AUDIO:
                    new_name = f"AUDIO_{date_created}_{file_path.stem}{ext}"
            
            # Determine target directory
            if options.get('ai_classify'):
                ai_folder = self.get_ai_folder_classification(file_path)
                if ai_folder and ai_folder != "기타":
                    new_dir = target_path / ai_folder
            else:
                # Basic categorization
                category = self.get_file_category(file_path.suffix)
                if category != '기타':
                    new_dir = target_path / category
                else:
                    new_dir = target_path / '기타'
            
            new_path = new_dir / new_name
            
            if file_path != new_path:
                change_plan.append({'from': file_path, 'to': new_path})
        
        # Execute organization plan
        self._execute_organization_plan(change_plan, options)
    
    def _execute_organization_plan(self, change_plan: List[Dict], options: Dict[str, Any]) -> None:
        """Execute the file organization plan."""
        if not change_plan:
            print("✅ All files are already organized. No changes needed.")
            return
        
        print(f"\n✨ Planned changes ({len(change_plan)} files):\n")
        for change in change_plan:
            try:
                from_rel = change['from'].relative_to(Path.cwd())
                to_rel = change['to'].relative_to(Path.cwd())
                print(f"  📝 '{from_rel}' → '{to_rel}'")
            except ValueError:
                print(f"  📝 '{change['from']}' → '{change['to']}'")
        
        if options.get('dry_run'):
            print("\n💧 Dry-run mode: No actual changes made.")
            return
        
        # Confirm execution
        if not options.get('skip_confirm'):
            confirm = input(f"\nProceed with organizing {len(change_plan)} files? (y/n): ")
            if confirm.lower() != 'y':
                print("Operation cancelled.")
                return
        
        # Execute changes
        print("\n🚀 Organizing files...")
        success_count = 0
        log_entries = []
        
        for change in change_plan:
            try:
                old_path = change['from']
                new_path = change['to']
                
                # Create target directory if needed
                new_path.parent.mkdir(parents=True, exist_ok=True)
                
                # Move/rename file
                old_path.rename(new_path)
                
                log_entry = f"[SUCCESS] '{old_path}' → '{new_path}'"
                log_entries.append(log_entry)
                success_count += 1
                
                print(f"  ✅ {old_path.name}")
                
            except Exception as e:
                log_entry = f"[ERROR] '{old_path}' → '{new_path}': {e}"
                log_entries.append(log_entry)
                print(f"  ❌ {old_path.name}: {e}")
        
        # Save log
        if options.get('log_file'):
            log_path = Path(options['log_file'])
            with open(log_path, 'a', encoding='utf-8') as f:
                f.write(f"\n=== Smart File Organizer Log - {datetime.datetime.now()} ===\n")
                for entry in log_entries:
                    f.write(entry + '\n')
            print(f"\n📝 Log saved to '{log_path.resolve()}'")
        
        print(f"\n🎉 Organization complete! {success_count}/{len(change_plan)} files processed successfully.")
    
    def scan_directory(self, target_path: Path, options: Dict[str, Any]) -> None:
        """Scan and analyze directory contents."""
        print(f"📊 Analyzing '{target_path.resolve()}'...")
        
        files = []
        file_iterator = target_path.rglob('*') if options.get('recursive') else target_path.glob('*')
        
        for item in file_iterator:
            if item.is_file() and not self.should_exclude_file(item):
                files.append(item)
        
        if not files:
            print("No files found for analysis.")
            return
        
        print(f"\nFound {len(files)} files.")
        
        # Extension analysis
        if options.get('by_ext', True):
            print("\n--- File Extensions Analysis ---")
            ext_count = {}
            for file in files:
                ext = file.suffix.lower() or 'no_extension'
                ext_count[ext] = ext_count.get(ext, 0) + 1
            
            sorted_exts = sorted(ext_count.items(), key=lambda x: x[1], reverse=True)
            for ext, count in sorted_exts:
                category = self.get_file_category(ext) if ext != 'no_extension' else '기타'
                print(f"  {ext:>10} : {count:>3} files ({category})")
        
        # Date analysis
        if options.get('by_date', True):
            print("\n--- Oldest Files (Top 10) ---")
            files_by_date = sorted(files, key=lambda f: f.stat().st_mtime)
            for file in files_by_date[:10]:
                mod_time = datetime.datetime.fromtimestamp(file.stat().st_mtime)
                print(f"  {mod_time.strftime('%Y-%m-%d %H:%M')} : {file.name}")


def main():
    """Main CLI interface."""
    parser = argparse.ArgumentParser(
        prog="smart-file-organizer",
        description="🤖 Smart File Organizer: AI-powered file organization with fileio_utils integration",
        epilog="Use 'smart-file-organizer [COMMAND] --help' for detailed help."
    )
    
    subparsers = parser.add_subparsers(dest="command", required=True, help="Command to execute")
    
    # Organize command
    parser_organize = subparsers.add_parser("organize", help="Organize files intelligently")
    parser_organize.add_argument("target_path", type=Path, help="Directory to organize")
    parser_organize.add_argument("-m", "--model-path", type=str, default=DEFAULT_MODEL_PATH,
                                help=f"AI model directory path (default: {DEFAULT_MODEL_PATH})")
    parser_organize.add_argument("-d", "--dry-run", action="store_true",
                                help="Show planned changes without executing them")
    parser_organize.add_argument("-r", "--recursive", action="store_true",
                                help="Process subdirectories recursively")
    parser_organize.add_argument("-k", "--ai-keyword", action="store_true",
                                help="Use AI to extract keywords for filenames")
    parser_organize.add_argument("-f", "--ai-filename", action="store_true",
                                help="Use AI to suggest complete new filenames")
    parser_organize.add_argument("-c", "--ai-classify", action="store_true",
                                help="Use AI to classify files into folders")
    parser_organize.add_argument("-A", "--ai-all", action="store_true",
                                help="Enable all AI features (-k -f -c)")
    parser_organize.add_argument("-y", "--yes", action="store_true",
                                help="Skip confirmation prompts")
    parser_organize.add_argument("-l", "--log", nargs='?', const=LOG_FILE_NAME, default=None,
                                help=f"Save operation log (default: {LOG_FILE_NAME})")
    parser_organize.add_argument("--no-ai", action="store_true",
                                help="Disable AI features completely")
    
    # Scan command
    parser_scan = subparsers.add_parser("scan", help="Analyze directory contents")
    parser_scan.add_argument("target_path", type=Path, help="Directory to analyze")
    parser_scan.add_argument("-r", "--recursive", action="store_true",
                            help="Include subdirectories")
    parser_scan.add_argument("-e", "--by-ext", action="store_true",
                            help="Show file extension statistics")
    parser_scan.add_argument("-t", "--by-date", action="store_true",
                            help="Show files by modification date")
    
    # Info command
    parser_info = subparsers.add_parser("info", help="Show configuration information")
    parser_info.add_argument("--supported-formats", action="store_true",
                            help="Show supported file formats")
    parser_info.add_argument("--categories", action="store_true",
                            help="Show file categories")
    parser_info.add_argument("--model-path", action="store_true",
                            help="Show current model path")
    
    args = parser.parse_args()
    
    # Handle AI all flag
    if hasattr(args, 'ai_all') and args.ai_all:
        args.ai_keyword = True
        args.ai_filename = True
        args.ai_classify = True
    
    # Execute commands
    if args.command == "organize":
        if not args.target_path.is_dir():
            print(f"Error: '{args.target_path}' is not a valid directory.")
            sys.exit(1)
        
        use_ai = not args.no_ai and (args.ai_keyword or args.ai_filename or args.ai_classify)
        
        organizer = SmartFileOrganizer(model_path=args.model_path, use_ai=use_ai)
        
        options = {
            'recursive': args.recursive,
            'ai_keyword': args.ai_keyword,
            'ai_filename': args.ai_filename,
            'ai_classify': args.ai_classify,
            'dry_run': args.dry_run,
            'skip_confirm': args.yes,
            'log_file': args.log
        }
        
        organizer.organize_files(args.target_path, options)
    
    elif args.command == "scan":
        if not args.target_path.is_dir():
            print(f"Error: '{args.target_path}' is not a valid directory.")
            sys.exit(1)
        
        organizer = SmartFileOrganizer(use_ai=False)  # No AI needed for scanning
        
        # If no specific options, show both
        show_all = not args.by_ext and not args.by_date
        
        options = {
            'recursive': args.recursive,
            'by_ext': args.by_ext or show_all,
            'by_date': args.by_date or show_all
        }
        
        organizer.scan_directory(args.target_path, options)
    
    elif args.command == "info":
        if args.supported_formats:
            print("Supported File Formats (via fileio_utils):")
            creator = InputMessageCreator()
            formats = creator.get_supported_formats()
            for category, extensions in formats.items():
                print(f"  {category}: {', '.join(extensions)}")
        
        elif args.categories:
            print("File Categories:")
            for category, extensions in FILE_CATEGORIES.items():
                if category == '기타':
                    print(f"  {category}: All other files")
                else:
                    print(f"  {category}: {', '.join(extensions)}")
        
        elif args.model_path:
            print(f"Default Model Path: {DEFAULT_MODEL_PATH}")
        
        else:
            print("Smart File Organizer v2.0")
            print("Features:")
            print("  - AI-powered keyword extraction")
            print("  - Smart filename suggestions")
            print("  - Intelligent folder classification")
            print("  - Advanced file content analysis via fileio_utils")
            print("  - Support for multiple file formats")
            print()
            print("Use --help with any command for detailed usage information.")


if __name__ == "__main__":
    main()
