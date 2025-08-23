"""
Core business logic package for File Fairy backend.

This package contains the main AI-powered functionality including
file indexing, semantic search, and intelligent file renaming.
"""

from .indexer import process_folder, get_index_status, clear_index
from .search import find_relevant_files, search_by_file_type, get_similar_files
from .renamer import generate_new_filename, batch_rename_files, validate_filename

__all__ = [
    "process_folder",
    "get_index_status", 
    "clear_index",
    "find_relevant_files",
    "search_by_file_type",
    "get_similar_files",
    "generate_new_filename",
    "batch_rename_files",
    "validate_filename"
]