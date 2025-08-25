import os
import time
import logging
from watchdog.observers import Observer
from watchdog.events import FileSystemEventHandler
from core.indexer import process_file
from core.database import remove_file

logger = logging.getLogger(__name__)


class FileChangeHandler(FileSystemEventHandler):
    """Handles file system events and triggers re-indexing."""

    def __init__(self, page):
        self.page = page

    def on_created(self, event):
        if not event.is_directory:
            logger.info(f"File created: {event.src_path}, processing...")
            success = process_file(event.src_path)
            self._update_ui(
                f"Indexed new file: {os.path.basename(event.src_path)}"
                if success
                else "Failed to index new file."
            )

    def on_modified(self, event):
        if not event.is_directory:
            logger.info(f"File modified: {event.src_path}, re-indexing...")
            success = process_file(event.src_path)
            self._update_ui(
                f"Updated index for: {os.path.basename(event.src_path)}"
                if success
                else "Failed to update index."
            )

    def on_deleted(self, event):
        if not event.is_directory:
            logger.info(f"File deleted: {event.src_path}, removing from index...")
            remove_file(event.src_path)
            self._update_ui(f"Removed from index: {os.path.basename(event.src_path)}")

    def _update_ui(self, message):
        """Safely update the Flet UI from the background thread."""
        # Find the status Text control on the page to update it
        status_text = next(
            (
                c
                for c in self.page.controls
                if getattr(c, "data", None) == "status_text"
            ),
            None,
        )
        if status_text:
            status_text.value = f"[{time.strftime('%H:%M:%S')}] {message}"
            self.page.update()


class FileWatcher:
    """Manages the file system observer."""

    def __init__(self, folder_path: str, page):
        self.observer = Observer()
        self.folder_path = folder_path
        self.page = page

    def start(self):
        event_handler = FileChangeHandler(self.page)
        self.observer.schedule(event_handler, self.folder_path, recursive=True)
        self.observer.start()
        logger.info(f"Started watching folder: {self.folder_path}")

    def stop(self):
        self.observer.stop()
        self.observer.join()
        logger.info("Stopped watching folder.")
