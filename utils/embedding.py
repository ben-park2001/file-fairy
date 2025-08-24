import logging
from typing import List, Optional
from sklearn.cluster import MiniBatchKMeans
from sklearn.metrics import pairwise_distances_argmin_min

# Set up logging
logger = logging.getLogger(__name__)

CHUNK_SIZE = 400
OVERLAP = 50
VECTOR_DIM = 1024

def chunk_text(
    text: str, chunk_size: int = CHUNK_SIZE, overlap: int = OVERLAP
) -> List[str]:
    
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


def extract_key_chunks(
    chunk_embeddings: List[List[float]],
    chunks_text: list,
    n_clusters: int = 4,
):
    
    if len(chunks_text) <= n_clusters:
        return chunks_text

    # Use MiniBatchKMeans for speed and efficiency
    kmeans = MiniBatchKMeans(
        n_clusters=n_clusters,
        random_state=42,
        batch_size=256,
        n_init="auto",  # Suppresses a future warning
    )
    kmeans.fit(chunk_embeddings)

    # Find the index of the chunk closest to each cluster centroid
    # This is a fast way to get the most representative chunk for each theme
    closest_chunk_indices, _ = pairwise_distances_argmin_min(
        kmeans.cluster_centers_, chunk_embeddings
    )

    # Sort the indices to maintain the original document order
    sorted_indices = sorted(closest_chunk_indices)

    # Retrieve the actual text for the key chunks
    key_chunks = [chunks_text[i] for i in sorted_indices]

    return key_chunks
