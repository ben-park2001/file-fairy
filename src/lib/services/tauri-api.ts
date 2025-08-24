import { invoke } from "@tauri-apps/api/core";
import type {
  DirectoryItem,
  PathInfo,
  WatchedFolderInfo,
  FileChunkSchema,
} from "../types/components";

/**
 * File system service functions that invoke Rust backend commands
 */
export class TauriFileSystemService {
  /**
   * Get the folder name from a path
   */
  static async getFolderName(path: string): Promise<string> {
    return await invoke<string>("get_folder_name", { path });
  }

  /**
   * Get path information (name and whether it's a directory)
   */
  static async getPathInfo(path: string): Promise<PathInfo> {
    return await invoke<PathInfo>("get_path_info", { path });
  }

  /**
   * Check if a folder exists
   */
  static async folderExists(path: string): Promise<boolean> {
    return await invoke<boolean>("folder_exists", { path });
  }

  /**
   * Read directory structure with depth limit
   */
  static async readDirectoryStructure(path: string): Promise<DirectoryItem> {
    return await invoke<DirectoryItem>("read_directory_structure", { path });
  }

  /**
   * Extract content from a file
   */
  static async extractFileContent(filePath: string): Promise<string> {
    return await invoke<string>("extract_file_content", { filePath });
  }
}

/**
 * AI-powered file renaming service
 */
export class TauriRenamerService {
  /**
   * Generate a new filename using Ollama AI
   */
  static async generateNewFilename(filePath: string): Promise<string> {
    return await invoke<string>("generate_new_filename", { filePath });
  }

  /**
   * Rename/move a file
   */
  static async renameFile(oldPath: string, newPath: string): Promise<void> {
    return await invoke<void>("rename_file", { oldPath, newPath });
  }
}

/**
 * Watch service for monitoring folders and managing embeddings
 */
export class TauriWatchService {
  /**
   * Register a folder to watch for automatic file organization and embedding
   */
  static async registerFolder(folderPath: string): Promise<void> {
    return await invoke<void>("watch_register_folder", { folderPath });
  }

  /**
   * Pause watching a folder
   */
  static async pauseFolder(folderPath: string): Promise<void> {
    return await invoke<void>("watch_pause_folder", { folderPath });
  }

  /**
   * Resume watching a folder
   */
  static async resumeFolder(folderPath: string): Promise<void> {
    return await invoke<void>("watch_resume_folder", { folderPath });
  }

  /**
   * Remove a folder from watch list
   */
  static async removeFolder(folderPath: string): Promise<void> {
    return await invoke<void>("watch_remove_folder", { folderPath });
  }

  /**
   * List all watched folders
   */
  static async listFolders(): Promise<WatchedFolderInfo[]> {
    return await invoke<WatchedFolderInfo[]>("watch_list_folders");
  }

  /**
   * Search documents using semantic search
   */
  static async searchDocuments(
    query: string,
    limit: number = 10
  ): Promise<FileChunkSchema[]> {
    return await invoke<FileChunkSchema[]>("search_documents", {
      query,
      limit,
    });
  }
}

/**
 * Unified API service that combines all backend operations
 */
export class TauriApiService {
  static filesystem = TauriFileSystemService;
  static renamer = TauriRenamerService;
  static watch = TauriWatchService;
}
