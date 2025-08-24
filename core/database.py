"""
Clean and minimal LanceDB CRUD operations for file indexing and semantic search.
"""

import os
import logging
from datetime import datetime
from typing import List, Dict, Any
import lancedb
from sentence_transformers import SentenceTransformer

from models.schema import SearchResult

# Set up logging
logger = logging.getLogger(__name__)

# Configuration
DB_PATH = "./data/lancedb"
MODEL_NAME = "Qwen3-0.6B-Embedding"
CHUNK_SIZE = 400
OVERLAP = 50
VECTOR_DIM = 1024

# Global instances
_db = None
_model = None
_table = None


def initialize_db(db_path: str = DB_PATH, model_name: str = MODEL_NAME) -> bool:
    """Initialize database and embedding model."""
    global _db, _model, _table

    try:
        os.makedirs(db_path, exist_ok=True)
        _db = lancedb.connect(db_path)
        _model = SentenceTransformer(model_name)

        # Create table if it doesn't exist
        if "files" not in _db.table_names():
            sample_data = [
                {
                    "vector": [0.0] * VECTOR_DIM,
                    "file_path": "init",
                    "chunk_id": 0,
                    "text": "init",
                    "file_name": "init",
                    "created_at": datetime.now().isoformat(),
                }
            ]
            _table = _db.create_table("files", sample_data)
            _table.delete("file_path = 'init'")
        else:
            _table = _db.open_table("files")

        logger.info(f"Database initialized at {db_path}")
        return True

    except Exception as e:
        logger.error(f"Failed to initialize database: {e}")
        return False


def _chunk_text(text: str) -> List[str]:
    """Split text into chunks."""
    if not text:
        return []

    chunks = []
    start = 0
    text_length = len(text)

    while start < text_length:
        end = min(start + CHUNK_SIZE, text_length)
        chunk = text[start:end]
        chunks.append(chunk)
        start += CHUNK_SIZE - OVERLAP  # Move start forward with overlap

    return chunks


def _calculate_text_relevance(text: str, filename: str, query: str) -> float:
    """Calculate text relevance score for a document."""
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


def add_file(file_path: str, content: str, file_name: str) -> bool:
    """Add file to index."""
    if not _table or not _model:
        logger.error("Database not initialized")
        return False

    try:
        # Remove existing file
        remove_file(file_path)

        # Chunk content
        chunks = _chunk_text(content)
        if not chunks:
            return False

        # Generate embeddings
        embeddings = _model.encode(chunks)

        # Prepare data
        data = []
        created_at = datetime.now().isoformat()

        for i, (chunk, embedding) in enumerate(zip(chunks, embeddings)):
            data.append(
                {
                    "vector": embedding.tolist(),
                    "file_path": file_path,
                    "chunk_id": i,
                    "text": chunk,
                    "file_name": file_name,
                    "created_at": created_at,
                }
            )

        _table.add(data)
        logger.info(f"Added {len(chunks)} chunks for {file_path}")
        return True

    except Exception as e:
        logger.error(f"Failed to add file {file_path}: {e}")
        return False


def search_files(
    query: str, limit: int = 10, threshold: float = 0.7
) -> List[SearchResult]:
    """Search files using LanceDB's native hybrid search (vector + full-text)."""
    if not _table or not _model:
        return []

    try:
        # Generate query embedding for vector search
        query_embedding = _model.encode([query])[0]

        # Use LanceDB's native hybrid search
        results_df = (
            _table.search(query_type="hybrid")
            .vector(query_embedding.tolist())
            .text(query)
            .limit(limit * 2)  # Get more results for filtering
            .to_pandas()
        )

        # Process hybrid search results
        final_results = []

        if not results_df.empty:
            for _, row in results_df.iterrows():
                # LanceDB hybrid search provides a combined score
                # The exact scoring mechanism depends on LanceDB version
                # We'll use the distance/score provided by LanceDB
                if hasattr(row, "_distance"):
                    similarity_score = 1.0 - float(row["_distance"])
                elif hasattr(row, "_score"):
                    similarity_score = float(row["_score"])
                else:
                    # Fallback: calculate our own hybrid score
                    similarity_score = _calculate_text_relevance(
                        str(row["text"]), str(row["file_name"]), query
                    )

                # Apply threshold filter
                if similarity_score >= threshold:
                    excerpt = (
                        str(row["text"])[:200] + "..."
                        if len(str(row["text"])) > 200
                        else str(row["text"])
                    )

                    final_results.append(
                        {
                            "file_path": row["file_path"],
                            "file_name": row["file_name"],
                            "similarity_score": round(similarity_score, 3),
                            "excerpt": excerpt,
                        }
                    )

        # Sort by similarity score and limit results
        final_results.sort(key=lambda x: x["similarity_score"], reverse=True)
        limited_results = final_results[:limit]

        return [SearchResult(**result) for result in limited_results]

    except Exception as e:
        logger.error(f"Hybrid search failed: {e}")
        return []


def remove_file(file_path: str) -> bool:
    """Remove file from index."""
    if not _table:
        return False

    try:
        _table.delete(f"file_path = '{file_path}'")
        return True
    except Exception as e:
        logger.error(f"Failed to remove file: {e}")
        return False


def is_file_indexed(file_path: str) -> bool:
    """Check if file is indexed."""
    if not _table:
        return False

    try:
        results = (
            _table.search().where(f"file_path = '{file_path}'").limit(1).to_pandas()
        )
        return not results.empty
    except Exception:
        return False


def get_index_stats() -> Dict[str, Any]:
    """Get index statistics."""
    if not _table:
        return {"status": "not_initialized"}

    try:
        df = _table.to_pandas()
        unique_files = df["file_path"].nunique() if not df.empty else 0
        total_chunks = len(df)

        return {
            "status": "ready",
            "total_files": unique_files,
            "total_chunks": total_chunks,
            "last_update": df["created_at"].max() if not df.empty else None,
        }
    except Exception as e:
        return {"status": "error", "error": str(e)}


def clear_index() -> bool:
    """Clear entire index."""
    global _table

    try:
        if _db and "files" in _db.table_names():
            _db.drop_table("files")

        # Recreate empty table
        sample_data = [
            {
                "vector": [0.0] * VECTOR_DIM,
                "file_path": "init",
                "chunk_id": 0,
                "text": "init",
                "file_name": "init",
                "created_at": datetime.now().isoformat(),
            }
        ]
        _table = _db.create_table("files", sample_data)
        _table.delete("file_path = 'init'")

        logger.info("Index cleared")
        return True

    except Exception as e:
        logger.error(f"Failed to clear index: {e}")
        return False
