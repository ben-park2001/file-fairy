import os
import logging
from datetime import datetime
from typing import List, Dict, Any, Optional
import lancedb

from models.schema import SearchResult
from utils.embedding import (
    embed_text,
    chunk_text,
    calculate_text_relevance,
    VECTOR_DIM,
)

logger = logging.getLogger(__name__)


class VectorDB:
    """
    A singleton class to manage all LanceDB operations.
    """

    _instance: Optional["VectorDB"] = None
    DB_PATH = "./data/lancedb"
    TABLE_NAME = "files"

    @classmethod
    def get_instance(cls) -> "VectorDB":
        """Gets the single instance of the VectorDB class."""
        if cls._instance is None:
            cls._instance = cls()
        return cls._instance

    def __init__(self):
        """
        Private constructor. Use get_instance() to get the object.
        Initializes the database connection and table.
        """
        if VectorDB._instance is not None:
            raise Exception("This class is a singleton! Use get_instance().")

        self.db = None
        self.table = None
        self._initialize_db()

    def _initialize_db(self):
        """Initializes the LanceDB connection and creates the table if it doesn't exist."""
        try:
            os.makedirs(self.DB_PATH, exist_ok=True)
            self.db = lancedb.connect(self.DB_PATH)

            if self.TABLE_NAME not in self.db.table_names():
                # Define a schema instead of sample data for a more robust creation
                schema = {
                    "vector": [0.0] * VECTOR_DIM,
                    "file_path": "init",
                    "chunk_id": 0,
                    "text": "init",
                    "file_name": "init",
                    "created_at": datetime.now().isoformat(),
                }
                self.table = self.db.create_table(self.TABLE_NAME, [schema])
                self.table.delete("file_path = 'init'")
            else:
                self.table = self.db.open_table(self.TABLE_NAME)

            logger.info(f"Database initialized and connected at {self.DB_PATH}")

        except Exception as e:
            logger.error(f"Failed to initialize database: {e}")
            raise  # Re-raise the exception to indicate a critical failure

    def upsert_file(self, file_path: str, content: str, file_name: str) -> bool:
        """Upserts a file and its chunks to the index."""
        self.remove_file(file_path)  # Remove existing entries first

        try:
            chunks = chunk_text(content)
            if not chunks:
                return False

            embeddings = embed_text(chunks)
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

            self.table.add(data)
            logger.info(f"Added {len(chunks)} chunks for {file_path}")
            return True
        except Exception as e:
            logger.error(f"Failed to add file {file_path}: {e}")
            return False

    def search_files(
        self,
        query: str,
        limit: int = 10,
        threshold: float = 0.7,
    ) -> List[SearchResult]:
        """Search files using LanceDB's native hybrid search (vector + full-text)."""
        if not self.table:
            return []

        try:
            # Generate query embedding using utility function
            query_embedding = embed_text(query)

            # Use LanceDB's native hybrid search
            results_df = (
                self.table.search(query_type="hybrid")
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

    def remove_file(self, file_path: str) -> bool:
        """Remove file from index."""
        if not self.table:
            return False

        try:
            self.table.delete(f"file_path = '{file_path}'")
            return True
        except Exception as e:
            logger.error(f"Failed to remove file: {e}")
            return False

    def is_file_indexed(self, file_path: str) -> bool:
        """Check if file is indexed."""
        if not self.table:
            return False

        try:
            results = (
                self.table.search()
                .where(f"file_path = '{file_path}'")
                .limit(1)
                .to_pandas()
            )
            return not results.empty
        except Exception:
            return False

    def get_index_stats(self) -> Dict[str, Any]:
        """Get index statistics."""
        if not self.table:
            return {"status": "not_initialized"}

        try:
            df = self.table.to_pandas()
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

    def clear_index(self) -> bool:
        """Clear entire index."""
        try:
            if self.table and self.TABLE_NAME in self.db.table_names():
                self.db.drop_table(self.TABLE_NAME)

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
            self.table = self.db.create_table(self.TABLE_NAME, sample_data)
            self.table.delete("file_path = 'init'")

            logger.info("Index cleared")
            return True

        except Exception as e:
            logger.error(f"Failed to clear index: {e}")
            return False
