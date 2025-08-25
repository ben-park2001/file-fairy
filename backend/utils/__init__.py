"""
Utility functions package for File Fairy backend.

This package contains helper functions and utilities used throughout
the application, including file parsing and text extraction.
"""

from .file_parser import extract_text, get_supported_file_types, is_file_supported
from .embedding import (
    embed_text,
    chunk_text,
    calculate_text_relevance,
    extract_key_chunks,
)
from .llm import generate_ai_filename

__all__ = [
    # File parsing utilities
    "extract_text",
    "get_supported_file_types",
    "is_file_supported",
    # Embedding utilities
    "embed_text",
    "chunk_text",
    "calculate_text_relevance",
    "extract_key_chunks",
    # LLM utilities
    "generate_ai_filename",
]
