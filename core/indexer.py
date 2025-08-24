"""
File indexing functionality for building searchable document embeddings.
This module handles scanning folders, extracting text content, and creating
vector embeddings for semantic search capabilities.
"""

import os
import logging
from pathlib import Path
from typing import List, Optional
from datetime import datetime

from models.schema import FileInfo, IndexFolderResponse, ExtractTextRequest
from utils.file_parser import extract_text, is_file_supported
from core.database import add_file

# Set up logging
logger = logging.getLogger(__name__)


def process_folder(
    folder_path: str,
    recursive: bool = True,
    file_extensions: Optional[List[str]] = None,
) -> IndexFolderResponse:
    """
    Process a folder and index all supported files for semantic search.

    This function scans the specified folder, extracts text content from supported
    files, and creates vector embeddings for semantic search. In the full implementation,
    this would:
    1. Scan the folder for files
    2. Extract text content using file_parser
    3. Generate embeddings using sentence-transformers
    4. Store embeddings in LanceDB vector database
    5. Update the search index

    Args:
        folder_path (str): Absolute path to the folder to index
        recursive (bool): Whether to scan subdirectories recursively
        file_extensions (Optional[List[str]]): List of file extensions to include.
                                            If None, all supported files are included.

    Returns:
        IndexFolderResponse: Results of the indexing operation

    Raises:
        FileNotFoundError: If the folder doesn't exist
        PermissionError: If the folder can't be accessed
    """
    logger.info(f"Starting folder indexing: {folder_path}")

    if not os.path.exists(folder_path):
        return IndexFolderResponse(
            success=False,
            message=f"Folder not found: {folder_path}",
            indexed_files=[],
            failed_files=[],
            total_files=0,
        )

    if not os.path.isdir(folder_path):
        return IndexFolderResponse(
            success=False,
            message=f"Path is not a directory: {folder_path}",
            indexed_files=[],
            failed_files=[],
            total_files=0,
        )

    indexed_files: List[FileInfo] = []
    failed_files: List[str] = []

    try:
        # Scan for files
        files_to_process = _scan_folder(folder_path, recursive, file_extensions)

        for file_path in files_to_process:
            try:
                # Extract file information
                file_info = _get_file_info(file_path)

                # Extract text content
                request = ExtractTextRequest(file_path=file_path, clean=True)
                response = extract_text(request)

                if not response.success:
                    failed_files.append(file_path)
                    logger.error(
                        f"Failed to extract text from {file_path}: {response.error}"
                    )
                    continue

                text_content = response.content

                # Add to vector database
                success = add_file(file_path, text_content, file_info.file_name)

                if success:
                    indexed_files.append(file_info)
                    logger.debug(f"Successfully indexed: {file_path}")
                else:
                    failed_files.append(file_path)

            except Exception as e:
                failed_files.append(file_path)
                logger.error(f"Failed to index {file_path}: {str(e)}")

        success = len(failed_files) == 0
        message = f"Indexed {len(indexed_files)} files successfully"
        if failed_files:
            message += f", {len(failed_files)} files failed"

        return IndexFolderResponse(
            success=success,
            message=message,
            indexed_files=indexed_files,
            failed_files=failed_files,
            total_files=len(files_to_process),
        )

    except Exception as e:
        logger.error(f"Error during folder indexing: {str(e)}")
        return IndexFolderResponse(
            success=False,
            message=f"Indexing failed: {str(e)}",
            indexed_files=indexed_files,
            failed_files=failed_files,
            total_files=len(indexed_files) + len(failed_files),
        )


def _scan_folder(
    folder_path: str, recursive: bool, file_extensions: Optional[List[str]]
) -> List[str]:
    """
    Scan a folder for files to index.

    Args:
        folder_path (str): Path to scan
        recursive (bool): Whether to scan recursively
        file_extensions (Optional[List[str]]): File extensions to include

    Returns:
        List[str]: List of file paths to process
    """
    files_to_process = []

    if recursive:
        # Use Path.rglob for recursive scanning
        path = Path(folder_path)
        for file_path in path.rglob("*"):
            if file_path.is_file() and _should_include_file(
                str(file_path), file_extensions
            ):
                files_to_process.append(str(file_path))
    else:
        # Use Path.glob for non-recursive scanning
        path = Path(folder_path)
        for file_path in path.glob("*"):
            if file_path.is_file() and _should_include_file(
                str(file_path), file_extensions
            ):
                files_to_process.append(str(file_path))

    return files_to_process


def _should_include_file(file_path: str, file_extensions: Optional[List[str]]) -> bool:
    """
    Determine if a file should be included in indexing.

    Args:
        file_path (str): Path to the file
        file_extensions (Optional[List[str]]): Allowed extensions

    Returns:
        bool: True if the file should be included
    """
    # Skip hidden files and system files
    if Path(file_path).name.startswith("."):
        return False

    # Check if file is supported for text extraction
    if not is_file_supported(file_path):
        return False

    # If specific extensions are requested, check them
    if file_extensions:
        file_ext = Path(file_path).suffix.lower()
        return file_ext in [ext.lower() for ext in file_extensions]

    return True


def _get_file_info(file_path: str) -> FileInfo:
    """
    Extract file information for a given file.

    Args:
        file_path (str): Path to the file

    Returns:
        FileInfo: File information object
    """
    path = Path(file_path)
    stat = path.stat()

    return FileInfo(
        file_path=str(path.absolute()),
        file_name=path.name,
        file_size=stat.st_size,
        file_type=path.suffix.lower(),
        last_modified=datetime.fromtimestamp(stat.st_mtime).isoformat(),
    )


def get_index_status() -> dict:
    """
    Get the current status of the search index.

    Returns:
        dict: Index status information
    """
    from core.database import get_index_stats

    return get_index_stats()


def clear_index() -> bool:
    """
    Clear the entire search index.

    Returns:
        bool: True if successful
    """
    from core.database import clear_index as db_clear_index

    return db_clear_index()
