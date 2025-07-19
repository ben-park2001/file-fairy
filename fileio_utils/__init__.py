"""
FileIO Utils - A utility package for reading and processing various file formats.

This package provides a clean interface for extracting content from different file types
including documents, images, audio files, and data files.
"""

from .message_creator import InputMessageCreator
from .constants import SupportedFormats, MessageTypes, Roles
from .extractors import ExtractorRegistry

__version__ = "0.1.0"
__all__ = [
    "InputMessageCreator",
    "SupportedFormats", 
    "MessageTypes",
    "Roles",
    "ExtractorRegistry"
]

# Convenience function for quick file processing
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
