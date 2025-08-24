import logging
from typing import List, Optional
from llama_cpp import Llama

# Set up logging
logger = logging.getLogger(__name__)

# Configuration
MODEL_PATH = "./data/Qwen3-0.6B-Embedding-Q8_0.gguf"
CHUNK_SIZE = 400
OVERLAP = 50
VECTOR_DIM = 1024

# Global embedding model instance
_embedding_model: Optional[Llama] = None


def initialize_embedding_model(model_path: str = MODEL_PATH):
    """Initialize the GGUF model if available."""
    global _embedding_model
    try:
        _embedding_model = Llama(
            model_path=model_path,
            embedding=True,
            pooling_type=3,
            verbose=False,
        )
        logger.info(f"GGUF model initialized: {model_path}")
        return True
    except Exception as e:
        logger.warning(f"GGUF model not available: {e}")
        return False


def get_embedding_model() -> Optional[Llama]:
    """Get the global embedding model instance."""
    return _embedding_model


def embed_text(text: str | List[str]) -> List[List[float]]:
    """
    Encode text into embeddings.

    Args:
        text: Single text string or list of text strings

    Returns:
        List of embedding vectors
    """
    if not _embedding_model:
        raise RuntimeError(
            "Embedding model not initialized. Call initialize_embedding_model() first."
        )

    try:
        return _embedding_model.embed(text)
    except Exception as e:
        logger.error(f"Failed to encode text: {e}")
        raise


def chunk_text(
    text: str, chunk_size: int = CHUNK_SIZE, overlap: int = OVERLAP
) -> List[str]:
    """
    Split text into chunks with overlap.

    Args:
        text: Text to chunk
        chunk_size: Size of each chunk
        overlap: Overlap between chunks

    Returns:
        List of text chunks
    """
    if not text:
        return []

    chunks = []
    start = 0
    text_length = len(text)

    while start < text_length:
        end = min(start + chunk_size, text_length)
        chunk = text[start:end]
        chunks.append(chunk)
        start += chunk_size - overlap  # Move start forward with overlap

    return chunks


def calculate_text_relevance(text: str, filename: str, query: str) -> float:
    """
    Calculate text relevance score for a document.

    Args:
        text: Document text content
        filename: Document filename
        query: Search query

    Returns:
        Relevance score between 0 and 1
    """
    text_lower = text.lower()
    filename_lower = filename.lower()
    query_lower = query.lower()

    # Split query into tokens for better matching
    query_tokens = query_lower.split()

    score = 0.0

    # Exact phrase matching (highest weight)
    if query_lower in text_lower:
        score += 0.5
    if query_lower in filename_lower:
        score += 0.3

    # Individual token matching
    for token in query_tokens:
        if len(token) > 2:  # Skip very short tokens
            # Text content matches
            text_matches = text_lower.count(token)
            score += min(0.1, text_matches * 0.02)

            # Filename matches (weighted higher)
            filename_matches = filename_lower.count(token)
            score += min(0.2, filename_matches * 0.1)

    # Normalize score to 0-1 range
    return min(1.0, score)
