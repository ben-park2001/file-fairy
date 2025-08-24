"""
Utility functions package for File Fairy backend.

This package contains helper functions and utilities used throughout
the application, including file parsing and text extraction.
"""

from .file_parser import extract_text, get_supported_file_types, is_file_supported
from .embedding import (
    initialize_embedding_model,
    get_embedding_model,
    embed_text,
    chunk_text,
    calculate_text_relevance,
    extract_key_chunks,
)
from .llm import initialize_llm_model, generate_ai_filename

__all__ = [
    # File parsing utilities
    "extract_text",
    "get_supported_file_types",
    "is_file_supported",
    # Embedding utilities
    "initialize_embedding_model",
    "get_embedding_model",
    "embed_text",
    "chunk_text",
    "calculate_text_relevance",
    "extract_key_chunks",
    # LLM utilities
    "initialize_llm_model",
    "generate_ai_filename",
]
