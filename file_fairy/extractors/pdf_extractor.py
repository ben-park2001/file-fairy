"""Extractor for PDF files."""

from pathlib import Path
from .base import BaseExtractor


class PDFExtractor(BaseExtractor):
    """Extractor for PDF files."""
    
    def extract(self, file_path: Path) -> str:
        """Extract text from PDF files."""
        try:
            import PyPDF2
            with open(file_path, 'rb') as file:
                reader = PyPDF2.PdfReader(file)
                text = ""
                for page in reader.pages:
                    text += page.extract_text() + "\n"
                return text.strip()
        except ImportError:
            return self._handle_import_error("PDF", "pip install PyPDF2")
        except Exception as e:
            return self._handle_extraction_error(file_path, e)
