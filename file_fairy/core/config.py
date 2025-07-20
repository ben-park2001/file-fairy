"""Configuration management for File Fairy."""

import os
from pathlib import Path
from typing import Dict, List
from dataclasses import dataclass, field
from enum import Enum


class FileCategory(Enum):
    """File categories with Korean names."""
    DOCUMENTS = ('문서', ['.pdf', '.docx', '.pptx', '.hwp', '.txt', '.md'])
    IMAGES = ('이미지', ['.jpg', '.jpeg', '.png', '.gif', '.bmp', '.tiff'])
    DATA = ('데이터', ['.csv', '.xlsx', '.json', '.xls'])
    AUDIO = ('음악', ['.mp3', '.wav', '.flac', '.m4a', '.aac', '.ogg'])
    VIDEO = ('동영상', ['.mp4', '.avi', '.mkv', '.mov', '.wmv'])
    ARCHIVES = ('압축파일', ['.zip', '.7z', '.rar', '.tar', '.gz'])
    CODE = ('코드', ['.py', '.js', '.html', '.css', '.java', '.cpp', '.c'])
    OTHER = ('기타', [])
    
    def __init__(self, korean_name: str, extensions: List[str]):
        self.korean_name = korean_name
        self.extensions = extensions
    
    @classmethod
    def get_all_extensions(cls) -> List[str]:
        """Get all supported extensions."""
        extensions = []
        for category in cls:
            extensions.extend(category.extensions)
        return extensions
    
    @classmethod
    def get_category_for_extension(cls, extension: str) -> str:
        """Get category for file extension."""
        extension = extension.lower()
        if not extension.startswith('.'):
            extension = f'.{extension}'
        
        for category in cls:
            if extension in category.extensions:
                return category.korean_name
        return cls.OTHER.korean_name
    
    @classmethod
    def get_categories_dict(cls) -> Dict[str, List[str]]:
        """Get categories as dictionary for backward compatibility."""
        return {cat.korean_name: cat.extensions for cat in cls}


@dataclass
class AIConfig:
    """AI-related configuration."""
    model_path: str = "gemma-3n-E2B-it-ONNX"
    max_tokens: int = 64
    enable_vision: bool = True
    enable_audio: bool = True
    
    @classmethod
    def from_env(cls) -> 'AIConfig':
        """Create config from environment variables."""
        return cls(
            model_path=os.getenv('FILE_FAIRY_MODEL_PATH', cls.model_path),
            max_tokens=int(os.getenv('FILE_FAIRY_MAX_TOKENS', str(cls.max_tokens))),
            enable_vision=os.getenv('FILE_FAIRY_ENABLE_VISION', 'true').lower() == 'true',
            enable_audio=os.getenv('FILE_FAIRY_ENABLE_AUDIO', 'true').lower() == 'true'
        )


@dataclass
class CategoryConfig:
    """File categorization configuration."""
    categories: Dict[str, List[str]] = field(default_factory=lambda: FileCategory.get_categories_dict())
    
    def get_category_for_extension(self, extension: str) -> str:
        """Get category for file extension."""
        return FileCategory.get_category_for_extension(extension)


@dataclass
class AppConfig:
    """Main application configuration."""
    log_file: str = "file_fairy.log"
    exclude_patterns: List[str] = field(default_factory=lambda: [
        '.git', '.DS_Store', 'node_modules', 'venv', '__pycache__',
        'Thumbs.db', '.vscode', '.idea', 'file_fairy.log'
    ])
    ai: AIConfig = field(default_factory=AIConfig)
    categories: CategoryConfig = field(default_factory=CategoryConfig)
    
    @classmethod
    def load(cls) -> 'AppConfig':
        """Load configuration from environment and defaults."""
        return cls(
            log_file=os.getenv('FILE_FAIRY_LOG_FILE', cls.log_file),
            ai=AIConfig.from_env(),
        )
    
    def should_exclude(self, path: Path) -> bool:
        """Check if path should be excluded."""
        path_str = str(path)
        return any(pattern in path_str for pattern in self.exclude_patterns)
    
    def get_category_for_extension(self, extension: str) -> str:
        """Get category for file extension."""
        return self.categories.get_category_for_extension(extension)
