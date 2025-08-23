"""
FastAPI endpoints for the file organizing application.
This module defines all API routes and handles request/response processing.
"""

import logging
from datetime import datetime
from fastapi import APIRouter, HTTPException, status

from models.schema import (
    PingResponse,
    IndexFolderRequest,
    IndexFolderResponse,
    SearchRequest,
    SearchResponse,
    GenerateNameRequest,
    GenerateNameResponse,
)
from core.indexer import process_folder
from core.search import find_relevant_files
from core.renamer import generate_new_filename

# Set up logging
logger = logging.getLogger(__name__)

# Create API router
router = APIRouter()


@router.get("/ping", response_model=PingResponse)
async def ping():
    """
    Health check endpoint to verify the service is running.

    Returns:
        PingResponse: Service status and timestamp
    """
    logger.info("Ping endpoint called")

    return PingResponse(
        status="healthy",
        message="File Fairy backend is running",
        timestamp=datetime.now().isoformat(),
    )


@router.post("/index-folder", response_model=IndexFolderResponse)
async def index_folder(request: IndexFolderRequest):
    """
    Index all files in a specified folder for semantic search.

    This endpoint processes all supported files in a folder, extracts their
    text content, and creates searchable embeddings. The indexing process
    can be configured to include/exclude certain file types and can operate
    recursively on subdirectories.

    Args:
        request (IndexFolderRequest): Folder indexing configuration

    Returns:
        IndexFolderResponse: Results of the indexing operation

    Raises:
        HTTPException: If the request is invalid or processing fails
    """
    logger.info(f"Index folder request for: {request.folder_path}")

    try:
        # Validate request
        if not request.folder_path:
            raise HTTPException(
                status_code=status.HTTP_400_BAD_REQUEST,
                detail="Folder path cannot be empty",
            )

        # Process the folder
        result = process_folder(
            folder_path=request.folder_path,
            recursive=request.recursive,
            file_extensions=request.file_extensions,
        )

        # Log the result
        if result.success:
            logger.info(f"Successfully indexed {len(result.indexed_files)} files")
        else:
            logger.warning(f"Indexing completed with issues: {result.message}")

        return result

    except Exception as e:
        logger.error(f"Error processing index request: {str(e)}")
        raise HTTPException(
            status_code=status.HTTP_500_INTERNAL_SERVER_ERROR,
            detail=f"Failed to index folder: {str(e)}",
        )


@router.post("/search", response_model=SearchResponse)
async def search_files(request: SearchRequest):
    """
    Perform semantic search across indexed files.

    This endpoint accepts natural language queries and returns the most
    relevant files based on content similarity. Results are ranked by
    relevance score and can be filtered by similarity threshold.

    Args:
        request (SearchRequest): Search query and parameters

    Returns:
        SearchResponse: Search results with relevance scores

    Raises:
        HTTPException: If the request is invalid or search fails
    """
    logger.info(f"Search request for query: '{request.query}'")

    try:
        # Validate request
        if not request.query.strip():
            raise HTTPException(
                status_code=status.HTTP_400_BAD_REQUEST,
                detail="Search query cannot be empty",
            )

        if request.limit < 1 or request.limit > 100:
            raise HTTPException(
                status_code=status.HTTP_400_BAD_REQUEST,
                detail="Limit must be between 1 and 100",
            )

        # Perform the search
        result = find_relevant_files(
            query=request.query,
            limit=request.limit,
            similarity_threshold=request.similarity_threshold or 0.7,
        )

        # Log the result
        if result.success:
            logger.info(f"Search returned {result.total_results} results")
        else:
            logger.warning(f"Search failed: {result.query}")

        return result

    except Exception as e:
        logger.error(f"Error processing search request: {str(e)}")
        raise HTTPException(
            status_code=status.HTTP_500_INTERNAL_SERVER_ERROR,
            detail=f"Search failed: {str(e)}",
        )


@router.post("/generate-name", response_model=GenerateNameResponse)
async def generate_filename(request: GenerateNameRequest):
    """
    Generate a new filename for a file based on its content using AI.

    This endpoint analyzes file content and uses local LLM to suggest
    a more descriptive and organized filename. The suggestion considers
    file content, context, and naming conventions.

    Args:
        request (GenerateNameRequest): File path and generation parameters

    Returns:
        GenerateNameResponse: Generated filename with confidence and reasoning

    Raises:
        HTTPException: If the request is invalid or generation fails
    """
    logger.info(f"Generate filename request for: {request.file_path}")

    try:
        # Validate request
        if not request.file_path:
            raise HTTPException(
                status_code=status.HTTP_400_BAD_REQUEST,
                detail="File path cannot be empty",
            )

        # Generate the new filename
        result = generate_new_filename(
            file_path=request.file_path,
            context=request.context,
            preserve_extension=request.preserve_extension,
        )

        # Log the result
        if result.success:
            logger.info(f"Generated filename: {result.suggested_filename}")
        else:
            logger.warning(f"Filename generation failed: {result.reasoning}")

        return result

    except Exception as e:
        logger.error(f"Error processing filename generation request: {str(e)}")
        raise HTTPException(
            status_code=status.HTTP_500_INTERNAL_SERVER_ERROR,
            detail=f"Filename generation failed: {str(e)}",
        )


# Additional utility endpoints


@router.get("/status")
async def get_service_status():
    """
    Get detailed service status and statistics.

    Returns:
        dict: Service status information
    """
    from core.indexer import get_index_status

    try:
        index_status = get_index_status()

        return {
            "service": "running",
            "timestamp": datetime.now().isoformat(),
            "index": index_status,
            "version": "0.1.0",
        }
    except Exception as e:
        logger.error(f"Error getting service status: {str(e)}")
        raise HTTPException(
            status_code=status.HTTP_500_INTERNAL_SERVER_ERROR,
            detail="Failed to get service status",
        )


@router.delete("/index")
async def clear_search_index():
    """
    Clear the entire search index.

    This endpoint removes all indexed files and embeddings from the database.
    Use with caution as this operation cannot be undone.

    Returns:
        dict: Operation result
    """
    logger.warning("Index clearing requested")

    try:
        from core.indexer import clear_index

        success = clear_index()

        if success:
            return {"success": True, "message": "Search index cleared successfully"}
        else:
            raise HTTPException(
                status_code=status.HTTP_500_INTERNAL_SERVER_ERROR,
                detail="Failed to clear search index",
            )
    except Exception as e:
        logger.error(f"Error clearing index: {str(e)}")
        raise HTTPException(
            status_code=status.HTTP_500_INTERNAL_SERVER_ERROR,
            detail=f"Failed to clear index: {str(e)}",
        )
