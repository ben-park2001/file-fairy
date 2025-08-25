import os
import logging
import re
from datetime import datetime
import ollama

# Set up logging
logger = logging.getLogger(__name__)

# Configuration
LLM = "qwen3:4b-instruct-2507-q4_K_M"


def generate_ai_filename(
    key_chunks: list[str],
    original_filename: str,
    file_extension: str,
) -> str:
    """
    Generate a filename using local LLM based on key content chunks.

    Args:
        key_chunks: List of key text chunks extracted from the document
        original_filename: Original filename for reference
        file_extension: File extension to preserve

    Returns:
        str: AI-generated filename
    """
    try:
        # Prepare the content summary from key chunks
        # Limit content to avoid token limits
        content_summary = " ".join(key_chunks)[:1000]

        # Create a focused prompt for filename generation
        system_prompt = """You are a helpful assistant that generates concise, descriptive filenames based on document content. 
Generate a filename that:
- Is descriptive and meaningful
- Uses underscores instead of spaces
- Is professional and organized
- Does not include special characters except underscores and hyphens
- Is between 10-50 characters (excluding extension)
- Captures the main topic/purpose of the document

Respond with ONLY the filename base (without extension), nothing else."""

        user_prompt = f"""Based on this document content, generate a descriptive filename:

Content summary: {content_summary}

Original filename: {original_filename}

Generate filename (without extension):"""

        # Generate response using the LLM
        response = ollama.chat(
            model=LLM,
            messages=[
                {"role": "system", "content": system_prompt},
                {"role": "user", "content": user_prompt},
            ],
            stream=False,
            options={
                "max_tokens": 50,
                "temperature": 0.3,
                "top_k": 40,
                "top_p": 0.95,
                "repeat_penalty": 1.1,
                "stop": ["\n", "."],
            },
        )

        # Extract the generated filename
        generated_name = response.message.content.strip()

        # Clean and validate the generated name
        suggested_name = _sanitize_filename(generated_name)
        logger.info(f"LLM suggested filename: {suggested_name}.{file_extension}")
        return f"{suggested_name}.{file_extension}"

    except Exception as e:
        logger.error(f"LLM filename generation failed: {str(e)}")
        # Simple fallback if LLM fails
        base_name = "document"
        current_date = datetime.now().strftime("%Y%m%d")
        return f"{base_name}_{current_date}{file_extension}"


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
