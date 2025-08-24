//! Type definitions for the File Fairy application
//! Contains data structures for components, services, and application state

// === BASE COMPONENT TYPES ===

/// Base properties that all components can accept
export interface BaseProps {
  class?: string;
  id?: string;
}

// === CORE DATA INTERFACES ===

/// Search result from file content analysis
export interface SearchResult {
  readonly id: number;
  readonly filename: string;
  readonly path: string;
  readonly snippet: string;
  readonly type: string;
}

/// Directory tree item with hierarchical structure
export interface DirectoryItem {
  readonly name: string;
  readonly path: string;
  readonly is_directory: boolean;
  readonly children?: readonly DirectoryItem[];
}

/// Basic path information (matches Rust PathInfo)
export interface PathInfo {
  readonly name: string;
  readonly is_directory: boolean;
}

/// Watched folder information (matches Rust WatchedFolderInfo)
export interface WatchedFolderInfo {
  readonly id: string;
  readonly path: string;
  readonly name: string;
  readonly is_active: boolean;
}

/// File chunk with embedding data (matches Rust FileChunkSchema)
export interface FileChunkSchema {
  readonly chunk_id: number;
  readonly created_at: number;
  readonly file_path: string;
  readonly file_name: string;
  readonly text: string;
  readonly vector: number[];
}

/// AI analysis result from Rust backend (matches Rust OrganizationResult)
export interface OrganizationResult {
  readonly summary: string;
  readonly new_filename: string;
}

/// Enhanced file organization result with UI state and additional metadata
export interface FileOrganizationResult {
  filePath: string; // Mutable for path updates when files are moved
  readonly summary: string;
  readonly originalFilename: string;
  readonly newFilename: string;
  status: "completed" | "error" | "preview"; // Mutable for progress tracking
  error?: string; // Mutable for error reporting
  isSelected?: boolean; // Mutable for UI interaction
}

/// Progress tracking for batch operations
export interface OrganizationProgress {
  totalFiles: number;
  processedFiles: number; // Mutable for progress tracking
  isCompleted: boolean; // Mutable for state management
  results: FileOrganizationResult[]; // Mutable for accumulation
  phase: "analyzing" | "preview" | "applying" | "completed"; // Mutable for state tracking
  currentFile?: string; // Mutable for progress indication
}

// Component prop types
export interface IconProps extends BaseProps {
  icon: any;
  size?: "sm" | "md" | "lg";
}

export interface TextProps extends BaseProps {
  text: string;
  size?: "xs" | "sm" | "md" | "lg" | "xl";
  weight?: "normal" | "medium" | "semibold" | "bold";
  color?: "primary" | "secondary" | "muted" | "error" | "success";
}

export interface SpinnerProps extends BaseProps {
  show: boolean;
  size?: "sm" | "md" | "lg";
  color?: "primary" | "secondary" | "muted";
}

export interface DropOverlayProps extends BaseProps {
  isDragOver: boolean;
}
