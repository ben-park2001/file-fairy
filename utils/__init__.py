"""
Utility functions package for File Fairy backend.

This package contains helper functions and utilities used throughout
the application, including file parsing and text extraction.
"""

from .file_parser import extract_text, get_supported_file_types, is_file_supported

__all__ = ["extract_text", "get_supported_file_types", "is_file_supported"]
