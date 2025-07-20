"""Utility functions for File Fairy."""

import re
from pathlib import Path
from typing import List, Optional


def sanitize_filename(filename: str) -> str:
    """
    Sanitize a filename by removing or replacing unsafe characters.
    
    Args:
        filename: Original filename
        
    Returns:
        Sanitized filename
    """
    # Replace unsafe characters with underscores
    unsafe_chars = '<>:"/\\|?*'
    sanitized = filename
    
    for char in unsafe_chars:
        sanitized = sanitized.replace(char, '_')
    
    # Remove leading/trailing dots and spaces
    sanitized = sanitized.strip('. ')
    
    # Replace multiple underscores with single underscore
    sanitized = re.sub(r'_+', '_', sanitized)
    
    # Ensure it's not empty
    if not sanitized:
        sanitized = "unnamed_file"
    
    return sanitized


def get_safe_path(path: Path) -> Path:
    """
    Get a safe path by resolving and normalizing it.
    
    Args:
        path: Original path
        
    Returns:
        Safe normalized path
    """
    try:
        return path.resolve()
    except (OSError, ValueError):
        return Path(str(path))


def extract_text_from_filename(filename: str) -> List[str]:
    """
    Extract meaningful text parts from a filename.
    
    Args:
        filename: Filename to process
        
    Returns:
        List of extracted text parts
    """
    # Remove extension
    stem = Path(filename).stem
    
    # Extract words (Korean, English, numbers)
    words = re.findall(r'[가-힣A-Za-z0-9]+', stem)
    
    return words


def format_file_size(size_bytes: int) -> str:
    """
    Format file size in human-readable format.
    
    Args:
        size_bytes: Size in bytes
        
    Returns:
        Formatted size string
    """
    if size_bytes == 0:
        return "0 B"
    
    size_names = ["B", "KB", "MB", "GB", "TB"]
    i = 0
    
    while size_bytes >= 1024.0 and i < len(size_names) - 1:
        size_bytes /= 1024.0
        i += 1
    
    return f"{size_bytes:.1f} {size_names[i]}"


def truncate_text(text: str, max_length: int = 100) -> str:
    """
    Truncate text to a maximum length.
    
    Args:
        text: Text to truncate
        max_length: Maximum length
        
    Returns:
        Truncated text
    """
    if len(text) <= max_length:
        return text
    
    return text[:max_length - 3] + "..."


def is_text_file(file_path: Path) -> bool:
    """
    Check if a file is likely a text file based on its extension.
    
    Args:
        file_path: Path to the file
        
    Returns:
        True if likely a text file, False otherwise
    """
    text_extensions = {
        '.txt', '.md', '.rst', '.log', '.csv', '.json', '.xml', '.html', 
        '.css', '.js', '.py', '.java', '.cpp', '.c', '.h', '.yaml', '.yml'
    }
    
    return file_path.suffix.lower() in text_extensions
