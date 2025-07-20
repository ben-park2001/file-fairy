"""File utility functions for File Fairy."""

import os
import shutil
from pathlib import Path
from typing import List, Dict, Optional, Tuple

from .config import AppConfig


class FileUtils:
    """Utility class for file operations."""
    
    def __init__(self, config: Optional[AppConfig] = None):
        """Initialize FileUtils with configuration."""
        self.config = config or AppConfig.load()
    
    def get_files_in_directory(self, directory: Path, recursive: bool = True) -> List[Path]:
        """
        Get all files in a directory, optionally recursively.
        
        Args:
            directory: Directory to scan
            recursive: Whether to scan subdirectories
            
        Returns:
            List of file paths
        """
        if not directory.exists() or not directory.is_dir():
            return []
        
        files = []
        pattern = "**/*" if recursive else "*"
        
        for path in directory.glob(pattern):
            if path.is_file() and not self.config.should_exclude(path):
                files.append(path)
        
        return sorted(files)
    
    @staticmethod
    def create_directory_if_not_exists(directory: Path) -> None:
        """
        Create a directory if it doesn't exist.
        
        Args:
            directory: Directory path to create
        """
        directory.mkdir(parents=True, exist_ok=True)
    
    @staticmethod
    def move_file(source: Path, destination: Path) -> bool:
        """
        Move a file from source to destination.
        
        Args:
            source: Source file path
            destination: Destination file path
            
        Returns:
            True if successful, False otherwise
        """
        try:
            # Create destination directory if needed
            destination.parent.mkdir(parents=True, exist_ok=True)
            
            # Handle file name conflicts
            if destination.exists():
                destination = FileUtils._get_unique_filename(destination)
            
            shutil.move(str(source), str(destination))
            return True
        except Exception as e:
            print(f"Error moving file {source} to {destination}: {e}")
            return False
    
    @staticmethod
    def copy_file(source: Path, destination: Path) -> bool:
        """
        Copy a file from source to destination.
        
        Args:
            source: Source file path
            destination: Destination file path
            
        Returns:
            True if successful, False otherwise
        """
        try:
            # Create destination directory if needed
            destination.parent.mkdir(parents=True, exist_ok=True)
            
            # Handle file name conflicts
            if destination.exists():
                destination = FileUtils._get_unique_filename(destination)
            
            shutil.copy2(str(source), str(destination))
            return True
        except Exception as e:
            print(f"Error copying file {source} to {destination}: {e}")
            return False
    
    @staticmethod
    def _get_unique_filename(file_path: Path) -> Path:
        """
        Generate a unique filename if the original already exists.
        
        Args:
            file_path: Original file path
            
        Returns:
            Unique file path
        """
        if not file_path.exists():
            return file_path
        
        parent = file_path.parent
        stem = file_path.stem
        suffix = file_path.suffix
        
        counter = 1
        while True:
            new_name = f"{stem}_{counter}{suffix}"
            new_path = parent / new_name
            if not new_path.exists():
                return new_path
            counter += 1
    
    @staticmethod
    def get_file_size_mb(file_path: Path) -> float:
        """
        Get file size in megabytes.
        
        Args:
            file_path: Path to the file
            
        Returns:
            File size in MB
        """
        try:
            size_bytes = file_path.stat().st_size
            return size_bytes / (1024 * 1024)
        except OSError:
            return 0.0
    
    @staticmethod
    def is_safe_filename(filename: str) -> bool:
        """
        Check if a filename is safe for the filesystem.
        
        Args:
            filename: Filename to check
            
        Returns:
            True if safe, False otherwise
        """
        # Characters that are unsafe in filenames
        unsafe_chars = '<>:"/\\|?*'
        
        # Check for unsafe characters
        if any(char in filename for char in unsafe_chars):
            return False
        
        # Check for reserved names on Windows
        reserved_names = [
            'CON', 'PRN', 'AUX', 'NUL',
            'COM1', 'COM2', 'COM3', 'COM4', 'COM5', 'COM6', 'COM7', 'COM8', 'COM9',
            'LPT1', 'LPT2', 'LPT3', 'LPT4', 'LPT5', 'LPT6', 'LPT7', 'LPT8', 'LPT9'
        ]
        
        name_without_ext = Path(filename).stem.upper()
        if name_without_ext in reserved_names:
            return False
        
        return True
    
    @staticmethod
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
        
        # Ensure it's not empty
        if not sanitized:
            sanitized = "unnamed_file"
        
        return sanitized
