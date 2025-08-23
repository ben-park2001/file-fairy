"""
Pydantic models for API request and response validation.
Ensures type safety and provides automatic API documentation.
"""

from typing import List, Optional
from pydantic import BaseModel, Field


# Request Models
class IndexFolderRequest(BaseModel):
    """Request model for indexing a folder."""

    folder_path: str = Field(..., description="Absolute path to the folder to index")
    recursive: bool = Field(
        True, description="Whether to index subdirectories recursively"
    )
    file_extensions: Optional[List[str]] = Field(
        None,
        description="List of file extensions to include (e.g., ['.txt', '.pdf']). If None, all files are included.",
    )


class SearchRequest(BaseModel):
    """Request model for semantic search."""

    query: str = Field(..., min_length=1, description="Search query string")
    limit: int = Field(
        10, ge=1, le=100, description="Maximum number of results to return"
    )
    similarity_threshold: Optional[float] = Field(
        0.7, ge=0.0, le=1.0, description="Minimum similarity score for results"
    )


class GenerateNameRequest(BaseModel):
    """Request model for filename generation."""

    file_path: str = Field(..., description="Absolute path to the file to rename")
    context: Optional[str] = Field(
        None, description="Additional context for filename generation"
    )
    preserve_extension: bool = Field(
        True, description="Whether to preserve the original file extension"
    )


# Response Models
class PingResponse(BaseModel):
    """Response model for ping endpoint."""

    status: str = Field(..., description="Service status")
    message: str = Field(..., description="Status message")
    timestamp: str = Field(..., description="Current timestamp")


class FileInfo(BaseModel):
    """Model representing a file's information."""

    file_path: str = Field(..., description="Absolute path to the file")
    file_name: str = Field(..., description="Name of the file")
    file_size: int = Field(..., description="Size of the file in bytes")
    file_type: str = Field(..., description="File extension/type")
    last_modified: str = Field(..., description="Last modification timestamp")


class IndexFolderResponse(BaseModel):
    """Response model for folder indexing."""

    success: bool = Field(..., description="Whether the indexing was successful")
    message: str = Field(..., description="Status message")
    indexed_files: List[FileInfo] = Field(
        ..., description="List of successfully indexed files"
    )
    failed_files: List[str] = Field(
        ..., description="List of files that failed to index"
    )
    total_files: int = Field(..., description="Total number of files processed")


class SearchResult(BaseModel):
    """Model representing a search result."""

    file_path: str = Field(..., description="Absolute path to the file")
    file_name: str = Field(..., description="Name of the file")
    similarity_score: float = Field(..., description="Similarity score (0.0 to 1.0)")
    excerpt: Optional[str] = Field(None, description="Relevant excerpt from the file")


class SearchResponse(BaseModel):
    """Response model for search results."""

    success: bool = Field(..., description="Whether the search was successful")
    query: str = Field(..., description="Original search query")
    results: List[SearchResult] = Field(..., description="List of search results")
    total_results: int = Field(..., description="Total number of results found")


class GenerateNameResponse(BaseModel):
    """Response model for filename generation."""

    success: bool = Field(..., description="Whether the generation was successful")
    original_filename: str = Field(..., description="Original filename")
    suggested_filename: str = Field(..., description="AI-generated filename suggestion")
    confidence_score: float = Field(
        ..., description="Confidence score of the suggestion (0.0 to 1.0)"
    )
    reasoning: Optional[str] = Field(
        None, description="Explanation of the naming decision"
    )


class ExtractTextRequest(BaseModel):
    """Request model for text extraction."""

    file_path: str = Field(..., description="Absolute path to the file")
    clean: bool = Field(True, description="Whether to clean the extracted content")


class ExtractTextResponse(BaseModel):
    """Response model for text extraction."""

    file_name: str = Field(..., description="Name of the file")
    file_type: str = Field(..., description="File extension/type")
    content: str = Field(..., description="Extracted text content")
    error: Optional[str] = Field(None, description="Error message if extraction failed")


# Error Models
class ErrorResponse(BaseModel):
    """Standard error response model."""

    success: bool = Field(False, description="Indicates the request failed")
    error: str = Field(..., description="Error type")
    message: str = Field(..., description="Detailed error message")
    details: Optional[dict] = Field(None, description="Additional error details")
