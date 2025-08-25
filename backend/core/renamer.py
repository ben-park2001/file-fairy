"""
AI-powered file renaming functionality.
This module uses local LLM to generate meaningful filenames based on file content
and context, helping users organize their files more effectively.
"""

import os
import logging
import re
from pathlib import Path
from typing import Optional, Tuple
from datetime import datetime

from ..models.schema import GenerateNameResponse, ExtractTextRequest
from ..utils.file_parser import extract_text
from ..utils.embedding import embed_text, extract_key_chunks, chunk_text
from ..utils.llm import generate_ai_filename

# Set up logging
logger = logging.getLogger(__name__)


def generate_new_filename(
    file_path: str,
    context: Optional[str] = None,
    preserve_extension: bool = True,
) -> GenerateNameResponse:
    """
    Generate a new meaningful filename based on file content using AI.

    This function analyzes the file content and uses a local LLM to generate
    a descriptive, organized filename. In the full implementation, this would:
    1. Extract and analyze file content
    2. Use llama-cpp-python to run local LLM inference
    3. Generate contextually appropriate filename suggestions
    4. Apply naming conventions and sanitization
    5. Return suggestion

    Args:
        file_path (str): Absolute path to the file to rename
        context (Optional[str]): Additional context for filename generation
        preserve_extension (bool): Whether to keep the original file extension

    Returns:
        GenerateNameResponse: Generated filename with metadata
    """
    logger.info(f"Generating new filename for: {file_path}")

    if not os.path.exists(file_path):
        return GenerateNameResponse(
            success=False,
            original_filename="",
            suggested_filename="",
            reasoning="File not found",
        )

    try:
        # Get original filename
        original_path = Path(file_path)
        original_filename = original_path.name
        file_extension = original_path.suffix if preserve_extension else ""

        # Extract file content for analysis
        try:
            request = ExtractTextRequest(file_path=file_path, clean=True)
            response = extract_text(request)
            file_content = (
                response.content
                if response.success
                else f"File type: {original_path.suffix}"
            )
        except Exception as e:
            logger.warning(f"Could not extract content from {file_path}: {str(e)}")
            file_content = f"File type: {original_path.suffix}"

        # AI-powered filename generation workflow
        try:
            # Step 1: Breaking down the file content into manageable chunks
            chunks = chunk_text(file_content, chunk_size=400, overlap=50)
            logger.info(f"Split content into {len(chunks)} chunks")

            # Step 2: Creating embeddings for the chunks
            if chunks:
                chunk_embeddings = embed_text(chunks)
                logger.info(f"Generated embeddings for {len(chunk_embeddings)} chunks")

                # Step 3: Extracting key chunks from the embeddings
                key_chunks = extract_key_chunks(
                    chunk_embeddings,
                    chunks,
                    # Don't cluster more than available chunks
                    n_clusters=min(4, len(chunks)),
                )
                logger.info(f"Extracted {len(key_chunks)} key chunks")

                # Step 4: Generating a new filename based on the key chunks
                suggested_filename = generate_ai_filename(
                    key_chunks, original_filename, context, file_extension
                )
                reasoning = "Generated using AI analysis of file content with embeddings and LLM"
            else:
                # Fallback if no content could be chunked
                suggested_filename = generate_ai_filename(
                    ["No content available"],
                    original_filename,
                    context,
                    file_extension,
                )
                reasoning = "Generated using AI with minimal content (chunking failed)"

        except Exception as e:
            logger.warning(
                f"AI filename generation failed, using basic fallback: {str(e)}"
            )
            # Simple fallback if everything fails
            current_date = datetime.now().strftime("%Y%m%d")
            base_name = "document"
            suggested_filename = f"{base_name}_{current_date}{file_extension}"
            reasoning = f"Generated using basic fallback (AI failed: {str(e)})"

        return GenerateNameResponse(
            success=True,
            original_filename=original_filename,
            suggested_filename=suggested_filename,
            reasoning=reasoning,
        )

    except Exception as e:
        logger.error(f"Failed to generate filename for {file_path}: {str(e)}")
        return GenerateNameResponse(
            success=False,
            original_filename=Path(file_path).name,
            suggested_filename="",
            reasoning=f"Error during generation: {str(e)}",
        )


def batch_rename_files(
    file_paths: list[str], context: Optional[str] = None, dry_run: bool = True
) -> dict:
    """
    Generate new filenames for multiple files in batch.

    This function processes multiple files at once for efficient renaming
    operations, useful for organizing entire folders.

    Args:
        file_paths (list[str]): List of file paths to rename
        context (Optional[str]): Common context for all files
        dry_run (bool): If True, only generate suggestions without renaming

    Returns:
        dict: Results of batch renaming operation
    """
    logger.info(f"Starting batch rename for {len(file_paths)} files")

    results = {"successful": [], "failed": [], "suggestions": []}

    for file_path in file_paths:
        try:
            suggestion = generate_new_filename(
                file_path, context, preserve_extension=True
            )
            results["suggestions"].append(
                {"original_path": file_path, "suggestion": suggestion}
            )

            if suggestion.success:
                results["successful"].append(file_path)
            else:
                results["failed"].append(file_path)

        except Exception as e:
            logger.error(f"Failed to process {file_path}: {str(e)}")
            results["failed"].append(file_path)

    return results


def validate_filename(filename: str) -> Tuple[bool, str]:
    """
    Validate if a filename is acceptable and provide feedback.

    Args:
        filename (str): Filename to validate

    Returns:
        Tuple[bool, str]: (is_valid, feedback_message)
    """
    if not filename:
        return False, "Filename cannot be empty"

    if len(filename) > 255:
        return False, "Filename is too long (max 255 characters)"

    # Check for invalid characters
    invalid_chars = r'[<>:"/\\|?*]'
    if re.search(invalid_chars, filename):
        return False, "Filename contains invalid characters"

    # Check for reserved names (Windows)
    reserved_names = (
        ["CON", "PRN", "AUX", "NUL"]
        + [f"COM{i}" for i in range(1, 10)]
        + [f"LPT{i}" for i in range(1, 10)]
    )
    name_without_ext = os.path.splitext(filename)[0].upper()
    if name_without_ext in reserved_names:
        return False, "Filename uses a reserved system name"

    return True, "Filename is valid"
