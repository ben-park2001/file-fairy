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
    
    # Response models
    PingResponse,
    IndexFolderResponse,
    SearchResponse,
    GenerateNameResponse,
    ErrorResponse,
    
    # Data models
    FileInfo,
    SearchResult
)

__all__ = [
    # Request models
    "IndexFolderRequest",
    "SearchRequest", 
    "GenerateNameRequest",
    
    # Response models
    "PingResponse",
    "IndexFolderResponse",
    "SearchResponse",
    "GenerateNameResponse",
    "ErrorResponse",
    
    # Data models
    "FileInfo",
    "SearchResult"
]