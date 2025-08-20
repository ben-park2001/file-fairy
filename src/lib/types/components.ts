// Common interfaces used across components
export interface SearchResult {
  id: number;
  filename: string;
  path: string;
  snippet: string;
  type: string;
}

export interface DirectoryItem {
  name: string;
  path: string;
  is_directory: boolean;
  children?: DirectoryItem[];
}

// Component prop types
export interface BaseProps {
  class?: string;
}

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
