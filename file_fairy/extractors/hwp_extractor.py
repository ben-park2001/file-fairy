"""Extractor for HWP (Korean word processor) files."""

import zlib
import struct
import re
import unicodedata
from pathlib import Path
from typing import List
from .base import BaseExtractor


class HWPExtractor(BaseExtractor):
    """
    HWP file text extractor that handles Korean word processor files.
    """
    
    # HWP file constants
    FILE_HEADER_SECTION = "FileHeader"
    HWP_SUMMARY_SECTION = "\x05HwpSummaryInformation"
    SECTION_NAME_LENGTH = len("Section")
    BODYTEXT_SECTION = "BodyText"
    HWP_TEXT_TAGS = [67]

    def extract(self, file_path: Path) -> str:
        """Extract text from HWP files."""
        try:
            return self._extract_hwp_content(file_path)
        except Exception as e:
            return self._handle_extraction_error(file_path, e)

    def _extract_hwp_content(self, file_path: Path) -> str:
        """Main HWP extraction logic."""
        ole_file = self._load_ole_file(file_path)
        directories = ole_file.listdir()
        
        if not self._is_valid_hwp(directories):
            raise ValueError("Not a valid HWP file")
        
        is_compressed = self._is_compressed(ole_file)
        sections = self._get_body_sections(directories)
        
        extracted_text = ""
        for section in sections:
            extracted_text += self._extract_section_text(ole_file, section, is_compressed)
            extracted_text += "\n"
        
        ole_file.close()
        return extracted_text.strip()

    def _load_ole_file(self, file_path: Path):
        """Load OLE file using olefile library."""
        try:
            import olefile
            return olefile.OleFileIO(file_path)
        except ImportError:
            raise ImportError(self._handle_import_error("HWP", "pip install olefile"))

    def _is_valid_hwp(self, directories: List) -> bool:
        """Check if the file is a valid HWP file."""
        has_header = [self.FILE_HEADER_SECTION] in directories
        has_summary = [self.HWP_SUMMARY_SECTION] in directories
        return has_header and has_summary

    def _is_compressed(self, ole_file) -> bool:
        """Check if the document format is compressed."""
        header_stream = ole_file.openstream("FileHeader")
        header_data = header_stream.read()
        return (header_data[36] & 1) == 1

    def _get_body_sections(self, directories: List) -> List[str]:
        """Get list of bodytext sections."""
        section_numbers = []
        for directory in directories:
            if directory[0] == self.BODYTEXT_SECTION:
                section_num = int(directory[1][self.SECTION_NAME_LENGTH:])
                section_numbers.append(section_num)
        
        return [f"BodyText/Section{num}" for num in sorted(section_numbers)]

    def _extract_section_text(self, ole_file, section_name: str, is_compressed: bool) -> str:
        """Extract text from a specific section."""
        section_stream = ole_file.openstream(section_name)
        raw_data = section_stream.read()

        if is_compressed:
            try:
                unpacked_data = zlib.decompress(raw_data, -15)
            except zlib.error:
                return ""
        else:
            unpacked_data = raw_data

        return self._parse_section_data(unpacked_data)

    def _parse_section_data(self, data: bytes) -> str:
        """Parse section data to extract text content."""
        size = len(data)
        position = 0
        text_content = ""
        
        while position < size:
            try:
                header = struct.unpack_from("<I", data, position)[0]
                record_type = header & 0x3ff
                record_length = (header >> 20) & 0xfff

                if record_type in self.HWP_TEXT_TAGS:
                    record_data = data[position + 4:position + 4 + record_length]
                    decoded_text = self._decode_record_data(record_data)
                    if decoded_text:
                        text_content += decoded_text + "\n"

                position += 4 + record_length
                
            except (struct.error, IndexError):
                # Skip problematic data and continue
                position += 1
                continue

        return text_content

    def _decode_record_data(self, record_data: bytes) -> str:
        """Decode record data to text."""
        try:
            decoded_text = record_data.decode('utf-16le')
            return self._clean_text(decoded_text)
        except UnicodeDecodeError:
            return ""

    def _clean_text(self, text: str) -> str:
        """Clean extracted text by removing unwanted characters."""
        # Remove Chinese characters
        text = re.sub(r'[\u4e00-\u9fff]+', '', text)
        
        # Remove control characters
        text = "".join(char for char in text if unicodedata.category(char)[0] != "C")
        
        # Remove excessive whitespace
        text = re.sub(r'\s+', ' ', text).strip()
        
        return text
