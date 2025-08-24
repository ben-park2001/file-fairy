import { writable, type Writable } from "svelte/store";
import type { FileChunkSchema, WatchedFolderInfo } from "../types/components";
import { TauriWatchService } from "./tauri-api";

/**
 * Reactive store for watched folders
 */
export const watchedFolders: Writable<WatchedFolderInfo[]> = writable([]);

/**
 * Reactive store for search results
 */
export const searchResults: Writable<FileChunkSchema[]> = writable([]);

/**
 * Loading states
 */
export const isSearching: Writable<boolean> = writable(false);
export const isWatchingFolder: Writable<boolean> = writable(false);

/**
 * Watch service utilities with reactive updates
 */
export class WatchServiceStore {
  /**
   * Load and update watched folders
   */
  static async refreshWatchedFolders(): Promise<void> {
    try {
      const folders = await TauriWatchService.listFolders();
      watchedFolders.set(folders);
    } catch (error) {
      console.error("Failed to refresh watched folders:", error);
      throw error;
    }
  }

  /**
   * Register a folder and refresh the store
   */
  static async registerFolder(folderPath: string): Promise<void> {
    isWatchingFolder.set(true);
    try {
      await TauriWatchService.registerFolder(folderPath);
      await this.refreshWatchedFolders();
    } catch (error) {
      console.error("Failed to register folder:", error);
      throw error;
    } finally {
      isWatchingFolder.set(false);
    }
  }

  /**
   * Remove a folder and refresh the store
   */
  static async removeFolder(folderPath: string): Promise<void> {
    try {
      await TauriWatchService.removeFolder(folderPath);
      await this.refreshWatchedFolders();
    } catch (error) {
      console.error("Failed to remove folder:", error);
      throw error;
    }
  }

  /**
   * Pause a folder and refresh the store
   */
  static async pauseFolder(folderPath: string): Promise<void> {
    try {
      await TauriWatchService.pauseFolder(folderPath);
      await this.refreshWatchedFolders();
    } catch (error) {
      console.error("Failed to pause folder:", error);
      throw error;
    }
  }

  /**
   * Resume a folder and refresh the store
   */
  static async resumeFolder(folderPath: string): Promise<void> {
    try {
      await TauriWatchService.resumeFolder(folderPath);
      await this.refreshWatchedFolders();
    } catch (error) {
      console.error("Failed to resume folder:", error);
      throw error;
    }
  }

  /**
   * Search documents and update results store
   */
  static async searchDocuments(
    query: string,
    limit: number = 10
  ): Promise<FileChunkSchema[]> {
    if (!query.trim()) {
      searchResults.set([]);
      return [];
    }

    isSearching.set(true);
    try {
      const results = await TauriWatchService.searchDocuments(query, limit);
      searchResults.set(results);
      return results;
    } catch (error) {
      console.error("Failed to search documents:", error);
      searchResults.set([]);
      throw error;
    } finally {
      isSearching.set(false);
    }
  }

  /**
   * Clear search results
   */
  static clearSearchResults(): void {
    searchResults.set([]);
  }
}
