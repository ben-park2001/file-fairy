#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
File Fairy CLI - AI-powered file organization tool
"""

import sys
import argparse
from pathlib import Path

# Import file_fairy modules
try:
    from .core.organizer import FileOrganizer
    from .core.config import AppConfig, FileCategory
except ImportError as e:
    print(f"Error: Failed to import file_fairy package: {e}")
    print("Please ensure file_fairy is available in your Python path.")
    sys.exit(1)


class FileFairyCLI:
    """Command-line interface for File Fairy."""
    
    def __init__(self):
        """Initialize the CLI."""
        self.organizer = None
    
    def _initialize_organizer(self, use_ai: bool = True, model_path: str = None) -> FileOrganizer:
        """Initialize the file organizer with given parameters."""
        if not self.organizer:
            config = AppConfig.load()
            self.organizer = FileOrganizer(use_ai=use_ai, model_path=model_path, config=config)
        return self.organizer
    
    def organize_files(self, args) -> None:
        """
        Organize files in the target directory.
        
        Args:
            args: Parsed command line arguments
        """
        if not args.target_path.is_dir():
            print(f"Error: '{args.target_path}' is not a valid directory.")
            sys.exit(1)
        
        # Determine if AI should be used
        use_ai = not args.no_ai
        
        # Initialize organizer
        organizer = self._initialize_organizer(use_ai=use_ai, model_path=args.model_path)
        
        try:
            if args.preview:
                # Show preview of organization
                print(f"🔍 Preview: How files in '{args.target_path}' would be organized:")
                preview = organizer.get_organization_preview(args.target_path)
                
                if not preview:
                    print("No files found to organize.")
                    return
                
                print(f"\nShowing preview for first {len(preview)} files:\n")
                for item in preview:
                    print(f"  📄 {item['original_name']} → {item['new_name']} ({item['category']})")
                
                print(f"\n💡 Use 'file-fairy organize {args.target_path}' to apply changes.")
            
            else:
                # Actually organize files
                target_dir = args.output or args.target_path / "organized"
                
                print(f"🚀 Organizing files from '{args.target_path}' to '{target_dir}'")
                
                results = organizer.organize_directory(
                    source_dir=args.target_path,
                    target_dir=target_dir,
                    dry_run=args.dry_run
                )
                
                # Display results
                print(f"\n📊 Organization Results:")
                print(f"  Total files: {results['total_files']}")
                print(f"  Processed: {results['processed_files']}")
                print(f"  Skipped: {results['skipped_files']}")
                print(f"  Errors: {len(results['errors'])}")
                
                if results['errors']:
                    print("\n❌ Errors encountered:")
                    for error in results['errors']:
                        print(f"  • {error}")
                
                if args.dry_run:
                    print("\n💧 This was a dry run. No files were actually moved.")
                
        except Exception as e:
            print(f"Error during organization: {e}")
            sys.exit(1)
    
    def scan_directory(self, args) -> None:
        """
        Scan and analyze directory contents.
        
        Args:
            args: Parsed command line arguments
        """
        if not args.target_path.is_dir():
            print(f"Error: '{args.target_path}' is not a valid directory.")
            sys.exit(1)
        
        print(f"📊 Analyzing '{args.target_path}'...")
        
        from .core.file_utils import FileUtils
        
        # Get files
        files = FileUtils.get_files_in_directory(args.target_path, recursive=args.recursive)
        
        if not files:
            print("No files found for analysis.")
            return
        
        print(f"\nFound {len(files)} files.")
        
        # Extension analysis
        if args.by_ext or not (args.by_ext or args.by_date):
            print("\n--- File Extensions Analysis ---")
            ext_count = {}
            for file in files:
                ext = file.suffix.lower() or 'no_extension'
                ext_count[ext] = ext_count.get(ext, 0) + 1
            
            sorted_exts = sorted(ext_count.items(), key=lambda x: x[1], reverse=True)
            for ext, count in sorted_exts:
                category = FileCategory.get_category_for_extension(ext) if ext != 'no_extension' else '기타'
                print(f"  {ext:>10} : {count:>3} files ({category})")
        
        # Date analysis
        if args.by_date or not (args.by_ext or args.by_date):
            import datetime
            print("\n--- Oldest Files (Top 10) ---")
            files_by_date = sorted(files, key=lambda f: f.stat().st_mtime)
            for file in files_by_date[:10]:
                mod_time = datetime.datetime.fromtimestamp(file.stat().st_mtime)
                print(f"  {mod_time.strftime('%Y-%m-%d %H:%M')} : {file.name}")
    
    def show_info(self, args) -> None:
        """
        Show configuration and system information.
        
        Args:
            args: Parsed command line arguments
        """
        if args.supported_formats or not (args.supported_formats or args.categories or args.model_path):
            print("📁 Supported File Formats:")
            print(f"  Documents: {', '.join(FileCategory.DOCUMENTS.extensions)}")
            print(f"  Images: {', '.join(FileCategory.IMAGES.extensions)}")
            print(f"  Data: {', '.join(FileCategory.DATA.extensions)}")
            print(f"  Audio: {', '.join(FileCategory.AUDIO.extensions)}")
            print(f"  Video: {', '.join(FileCategory.VIDEO.extensions)}")
            print(f"  Archives: {', '.join(FileCategory.ARCHIVES.extensions)}")
            print(f"  Code: {', '.join(FileCategory.CODE.extensions)}")
        
        if args.categories or not (args.supported_formats or args.categories or args.model_path):
            print("\n🗂️  File Categories:")
            categories = FileCategory.get_categories_dict()
            for category, extensions in categories.items():
                if isinstance(extensions, list):
                    ext_str = ', '.join(extensions[:5])  # Show first 5 extensions
                    if len(extensions) > 5:
                        ext_str += f" ... ({len(extensions)} total)"
                    print(f"  {category}: {ext_str}")
                else:
                    print(f"  {category}: {extensions}")
        
        if args.model_path or not (args.supported_formats or args.categories or args.model_path):
            config = AppConfig.load()
            print(f"\n🤖 Default AI Model Path: {config.ai.model_path}")
            model_path = Path(config.ai.model_path)
            if model_path.exists():
                print("  ✅ Model directory found")
            else:
                print("  ❌ Model directory not found (AI features will be disabled)")


def create_parser() -> argparse.ArgumentParser:
    """Create the command line argument parser."""
    config = AppConfig.load()
    
    parser = argparse.ArgumentParser(
        prog="file-fairy",
        description="🧚 File Fairy: AI-powered file organization tool",
        epilog="Use 'file-fairy [COMMAND] --help' for detailed help."
    )
    
    subparsers = parser.add_subparsers(dest="command", required=True, help="Command to execute")
    
    # Organize command
    parser_organize = subparsers.add_parser("organize", help="Organize files intelligently")
    parser_organize.add_argument("target_path", type=Path, help="Directory to organize")
    parser_organize.add_argument("-o", "--output", type=Path,
                                help="Output directory for organized files (default: target_path/organized)")
    parser_organize.add_argument("-m", "--model-path", type=str, default=config.ai.model_path,
                                help=f"AI model directory path (default: {config.ai.model_path})")
    parser_organize.add_argument("-d", "--dry-run", action="store_true",
                                help="Show what would be done without executing")
    parser_organize.add_argument("-p", "--preview", action="store_true",
                                help="Show preview of how files would be organized")
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
    
    return parser


def main():
    """Main CLI entry point."""
    parser = create_parser()
    args = parser.parse_args()
    
    cli = FileFairyCLI()
    
    try:
        if args.command == "organize":
            cli.organize_files(args)
        elif args.command == "scan":
            cli.scan_directory(args)
        elif args.command == "info":
            cli.show_info(args)
        else:
            parser.print_help()
            sys.exit(1)
    
    except KeyboardInterrupt:
        print("\n\n👋 Operation cancelled by user.")
        sys.exit(1)
    except Exception as e:
        print(f"❌ Unexpected error: {e}")
        sys.exit(1)


if __name__ == "__main__":
    main()
