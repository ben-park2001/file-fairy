"""Extractor for plain text files."""

from pathlib import Path
from .base import BaseExtractor


class PlainTextExtractor(BaseExtractor):
    """Extractor for plain text files (.txt, .md)."""
    
    def extract(self, file_path: Path) -> str:
        """Extract text from plain text files."""
        return self._read_text_file(file_path)
