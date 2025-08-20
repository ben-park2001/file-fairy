import { writable } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";

export interface WatchedFolder {
  id: string;
  path: string;
  name: string;
  isActive: boolean;
}

// Create a writable store for watched folders
export const watchedFolders = writable<WatchedFolder[]>([]);

// Load initial folders from localStorage or set defaults
const loadWatchedFolders = (): WatchedFolder[] => {
  if (typeof window !== "undefined") {
    const stored = localStorage.getItem("watchedFolders");
    if (stored) {
      try {
        return JSON.parse(stored);
      } catch (e) {
        console.error("Failed to parse stored watched folders:", e);
      }
    }
  }

  // Start with empty array - user will add folders
  return [];
};

// Initialize the store
watchedFolders.set(loadWatchedFolders());

// Subscribe to changes and save to localStorage
watchedFolders.subscribe((folders) => {
  if (typeof window !== "undefined") {
    localStorage.setItem("watchedFolders", JSON.stringify(folders));
  }
});

export const watchedFoldersActions = {
  async addFolder(): Promise<string | null> {
    try {
      const selectedPath = await open({
        directory: true,
        multiple: false,
        title: "Select Folder to Watch",
      });

      if (selectedPath && typeof selectedPath === "string") {
        const folderName = await invoke<string>("get_folder_name", {
          path: selectedPath,
        });
        const newFolder: WatchedFolder = {
          id: Date.now().toString(),
          path: selectedPath,
          name: folderName,
          isActive: true,
        };

        watchedFolders.update((folders) => [...folders, newFolder]);
        return selectedPath;
      }
      return null;
    } catch (error) {
      console.error("Failed to add folder:", error);
      return null;
    }
  },

  removeFolder(id: string): void {
    watchedFolders.update((folders) =>
      folders.filter((folder) => folder.id !== id)
    );
  },

  toggleFolder(id: string): void {
    watchedFolders.update((folders) =>
      folders.map((folder) =>
        folder.id === id ? { ...folder, isActive: !folder.isActive } : folder
      )
    );
  },

  async validateFolders(): Promise<void> {
    watchedFolders.update((folders) =>
      folders.filter(async (folder) => {
        try {
          return await invoke<boolean>("folder_exists", { path: folder.path });
        } catch {
          return false;
        }
      })
    );
  },
};
