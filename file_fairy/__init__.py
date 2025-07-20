"""
File Fairy - A utility package for reading and processing various file formats.

This package provides a clean interface for extracting content from different file types
including documents, images, audio files, and data files.
"""

from .message_creator import InputMessageCreator
from .constants import MessageTypes, Roles
from .extractors import ExtractorRegistry
from .core import AppConfig, FileUtils, FileOrganizer

__version__ = "0.1.0"
__all__ = [
    "InputMessageCreator",
    "MessageTypes",
    "Roles",
    "ExtractorRegistry",
    "AppConfig",
    "FileUtils",
    "FileOrganizer",
    "create_message_from_file",
    "is_supported_file",
    "organize_directory"
]


# Convenience functions for quick file processing
def create_message_from_file(file_path: str):
    """
    Convenience function to create a structured message from a file.
    
    Args:
        file_path: Path to the file to process
        
    Returns:
        Structured message in the specified format
    """
    creator = InputMessageCreator()
    return creator.create_message(file_path)


def is_supported_file(file_path: str) -> bool:
    """
    Check if a file format is supported.
    
    Args:
        file_path: Path to the file
        
    Returns:
        True if supported, False otherwise
    """
    creator = InputMessageCreator()
    return creator.is_supported_format(file_path)


def organize_directory(source_dir: str, target_dir: str = None, use_ai: bool = True, **kwargs):
    """
    Convenience function to organize a directory.
    
    Args:
        source_dir: Source directory to organize
        target_dir: Target directory (optional)
        use_ai: Whether to use AI features
        **kwargs: Additional arguments for FileOrganizer
        
    Returns:
        Organization results dictionary
    """
    from pathlib import Path
    
    source_path = Path(source_dir)
    target_path = Path(target_dir) if target_dir else source_path / "organized"
    
    organizer = FileOrganizer(use_ai=use_ai, **kwargs)
    return organizer.organize_directory(source_path, target_path)
