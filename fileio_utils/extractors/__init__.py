"""Extractor registry and factory."""

from typing import Dict, Type

from .base import BaseExtractor
from .text_extractor import PlainTextExtractor
from .pdf_extractor import PDFExtractor
from .office_extractors import DOCXExtractor, PPTXExtractor
from .hwp_extractor import HWPExtractor
from .data_extractors import CSVExtractor, JSONExtractor, ExcelExtractor


class ExtractorRegistry:
    """Registry for file extractors."""
    
    _extractors: Dict[str, Type[BaseExtractor]] = {
        # Text files
        '.txt': PlainTextExtractor,
        '.md': PlainTextExtractor,
        
        # PDF files
        '.pdf': PDFExtractor,
        
        # Office files
        '.docx': DOCXExtractor,
        '.pptx': PPTXExtractor,
        
        # HWP files
        '.hwp': HWPExtractor,
        
        # Data files
        '.csv': CSVExtractor,
        '.json': JSONExtractor,
        '.xlsx': ExcelExtractor,
    }
    
    @classmethod
    def get_extractor(cls, file_extension: str) -> BaseExtractor:
        """
        Get an extractor instance for the given file extension.
        
        Args:
            file_extension: File extension (with dot, e.g., '.pdf')
            
        Returns:
            Extractor instance
            
        Raises:
            ValueError: If no extractor is available for the file type
        """
        extension = file_extension.lower()
        extractor_class = cls._extractors.get(extension)
        
        if extractor_class is None:
            raise ValueError(f"No extractor available for file type: {extension}")
        
        return extractor_class()
    
    @classmethod
    def is_extractable(cls, file_extension: str) -> bool:
        """Check if a file extension can be extracted."""
        return file_extension.lower() in cls._extractors
    
    @classmethod
    def get_supported_extensions(cls) -> list[str]:
        """Get list of all supported file extensions."""
        return list(cls._extractors.keys())
