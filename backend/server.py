"""
File Fairy Backend Server

A FastAPI-based backend server for a local file organizing application.
This server provides AI-powered features including file indexing, semantic search,
and intelligent filename generation.

The server is designed to be run as a sidecar process by a Tauri desktop application.
"""

import os
import logging
from contextlib import asynccontextmanager
from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware
from fastapi.staticfiles import StaticFiles

from .core.database import VectorDB
from .api.endpoints import router

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
    allow_origins=["http://localhost:*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

# Include API routes
app.include_router(router, prefix="/api/v1")

frontend_build_path = "frontend/build"
if os.path.exists(frontend_build_path):
    app.mount("/", StaticFiles(directory=frontend_build_path, html=True), name="static")
else:
    logger.warning(
        f"Frontend build directory '{frontend_build_path}' not found. Static files will not be served."
    )
