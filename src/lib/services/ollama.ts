import { invoke } from "@tauri-apps/api/core";
import type { OrganizationResult } from "$lib/types";

export interface OllamaModel {
  name: string;
}

/// Simplified result for Ollama-specific operations
export interface OllamaOrganizationResult {
  summary: string;
  newFilename: string;
}

/**
 * Service for interacting with Ollama AI models through the Rust backend
 */
export class OllamaService {
  // === HEALTH AND MODEL MANAGEMENT ===

  /**
   * Checks if Ollama service is available
   */
  static async healthCheck(): Promise<boolean> {
    try {
      return await invoke("ollama_health_check");
    } catch (error) {
      console.error("Ollama health check failed:", error);
      return false;
    }
  }

  /**
   * Lists available Ollama models
   */
  static async listModels(): Promise<string[]> {
    try {
      return await invoke("ollama_list_models");
    } catch (error) {
      console.error("Failed to list Ollama models:", error);
      return [];
    }
  }

  // === FILE OPERATIONS ===

  /**
   * Extracts content from a file
   */
  static async extractFileContent(filePath: string): Promise<string> {
    try {
      return await invoke("extract_file_content", { filePath });
    } catch (error) {
      console.error("Failed to extract file content:", error);
      throw new Error(`Failed to extract content from ${filePath}: ${error}`);
    }
  }

  /**
   * Renames a file
   */
  static async renameFile(oldPath: string, newPath: string): Promise<void> {
    try {
      await invoke("rename_file", { oldPath, newPath });
    } catch (error) {
      console.error("Failed to rename file:", error);
      throw new Error(`Failed to rename file: ${error}`);
    }
  }

  // === AI GENERATION METHODS ===

  static async generateSummary(
    content: string,
    model: string
  ): Promise<string> {
    try {
      return await invoke("ollama_generate_summary", {
        content: content,
        model,
      });
    } catch (error) {
      console.error("Failed to generate summary:", error);
      throw new Error(`Failed to generate summary: ${error}`);
    }
  }

  static async generateFilename(
    summary: string,
    model: string
  ): Promise<string> {
    try {
      return await invoke("ollama_generate_filename", {
        summary,
        model,
      });
    } catch (error) {
      console.error("Failed to generate filename:", error);
      throw new Error(`Failed to generate filename: ${error}`);
    }
  }

  static async generateOrganizationPath(
    summary: string,
    filePath: string,
    folderStructure: string[],
    model: string
  ): Promise<string> {
    try {
      return await invoke("ollama_generate_organization_path", {
        summary,
        filePath,
        folderStructure,
        model,
      });
    } catch (error) {
      console.error("Failed to generate organization path:", error);
      throw new Error(`Failed to generate organization path: ${error}`);
    }
  }

  /**
   * Analyzes and organizes a file using the Rust backend
   */
  static async analyzeAndOrganizeFile(
    filePath: string,
    summaryModel: string = "gemma3:270m",
    filenameModel: string = "gemma3:1b"
  ): Promise<OllamaOrganizationResult> {
    try {
      const result = await invoke<OrganizationResult>(
        "analyze_and_organize_file",
        {
          filePath,
          summaryModel,
          filenameModel,
        }
      );

      return {
        summary: result.summary,
        newFilename: result.new_filename,
      };
    } catch (error) {
      console.error("Failed to analyze and organize file:", error);
      throw new Error(`Failed to analyze and organize file: ${error}`);
    }
  }

  // === FILE SUPPORT UTILITIES ===

  /**
   * Gets list of supported file extensions
   */
  static getSupportedFileTypes(): string[] {
    return [".txt", ".md", ".pdf", ".docx", ".xlsx", ".pptx", ".hwp"];
  }

  /**
   * Checks if a file type is supported for processing
   */
  static isFileSupported(filePath: string): boolean {
    const extension = filePath.toLowerCase().split(".").pop();
    return this.getSupportedFileTypes().includes(`.${extension}`);
  }
}
