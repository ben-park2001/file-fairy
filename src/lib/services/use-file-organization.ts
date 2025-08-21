import { invoke } from "@tauri-apps/api/core";
import { FileOrganizationService } from "$lib/services/file-organization";
import type {
  FileOrganizationResult,
  DirectoryItem,
  PathInfo,
} from "$lib/types/components";

export interface UseFileOrganizationState {
  isLoading: boolean;
  isApplying: boolean;
  structure: DirectoryItem | null;
  isTargetFile: boolean;
  filenamePreviews: FileOrganizationResult[];
  error: string | null;
  progressText: string;
  progressCurrent: number;
  progressTotal: number;
}

export interface UseFileOrganizationActions {
  loadFilenamePreviews: (folderPath: string) => Promise<void>;
  handleConfirm: (onComplete?: () => void) => Promise<void>;
  toggleFileSelection: (index: number) => void;
  toggleAllFiles: () => void;
}

export interface UseFileOrganizationComputed {
  selectedCount: number;
  validFileCount: number;
}

export function createFileOrganizationState(): UseFileOrganizationState {
  return {
    isLoading: true,
    isApplying: false,
    structure: null,
    isTargetFile: false,
    filenamePreviews: [],
    error: null,
    progressText: "",
    progressCurrent: 0,
    progressTotal: 0,
  };
}

export function createFileOrganizationActions(
  state: UseFileOrganizationState
): UseFileOrganizationActions {
  const loadFilenamePreviews = async (folderPath: string) => {
    state.isLoading = true;
    state.structure = null;
    state.isTargetFile = false;
    state.filenamePreviews = [];
    state.error = null;
    state.progressText = "Initializing...";
    state.progressCurrent = 0;
    state.progressTotal = 0;

    try {
      // Get path info first
      const pathInfo = await invoke<PathInfo>("get_path_info", {
        path: folderPath,
      });
      state.isTargetFile = !pathInfo.is_directory;

      // Load directory structure for display
      if (pathInfo.is_directory) {
        state.structure = await invoke<DirectoryItem>(
          "read_directory_structure",
          {
            path: folderPath,
          }
        );
      } else {
        state.structure = {
          name: pathInfo.name,
          path: folderPath,
          is_directory: false,
          children: undefined,
        };
      }

      // Generate AI filename previews with progress tracking
      state.filenamePreviews =
        await FileOrganizationService.generateFilenamePreviews(
          [folderPath],
          (current, total, currentFile) => {
            state.progressCurrent = current;
            state.progressTotal = total;
            state.progressText = `Analyzing ${currentFile} (${current}/${total})`;
          }
        );

      state.progressText = "Analysis complete";
    } catch (err) {
      state.error =
        err instanceof Error ? err.message : "Failed to analyze files";
      console.error("Failed to load filename previews:", err);

      // Fallback structure
      const pathName = folderPath.split("/").pop() || "Unknown";
      state.structure = {
        name: pathName,
        path: folderPath,
        is_directory: !state.isTargetFile,
        children: state.isTargetFile ? undefined : [],
      };
    } finally {
      state.isLoading = false;
    }
  };

  const handleConfirm = async (onComplete?: () => void) => {
    if (state.filenamePreviews.length === 0) {
      onComplete?.();
      return;
    }

    state.isApplying = true;
    state.progressText = "Preparing to rename files...";
    state.progressCurrent = 0;
    state.progressTotal = state.filenamePreviews.filter(
      (p) => p.isSelected && p.status === "preview"
    ).length;

    try {
      const result = await FileOrganizationService.applySelectedRenames(
        state.filenamePreviews,
        (current, total) => {
          state.progressCurrent = current;
          state.progressTotal = total;
          state.progressText = `Renaming files (${current}/${total})`;
        }
      );

      if (result.failed.length > 0) {
        console.error("Some files failed to rename:", result.failed);
        state.error = `Failed to rename ${result.failed.length} file(s)`;
      } else {
        state.progressText = "All files renamed successfully";
      }

      // Wait a moment to show completion message
      await new Promise((resolve) => setTimeout(resolve, 1000));
      onComplete?.();
    } catch (err) {
      state.error =
        err instanceof Error ? err.message : "Failed to apply renames";
      console.error("Failed to apply renames:", err);
    } finally {
      state.isApplying = false;
    }
  };

  const toggleFileSelection = (index: number) => {
    state.filenamePreviews[index].isSelected =
      !state.filenamePreviews[index].isSelected;
  };

  const toggleAllFiles = () => {
    const validPreviews = state.filenamePreviews.filter(
      (p) => p.status !== "error"
    );
    const allSelected = validPreviews.every((p) => p.isSelected);

    state.filenamePreviews.forEach((preview, index) => {
      if (preview.status !== "error") {
        state.filenamePreviews[index].isSelected = !allSelected;
      }
    });
  };

  return {
    loadFilenamePreviews,
    handleConfirm,
    toggleFileSelection,
    toggleAllFiles,
  };
}

export function createFileOrganizationComputed(
  state: UseFileOrganizationState
): UseFileOrganizationComputed {
  return {
    get selectedCount() {
      return state.filenamePreviews.filter((p) => p.isSelected).length;
    },
    get validFileCount() {
      return state.filenamePreviews.filter((p) => p.status !== "error").length;
    },
  };
}
