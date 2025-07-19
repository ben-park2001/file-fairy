"""Constants and configuration for FileIO Utils."""

from typing import List


class SupportedFormats:
    """Centralized definition of supported file formats."""
    
    DOCUMENTS = ['.pdf', '.docx', '.pptx', '.hwp', '.txt', '.md']
    IMAGES = ['.jpg', '.jpeg', '.png', '.gif']
    DATA = ['.csv', '.xlsx', '.json']
    AUDIO = ['.mp3', '.wav', '.flac', '.m4a', '.aac']
    
    # Korean category names for backward compatibility
    CATEGORIES = {
        '문서': DOCUMENTS,
        '이미지': IMAGES,
        '데이터': DATA,
        '음악': AUDIO
    }
    
    @classmethod
    def get_all_extensions(cls) -> List[str]:
        """Get all supported file extensions."""
        return cls.DOCUMENTS + cls.IMAGES + cls.DATA + cls.AUDIO
    
    @classmethod
    def get_category_for_extension(cls, extension: str) -> str:
        """Get the category name for a given file extension."""
        extension = extension.lower()
        for category, extensions in cls.CATEGORIES.items():
            if extension in extensions:
                return category
        raise ValueError(f"Unsupported file extension: {extension}")
    
    @classmethod
    def is_supported(cls, extension: str) -> bool:
        """Check if a file extension is supported."""
        return extension.lower() in cls.get_all_extensions()


class MessageTypes:
    """Message content types."""
    TEXT = "text"
    IMAGE = "image"
    AUDIO = "audio"


class Roles:
    """Message roles."""
    USER = "user"
    ASSISTANT = "assistant"
