"""
File Fairy Backend Server

A FastAPI-based backend server for a local file organizing application.
This server provides AI-powered features including file indexing, semantic search,
and intelligent filename generation.

The server is designed to be run as a sidecar process by a Tauri desktop application.
"""

import logging
import uvicorn
from contextlib import asynccontextmanager
from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware

from core.database import VectorDB
from api.endpoints import router

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s - %(name)s - %(levelname)s - %(message)s",
    handlers=[logging.StreamHandler(), logging.FileHandler("file_fairy.log")],
)

logger = logging.getLogger(__name__)


@asynccontextmanager
async def lifespan(app: FastAPI):
    """Application lifespan manager."""
    # Startup
    logger.info("Initializing File Fairy Backend...")
    VectorDB().get_instance()  # Initialize database instance

    yield

    # Shutdown
    logger.info("Shutting down File Fairy Backend...")


# Create FastAPI application
app = FastAPI(
    title="File Fairy Backend",
    description="AI-powered file organizing backend server",
    version="0.1.0",
    docs_url="/docs",
    redoc_url="/redoc",
    lifespan=lifespan,
)

# Configure CORS for desktop application integration
app.add_middleware(
    CORSMiddleware,
    allow_origins=["http://localhost:*", "https://tauri.localhost"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

# Include API routes
app.include_router(router, prefix="/api/v1")


# Root endpoint
@app.get("/")
async def root():
    """Root endpoint providing basic service information."""
    return {
        "service": "File Fairy Backend",
        "version": "0.1.0",
        "status": "running",
        "docs": "/docs",
        "api": "/api/v1",
    }


# Health check endpoint (alternative to /api/v1/ping)
@app.get("/health")
async def health_check():
    """Simple health check endpoint."""
    return {"status": "healthy"}


# Main entry point
if __name__ == "__main__":
    logger.info("Starting File Fairy Backend server...")

    # Run the server
    uvicorn.run(
        "main:app",
        host="127.0.0.1",  # Localhost only for security
        port=8000,
        reload=True,  # Enable auto-reload during development
        log_level="info",
    )
