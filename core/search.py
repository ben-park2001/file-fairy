"""
Semantic search functionality for finding relevant files based on natural language queries.
This module handles query processing, vector similarity search, and result ranking.
"""

import logging

from models.schema import SearchResponse
from core.database import search_files as db_search_files

# Set up logging
logger = logging.getLogger(__name__)


def find_relevant_files(
    query: str,
    limit: int = 10,
    similarity_threshold: float = 0.7,
) -> SearchResponse:
    """
    Find files relevant to a natural language query using semantic search.

    Args:
        query (str): Natural language search query
        limit (int): Maximum number of results to return (1-100)
        similarity_threshold (float): Minimum similarity score for results (0.0-1.0)

    Returns:
        SearchResponse: Search results with metadata
    """
    logger.info(f"Performing search for query: '{query}'")

    if not query.strip():
        return SearchResponse(success=False, query=query, results=[], total_results=0)

    try:
        # Use database search
        results = db_search_files(query, limit, similarity_threshold)

        return SearchResponse(
            success=True,
            query=query,
            results=results,
            total_results=len(results),
        )

    except Exception as e:
        logger.error(f"Search failed for query '{query}': {str(e)}")
        return SearchResponse(success=False, query=query, results=[], total_results=0)
