//! Service layer exports for File Fairy application
//! Provides clean access to all services, types, and composables

// === CORE SERVICES ===
export { OllamaService } from "./ollama";
export { FileOrganizationService } from "./file-organization";

// === SERVICE TYPES ===
export type { OllamaOrganizationResult, OllamaModel } from "./ollama";
export type {
  UseFileOrganizationState,
  UseFileOrganizationActions,
  UseFileOrganizationComputed,
} from "./use-file-organization";

// === COMPOSABLES ===
export {
  createFileOrganizationState,
  createFileOrganizationActions,
  createFileOrganizationComputed,
} from "./use-file-organization";
