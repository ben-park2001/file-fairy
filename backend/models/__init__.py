"""
Data models package for File Fairy backend.

This package contains all Pydantic models used for API request/response
validation and data serialization.
"""

from .schema import (
    # Request models
    IndexFolderRequest,
    SearchRequest,
    GenerateNameRequest,
    ExtractTextRequest,
    # Response models
    PingResponse,
    IndexFolderResponse,
    SearchResponse,
    GenerateNameResponse,
    ExtractTextResponse,
    ErrorResponse,
    # Data models
    FileInfo,
    SearchResult,
    # Vector DB models
    FileChunkSchema,
)

__all__ = [
    # Request models
    "IndexFolderRequest",
    "SearchRequest",
    "GenerateNameRequest",
    "ExtractTextRequest",
    # Response models
    "PingResponse",
    "IndexFolderResponse",
    "SearchResponse",
    "GenerateNameResponse",
    "ExtractTextResponse",
    "ErrorResponse",
    # Data models
    "FileInfo",
    "SearchResult",
    # Vector DB models
    "FileChunkSchema",
]
