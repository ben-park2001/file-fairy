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

from models.schema import GenerateNameResponse, ExtractTextRequest
from utils.file_parser import extract_text

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

        # TODO: Use local LLM for filename generation
        # This would involve:
        # 1. Preparing a prompt with file content and context
        # 2. Running llama-cpp-python inference
        # 3. Parsing the LLM response for filename suggestions

        # Placeholder implementation with rule-based naming
        suggested_name = _generate_placeholder_filename(
            original_filename,
            file_content,
            context,
            file_extension,
        )

        # Sanitize the filename
        sanitized_name = _sanitize_filename(suggested_name)

        return GenerateNameResponse(
            success=True,
            original_filename=original_filename,
            suggested_filename=sanitized_name,
            reasoning="Generated based on file content analysis and naming conventions",
        )

    except Exception as e:
        logger.error(f"Failed to generate filename for {file_path}: {str(e)}")
        return GenerateNameResponse(
            success=False,
            original_filename=Path(file_path).name,
            suggested_filename="",
            reasoning=f"Error during generation: {str(e)}",
        )


def _generate_placeholder_filename(
    original_filename: str,
    file_content: str,
    context: Optional[str],
    file_extension: str,
) -> str:
    """
    Generate a filename using rule-based logic as a placeholder for LLM.

    This is a simplified implementation that will be replaced by LLM-based
    generation in the full version.

    Args:
        original_filename (str): Original filename
        file_content (str): Extracted file content
        context (Optional[str]): Additional context
        file_extension (str): File extension to preserve

    Returns:
        str: Generated filename
    """
    # Extract key information from content
    content_lower = file_content.lower()

    # Try to identify document type and subject
    if "meeting" in content_lower or "agenda" in content_lower:
        base_name = "meeting_notes"
    elif "report" in content_lower:
        base_name = "report"
    elif "invoice" in content_lower or "receipt" in content_lower:
        base_name = "invoice"
    elif "contract" in content_lower or "agreement" in content_lower:
        base_name = "contract"
    elif "presentation" in content_lower or "slides" in content_lower:
        base_name = "presentation"
    elif "proposal" in content_lower:
        base_name = "proposal"
    elif "readme" in original_filename.lower():
        base_name = "readme"
    elif file_extension in [".py", ".js", ".html", ".css"]:
        base_name = "code_file"
    elif file_extension in [".jpg", ".png", ".gif"]:
        base_name = "image"
    else:
        base_name = "document"

    # Add date if relevant
    current_date = datetime.now().strftime("%Y%m%d")

    # Combine with context if provided
    if context:
        context_clean = re.sub(r"[^\w\s-]", "", context.lower()).replace(" ", "_")
        suggested_name = f"{base_name}_{context_clean}_{current_date}{file_extension}"
    else:
        suggested_name = f"{base_name}_{current_date}{file_extension}"

    return suggested_name


def _sanitize_filename(filename: str) -> str:
    """
    Sanitize a filename to ensure it's valid across different operating systems.

    Args:
        filename (str): Raw filename to sanitize

    Returns:
        str: Sanitized filename
    """
    # Remove or replace invalid characters
    invalid_chars = r'[<>:"/\\|?*]'
    sanitized = re.sub(invalid_chars, "_", filename)

    # Remove multiple consecutive underscores
    sanitized = re.sub(r"_+", "_", sanitized)

    # Remove leading/trailing dots and spaces
    sanitized = sanitized.strip(". ")

    # Limit length to reasonable maximum
    if len(sanitized) > 200:
        name_part, ext = os.path.splitext(sanitized)
        sanitized = name_part[: 200 - len(ext)] + ext

    # Ensure it's not empty
    if not sanitized:
        sanitized = "unnamed_file"

    return sanitized


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


def get_naming_suggestions(file_type: str, content_keywords: list[str]) -> list[str]:
    """
    Get naming pattern suggestions based on file type and content.

    Args:
        file_type (str): File extension or type
        content_keywords (list[str]): Keywords extracted from content

    Returns:
        list[str]: List of naming pattern suggestions
    """
    # TODO: Implement intelligent naming pattern suggestions
    # This could be based on:
    # - File type conventions
    # - Content analysis
    # - User's historical naming patterns
    # - Industry standards

    patterns = [
        f"{file_type}_document_YYYYMMDD",
        f"project_{file_type}_v1",
        f"draft_{file_type}_notes",
    ]

    return patterns
