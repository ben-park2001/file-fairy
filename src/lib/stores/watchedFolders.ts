import { writable } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";

export interface WatchedFolder {
  id: string;
  path: string;
  name: string;
  is_active: boolean;
}

export interface WatchConfig {
  summary_model: string;
  filename_model: string;
  recursive: boolean;
  process_existing_files: boolean;
}

// Create a writable store for watched folders
export const watchedFolders = writable<WatchedFolder[]>([]);

// Create a writable store for watch configuration
export const watchConfig = writable<WatchConfig>({
  summary_model: "gemma2:2b",
  filename_model: "gemma2:2b",
  recursive: false,
  process_existing_files: true,
});

// Load watched folders from backend on initialization
const loadWatchedFolders = async (): Promise<void> => {
  try {
    const folders = await invoke<WatchedFolder[]>("watch_list_folders");
    watchedFolders.set(folders);
  } catch (error) {
    console.error("Failed to load watched folders:", error);
    watchedFolders.set([]);
  }
};

// Load watch configuration from backend
const loadWatchConfig = async (): Promise<void> => {
  try {
    const config = await invoke<WatchConfig>("watch_get_config");
    watchConfig.set(config);
  } catch (error) {
    console.error("Failed to load watch config:", error);
  }
};

export const watchedFoldersActions = {
  // Initialize the stores by loading data from backend
  async initialize(): Promise<void> {
    await Promise.all([loadWatchedFolders(), loadWatchConfig()]);
  },

  async addFolder(): Promise<string | null> {
    try {
      const selectedPath = await open({
        directory: true,
        multiple: false,
        title: "Select Folder to Watch",
      });

      if (selectedPath && typeof selectedPath === "string") {
        // Register the folder with the backend watch service
        await invoke("watch_register_folder", {
          folderPath: selectedPath,
        });

        // Reload the watched folders from backend to get updated list
        await loadWatchedFolders();

        return selectedPath;
      }
      return null;
    } catch (error) {
      console.error("Failed to add folder:", error);
      throw new Error(`Failed to add folder: ${error}`);
    }
  },

  async removeFolder(folderPath: string): Promise<void> {
    try {
      // Remove from backend watch service
      await invoke("watch_remove_folder", {
        folderPath: folderPath,
      });

      // Reload the watched folders from backend
      await loadWatchedFolders();
    } catch (error) {
      console.error("Failed to remove folder:", error);
      throw new Error(`Failed to remove folder: ${error}`);
    }
  },

  async toggleFolder(folderPath: string, currentState: boolean): Promise<void> {
    try {
      if (currentState) {
        // Currently active, so pause it
        await invoke("watch_pause_folder", {
          folderPath: folderPath,
        });
      } else {
        // Currently paused, so resume it
        await invoke("watch_resume_folder", {
          folderPath: folderPath,
        });
      }

      // Reload the watched folders from backend
      await loadWatchedFolders();
    } catch (error) {
      console.error("Failed to toggle folder:", error);
      throw new Error(`Failed to toggle folder: ${error}`);
    }
  },

  async updateConfig(config: WatchConfig): Promise<void> {
    try {
      await invoke("watch_update_config", {
        summaryModel: config.summary_model,
        filenameModel: config.filename_model,
        recursive: config.recursive,
        processExistingFiles: config.process_existing_files,
      });

      // Update local store
      watchConfig.set(config);
    } catch (error) {
      console.error("Failed to update config:", error);
      throw new Error(`Failed to update configuration: ${error}`);
    }
  },

  async refreshFolders(): Promise<void> {
    await loadWatchedFolders();
  },

  async validateFolders(): Promise<void> {
    try {
      const folders = await invoke<WatchedFolder[]>("watch_list_folders");
      const validatedFolders: WatchedFolder[] = [];

      for (const folder of folders) {
        try {
          const exists = await invoke<boolean>("folder_exists", {
            path: folder.path,
          });
          if (exists) {
            validatedFolders.push(folder);
          } else {
            // Remove non-existent folder from watch service
            await invoke("watch_remove_folder", { folderPath: folder.path });
          }
        } catch {
          // Remove folder that can't be validated
          try {
            await invoke("watch_remove_folder", { folderPath: folder.path });
          } catch (removeError) {
            console.error("Failed to remove invalid folder:", removeError);
          }
        }
      }

      watchedFolders.set(validatedFolders);
    } catch (error) {
      console.error("Failed to validate folders:", error);
    }
  },
};
