import { invoke } from "@tauri-apps/api/core";
import { OllamaService } from "./ollama";
import type {
  DirectoryItem,
  FileOrganizationResult,
  OrganizationProgress,
  PathInfo,
} from "$lib/types";

export class FileOrganizationService {
  private static readonly DEFAULT_SUMMARY_MODEL = "gemma3:270m";
  private static readonly DEFAULT_FILENAME_MODEL = "gemma3:1b";

  /**
   * Validates that Ollama is available and models are accessible
   */
  private static async validateOllamaSetup(
    summaryModel = this.DEFAULT_SUMMARY_MODEL,
    filenameModel = this.DEFAULT_FILENAME_MODEL
  ): Promise<void> {
    const isOllamaAvailable = await OllamaService.healthCheck();
    if (!isOllamaAvailable) {
      throw new Error(
        "Ollama server is not available. Please make sure Ollama is running."
      );
    }

    const availableModels = await OllamaService.listModels();
    if (!availableModels.includes(summaryModel)) {
      throw new Error(
        `Summary model "${summaryModel}" is not available. Available models: ${availableModels.join(
          ", "
        )}`
      );
    }
    if (!availableModels.includes(filenameModel)) {
      throw new Error(
        `Filename model "${filenameModel}" is not available. Available models: ${availableModels.join(
          ", "
        )}`
      );
    }
  }

  /**
   * Collects all files to process from given paths
   */
  private static async collectFilesToProcess(
    paths: string[]
  ): Promise<string[]> {
    const allFilesToProcess: string[] = [];

    for (const path of paths) {
      try {
        const pathInfo = await invoke<PathInfo>("get_path_info", { path });

        if (pathInfo.is_directory) {
          const allFiles = await this.getAllFiles(path);
          const supportedFiles = allFiles.filter((file) =>
            OllamaService.isFileSupported(file)
          );
          allFilesToProcess.push(...supportedFiles);
        } else {
          if (OllamaService.isFileSupported(path)) {
            allFilesToProcess.push(path);
          }
        }
      } catch (error) {
        // Add path for error handling even if it fails
        allFilesToProcess.push(path);
      }
    }

    return allFilesToProcess;
  }

  /**
   * Processes a single file and returns its organization result
   */
  private static async processSingleFile(
    filePath: string,
    summaryModel: string,
    filenameModel: string,
    status: "preview" | "completed" = "preview"
  ): Promise<FileOrganizationResult> {
    const fileName = this.getFilename(filePath);

    try {
      if (!OllamaService.isFileSupported(filePath)) {
        return {
          filePath,
          summary: "",
          originalFilename: fileName,
          newFilename: fileName,
          status: "error",
          error: "Unsupported file type",
          isSelected: false,
        };
      }

      const result = await OllamaService.analyzeAndOrganizeFile(
        filePath,
        summaryModel,
        filenameModel
      );

      return {
        filePath,
        summary: result.summary,
        originalFilename: fileName,
        newFilename: result.newFilename,
        status,
        isSelected: status === "preview", // Default to selected for preview
      };
    } catch (error) {
      return {
        filePath,
        summary: "",
        originalFilename: fileName,
        newFilename: "",
        status: "error",
        error:
          error instanceof Error ? error.message : "Failed to analyze file",
        isSelected: false,
      };
    }
  }

  /**
   * Generates filename previews for the given paths
   */
  static async generateFilenamePreviews(
    paths: string[],
    onProgress?: (current: number, total: number, currentFile: string) => void
  ): Promise<FileOrganizationResult[]> {
    await this.validateOllamaSetup();

    const allFilesToProcess = await this.collectFilesToProcess(paths);
    const results: FileOrganizationResult[] = [];

    // Process files with progress tracking
    for (let i = 0; i < allFilesToProcess.length; i++) {
      const filePath = allFilesToProcess[i];
      const fileName = this.getFilename(filePath);

      onProgress?.(i + 1, allFilesToProcess.length, fileName);

      const result = await this.processSingleFile(
        filePath,
        this.DEFAULT_SUMMARY_MODEL,
        this.DEFAULT_FILENAME_MODEL,
        "preview"
      );
      results.push(result);
    }

    return results;
  }

  /**
   * Applies the selected file renames
   */
  static async applySelectedRenames(
    results: FileOrganizationResult[],
    onProgress?: (current: number, total: number) => void
  ): Promise<{ success: string[]; failed: { path: string; error: string }[] }> {
    const selectedResults = results.filter(
      (r) => r.isSelected && r.status === "preview"
    );
    const success: string[] = [];
    const failed: { path: string; error: string }[] = [];

    for (let i = 0; i < selectedResults.length; i++) {
      const result = selectedResults[i];
      onProgress?.(i + 1, selectedResults.length);

      try {
        const directory = result.filePath.substring(
          0,
          result.filePath.lastIndexOf("/")
        );
        const newPath = `${directory}/${result.newFilename}`;

        await invoke("rename_file", {
          oldPath: result.filePath,
          newPath: newPath,
        });

        success.push(newPath);
      } catch (error) {
        failed.push({
          path: result.filePath,
          error: error instanceof Error ? error.message : "Unknown error",
        });
      }
    }

    return { success, failed };
  }

