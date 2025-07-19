"""Core message creation functionality."""

from pathlib import Path
from typing import List, Dict, Any

from .constants import SupportedFormats, MessageTypes, Roles
from .extractors import ExtractorRegistry


class InputMessageCreator:
    """
    A utility class that creates structured input messages from various file types.
    Handles text, image, audio, and data files and converts them into a standardized format.
    """
    
    def __init__(self):
        """Initialize the InputMessageCreator."""
        self._extractor_registry = ExtractorRegistry()
    
    def create_message(self, file_path: str) -> List[Dict[str, Any]]:
        """
        Create a structured message from a file path.
        
        Args:
            file_path: Path to the file to process
            
        Returns:
            Structured message in the specified format
            
        Raises:
            FileNotFoundError: If the file doesn't exist
            ValueError: If the file format is not supported
        """
        file_path_obj = Path(file_path)
        
        if not file_path_obj.exists():
            raise FileNotFoundError(f"File not found: {file_path_obj}")
        
        file_extension = file_path_obj.suffix.lower()
        file_name = file_path_obj.name
        
        if not SupportedFormats.is_supported(file_extension):
            raise ValueError(f"Unsupported file format: {file_extension}")
        
        # Create content item based on file type
        content_item = self._create_content_item(file_path_obj, file_extension)
        
        # Create the structured message
        return [
            {
                "role": Roles.USER,
                "content": [
                    {"type": MessageTypes.TEXT, "text": file_name},
                    content_item
                ]
            }
        ]
    
    def _create_content_item(self, file_path: Path, file_extension: str) -> Dict[str, str]:
        """
        Create the appropriate content item based on file type.
        
        Args:
            file_path: Path object of the file
            file_extension: File extension
            
        Returns:
            Content item with type and content
        """
        if file_extension in SupportedFormats.DOCUMENTS:
            text_content = self._extract_text_content(file_path, file_extension)
            return {"type": MessageTypes.TEXT, "text": text_content}
        
        elif file_extension in SupportedFormats.IMAGES:
            return {"type": MessageTypes.IMAGE, "path": str(file_path)}
        
        elif file_extension in SupportedFormats.AUDIO:
            return {"type": MessageTypes.AUDIO, "path": str(file_path)}
        
        elif file_extension in SupportedFormats.DATA:
            text_content = self._extract_text_content(file_path, file_extension)
            return {"type": MessageTypes.TEXT, "text": text_content}
        
        else:
            raise ValueError(f"Unsupported file format: {file_extension}")
    
    def _extract_text_content(self, file_path: Path, file_extension: str) -> str:
        """
        Extract text content from files using appropriate extractors.
        
        Args:
            file_path: Path to the file
            file_extension: File extension
            
        Returns:
            Extracted text content
        """
        try:
            extractor = self._extractor_registry.get_extractor(file_extension)
            return extractor.extract(file_path)
        except Exception as e:
            return f"Error reading file: {str(e)}"
    
    def is_supported_format(self, file_path: str) -> bool:
        """
        Check if a file format is supported.
        
        Args:
            file_path: Path to the file
            
        Returns:
            True if supported, False otherwise
        """
        file_extension = Path(file_path).suffix.lower()
        return SupportedFormats.is_supported(file_extension)
    
    def get_supported_formats(self) -> Dict[str, List[str]]:
        """
        Get all supported file formats.
        
        Returns:
            Dictionary of supported formats by category
        """
        return SupportedFormats.CATEGORIES.copy()
