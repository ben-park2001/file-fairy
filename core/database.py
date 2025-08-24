"""
Clean and minimal LanceDB CRUD operations for file indexing and semantic search.
"""

import os
import logging
from datetime import datetime
from typing import List, Dict, Any
import lancedb


from models.schema import SearchResult
from utils.embedding import (
    chunk_text,
    calculate_text_relevance,
    VECTOR_DIM,
)
from utils.ollama import get_embedding

# Set up logging
logger = logging.getLogger(__name__)

# Configuration
DB_PATH = "./data/lancedb"
MODEL_NAME = "Qwen3-0.6B-Embedding"

# Global instances
_db = None
_table = None


def initialize_db(db_path: str = DB_PATH, model_name: str = MODEL_NAME) -> bool:
    """Initialize database and embedding model."""
    global _db, _table

    try:
        os.makedirs(db_path, exist_ok=True)
        _db = lancedb.connect(db_path)

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


def add_file(file_path: str, content: str, file_name: str) -> bool:
    """Add file to index."""
    if not _table:
        logger.error("Database not initialized")
        return False

    try:
        # Remove existing file
        remove_file(file_path)

        # Chunk content using utility function
        chunks = chunk_text(content)
        if not chunks:
            return False

        # Generate embeddings using utility function
        embeddings = get_embedding(chunks)

        # Prepare data
        data = []
        created_at = datetime.now().isoformat()

        for i, (chunk, embedding) in enumerate(zip(chunks, embeddings)):
            data.append(
                {
                    "vector": embedding,
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
    if not _table:
        return []

    try:
        # Generate query embedding using utility function
        query_embedding = embed_text(query)

        # Use LanceDB's native hybrid search
        results_df = (
            _table.search(query_type="hybrid")
            .vector(query_embedding)
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
                    # Fallback: calculate our own hybrid score using utility function
                    similarity_score = calculate_text_relevance(
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
