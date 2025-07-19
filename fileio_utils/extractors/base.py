"""Base extractor interface and common utilities."""

from abc import ABC, abstractmethod
from pathlib import Path
from typing import Optional


class BaseExtractor(ABC):
    """Abstract base class for file content extractors."""
    
    @abstractmethod
    def extract(self, file_path: Path) -> str:
        """
        Extract text content from a file.
        
        Args:
            file_path: Path to the file to extract content from
            
        Returns:
            Extracted text content
            
        Raises:
            FileNotFoundError: If the file doesn't exist
            ValueError: If the file format is not supported
        """
        pass
    
    @staticmethod
    def _read_text_file(file_path: Path, encoding: str = 'utf-8') -> str:
        """Read a plain text file with error handling."""
        try:
            with open(file_path, 'r', encoding=encoding) as file:
                return file.read()
        except UnicodeDecodeError:
            # Fallback to different encodings
            for fallback_encoding in ['cp949', 'euc-kr', 'latin-1']:
                try:
                    with open(file_path, 'r', encoding=fallback_encoding) as file:
                        return file.read()
                except UnicodeDecodeError:
                    continue
            raise ValueError(f"Unable to decode file with any supported encoding: {file_path}")
    
    @staticmethod
    def _handle_import_error(library_name: str, install_command: str) -> str:
        """Generate a standardized import error message."""
        return f"{library_name} reading requires additional dependencies. Please install with: {install_command}"
    
    @staticmethod
    def _handle_extraction_error(file_path: Path, error: Exception) -> str:
        """Generate a standardized extraction error message."""
        return f"Error extracting content from {file_path.name}: {str(error)}"
