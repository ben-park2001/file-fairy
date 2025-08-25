# main.py

import webview
import uvicorn
import threading

# Import the FastAPI app instance from your server file
from backend.server import app


def run_server():
    """
    Runs the FastAPI server using uvicorn.
    Note: We run the server programmatically here instead of using the command line.
    """
    uvicorn.run(app, host="127.0.0.1", port=8000)


if __name__ == "__main__":
    # 1. Start the FastAPI server in a background thread.
    #    The `daemon=True` flag ensures the thread will close when the main app exits.
    server_thread = threading.Thread(target=run_server, daemon=True)
    server_thread.start()

    # 2. Create and start the PyWebview window.
    #    This creates a native window that displays your web-based UI.
    webview.create_window(
        "File Fairy",  # The window title
        "http://127.0.0.1:8000",  # The URL of your local server
        width=800,
        height=600,
        resizable=True,
    )
    webview.start()