  /**
   * Organizes all supported files in a folder
   */
  static async organizeFolder(
    folderPath: string,
    onProgress?: (progress: OrganizationProgress) => void
  ): Promise<OrganizationProgress> {
    await this.validateOllamaSetup();

    // Get folder structure and supported files
    const folderStructure = await this.getFolderStructure(folderPath);
    const filesToProcess = await this.getAllFiles(folderPath);
    const supportedFiles = filesToProcess.filter((file) =>
      OllamaService.isFileSupported(file)
    );

    if (supportedFiles.length === 0) {
      throw new Error("No supported files found in the selected folder.");
    }

    const progress: OrganizationProgress = {
      totalFiles: supportedFiles.length,
      processedFiles: 0,
      isCompleted: false,
      results: [],
      phase: "analyzing",
    };

    // Process each file
    for (const filePath of supportedFiles) {
      progress.currentFile = filePath;
      onProgress?.(progress);

      const result = await this.processSingleFile(
        filePath,
        this.DEFAULT_SUMMARY_MODEL,
        this.DEFAULT_FILENAME_MODEL,
        "completed"
      );

      progress.results.push(result);
      progress.processedFiles++;
      onProgress?.(progress);
    }

    progress.isCompleted = true;
    progress.currentFile = undefined;
    progress.phase = "preview";
    onProgress?.(progress);

    return progress;
  }

  /**
   * Organizes a single file
   */
  static async organizeSingleFile(
    filePath: string
  ): Promise<FileOrganizationResult> {
    await this.validateOllamaSetup();

    if (!OllamaService.isFileSupported(filePath)) {
      throw new Error(`File type not supported: ${filePath}`);
    }

    try {
      const result = await OllamaService.analyzeAndOrganizeFile(
        filePath,
        this.DEFAULT_SUMMARY_MODEL,
        this.DEFAULT_FILENAME_MODEL
      );

      // Convert new filename to lowercase and append the original extension
      const extension = filePath.split(".").pop();
      const newFilename = `${result.newFilename.toLowerCase()}.${extension}`;

      return {
        filePath,
        summary: result.summary,
        originalFilename: this.getFilename(filePath),
        newFilename,
        status: "completed",
      };
    } catch (error) {
      throw new Error(`Failed to organize file: ${error}`);
    }
  }

  // === UTILITY METHODS ===

  /**
   * Gets the folder structure for a given path
   */
  private static async getFolderStructure(
    folderPath: string
  ): Promise<string[]> {
    try {
      const structure = await invoke<DirectoryItem>(
        "read_directory_structure",
        {
          path: folderPath,
        }
      );
      return this.extractFolderPaths(structure);
    } catch (error) {
      console.error("Failed to get folder structure:", error);
      return [folderPath]; // Fallback to just the root folder
    }
  }

  /**
   * Extracts folder paths from directory structure
   */
  private static extractFolderPaths(item: DirectoryItem): string[] {
    const paths: string[] = [];

    if (item.is_directory) {
      paths.push(item.path);

      if (item.children) {
        for (const child of item.children) {
          paths.push(...this.extractFolderPaths(child));
        }
      }
    }

    return paths;
  }

  /**
   * Gets all files in a directory recursively
   */
  private static async getAllFiles(folderPath: string): Promise<string[]> {
    try {
      const structure = await invoke<DirectoryItem>(
        "read_directory_structure",
        {
          path: folderPath,
        }
      );
      return this.extractFilePaths(structure);
    } catch (error) {
      console.error("Failed to get all files:", error);
      return [];
    }
  }

  /**
   * Extracts file paths from directory structure
   */
  private static extractFilePaths(item: DirectoryItem): string[] {
    const paths: string[] = [];

    if (!item.is_directory) {
      paths.push(item.path);
    } else if (item.children) {
      for (const child of item.children) {
        paths.push(...this.extractFilePaths(child));
      }
    }

    return paths;
  }

  /**
   * Extracts filename from full path
   */
  private static getFilename(filePath: string): string {
    return filePath.split("/").pop() || filePath;
  }

  // === PUBLIC UTILITY METHODS ===

  /**
   * Gets available Ollama models
   */
  static async getAvailableModels(): Promise<string[]> {
    try {
      return await OllamaService.listModels();
    } catch (error) {
      console.error("Failed to get available models:", error);
      return [];
    }
  }

  /**
   * Checks Ollama service status
   */
  static async checkOllamaStatus(): Promise<boolean> {
    return await OllamaService.healthCheck();
  }
}
