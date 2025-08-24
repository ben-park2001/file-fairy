<script lang="ts">
  import { FileText, Folder, CircleCheck, CircleAlert } from "lucide-svelte";
  import type {
    BaseProps,
    FileOrganizationResult,
    OrganizationProgress,
  } from "$lib/types/components";
  import { Button } from "$lib/components/ui/button";
  import { Checkbox } from "$lib/components/ui/checkbox";
  import Text from "$lib/components/atoms/text.svelte";
  import Icon from "$lib/components/atoms/icon.svelte";
  import Spinner from "$lib/components/atoms/spinner.svelte";
  import {
    TauriFileSystemService,
    TauriRenamerService,
  } from "$lib/services/tauri-api";
  import { onMount } from "svelte";

  interface Props extends BaseProps {
    folderPath: string;
    onClear: () => void;
    onComplete?: () => void;
  }

  const { folderPath, onClear, onComplete, class: className }: Props = $props();

  let progress = $state<OrganizationProgress>({
    totalFiles: 0,
    processedFiles: 0,
    isCompleted: false,
    results: [],
    phase: "analyzing",
    currentFile: undefined,
  });

  let isTargetFile = $state(false);
  let error = $state<string | null>(null);

  // Initialize when component mounts
  onMount(async () => {
    await loadFilenamePreviews();
  });

  const loadFilenamePreviews = async () => {
    try {
      error = null;
      progress.phase = "analyzing";

      // Check if it's a file or directory
      const pathInfo = await TauriFileSystemService.getPathInfo(folderPath);
      isTargetFile = !pathInfo.is_directory;

      let filesToProcess: string[] = [];

      if (isTargetFile) {
        filesToProcess = [folderPath];
      } else {
        // Read directory structure to get files
        const directoryStructure =
          await TauriFileSystemService.readDirectoryStructure(folderPath);
        filesToProcess = extractFilePaths(directoryStructure);
      }

      progress.totalFiles = filesToProcess.length;
      progress.results = [];

      // Generate previews for each file
      for (let i = 0; i < filesToProcess.length; i++) {
        const filePath = filesToProcess[i];
        progress.currentFile = filePath;
        progress.processedFiles = i;

        try {
          const originalFilename = filePath.split("/").pop() || "";
          const newFilename =
            await TauriRenamerService.generateNewFilename(filePath);

          const result: FileOrganizationResult = {
            filePath,
            summary: `AI-generated filename based on content analysis`,
            originalFilename,
            newFilename,
            status: "preview",
            isSelected: true,
          };

          progress.results.push(result);
        } catch (err) {
          console.error(`Failed to generate filename for ${filePath}:`, err);
          const originalFilename = filePath.split("/").pop() || "";
          const result: FileOrganizationResult = {
            filePath,
            summary: `Failed to analyze file content`,
            originalFilename,
            newFilename: originalFilename,
            status: "error",
            error: err instanceof Error ? err.message : "Unknown error",
            isSelected: false,
          };
          progress.results.push(result);
        }
      }

      progress.phase = "preview";
      progress.processedFiles = filesToProcess.length;
    } catch (err) {
      console.error("Failed to load filename previews:", err);
      error = err instanceof Error ? err.message : "Failed to analyze files";
    }
  };

  const extractFilePaths = (item: any): string[] => {
    const paths: string[] = [];

    if (!item.is_directory) {
      paths.push(item.path);
    }

    if (item.children) {
      for (const child of item.children) {
        paths.push(...extractFilePaths(child));
      }
    }

    return paths;
  };

  const toggleFileSelection = (index: number) => {
    if (progress.results[index]) {
      progress.results[index].isSelected = !progress.results[index].isSelected;
    }
  };

  const applyChanges = async () => {
    try {
      error = null;
      progress.phase = "applying";
      progress.processedFiles = 0;

      const selectedResults = progress.results.filter(
        (r) => r.isSelected && r.status !== "error"
      );

      for (let i = 0; i < selectedResults.length; i++) {
        const result = selectedResults[i];
        progress.currentFile = result.filePath;
        progress.processedFiles = i;

        try {
          if (result.newFilename !== result.originalFilename) {
            const directory = result.filePath.substring(
              0,
              result.filePath.lastIndexOf("/")
            );
            const newPath = `${directory}/${result.newFilename}`;

            await TauriRenamerService.renameFile(result.filePath, newPath);
            result.status = "completed";
            result.filePath = newPath; // Update path after rename
          }
        } catch (err) {
          console.error(`Failed to rename ${result.filePath}:`, err);
          result.status = "error";
          result.error = err instanceof Error ? err.message : "Unknown error";
        }
      }

      progress.phase = "completed";
      progress.processedFiles = selectedResults.length;
      progress.isCompleted = true;

      // Call completion callback
      onComplete?.();
    } catch (err) {
      console.error("Failed to apply changes:", err);
      error = err instanceof Error ? err.message : "Failed to apply changes";
    }
  };

  const selectAll = () => {
    progress.results.forEach((result) => {
      if (result.status !== "error") {
        result.isSelected = true;
      }
    });
  };

  const selectNone = () => {
    progress.results.forEach((result) => {
      result.isSelected = false;
    });
  };
</script>

<div class="bg-card rounded-xl border shadow-lg overflow-hidden {className}">
  <!-- Header -->
  <div class="flex items-center justify-between p-4 border-b">
    <div class="flex items-center gap-2">
      <Icon icon={isTargetFile ? FileText : Folder} size="sm" />
      <Text
        text={isTargetFile
          ? "File Organization Preview"
          : "Folder Organization Preview"}
        weight="medium"
      />
    </div>
    <Button variant="ghost" size="sm" onclick={onClear} class="h-8 w-8 p-0">
      <Text text="×" />
    </Button>
  </div>

  {#if error}
    <div class="p-4 bg-red-50 border-b border-red-200">
      <div class="flex items-center gap-2">
        <Icon icon={CircleAlert} size="sm" class="text-red-500" />
        <Text text={error} color="error" size="sm" />
      </div>
    </div>
  {/if}

  {#if progress.phase === "analyzing"}
    <div class="flex items-center justify-center py-12">
      <div class="text-center space-y-3">
        <Spinner show={true} size="lg" />
        <Text text="Analyzing files..." weight="medium" />
        {#if progress.currentFile}
          <Text
            text={`Processing: ${progress.currentFile.split("/").pop()}`}
            size="sm"
            color="muted"
          />
        {/if}
        <Text
          text={`${progress.processedFiles} of ${progress.totalFiles} files`}
          size="sm"
          color="muted"
        />
      </div>
    </div>
  {:else if progress.phase === "applying"}
    <div class="flex items-center justify-center py-12">
      <div class="text-center space-y-3">
        <Spinner show={true} size="lg" />
        <Text text="Applying changes..." weight="medium" />
        {#if progress.currentFile}
          <Text
            text={`Renaming: ${progress.currentFile.split("/").pop()}`}
            size="sm"
            color="muted"
          />
        {/if}
        <Text
          text={`${progress.processedFiles} of ${progress.totalFiles} files`}
          size="sm"
          color="muted"
        />
      </div>
    </div>
  {:else if progress.phase === "preview" && progress.results.length > 0}
    <!-- Preview Results -->
    <div class="p-4 space-y-4">
      <!-- Action Buttons -->
      <div class="flex justify-between items-center">
        <div class="flex gap-2">
          <Button variant="outline" size="sm" onclick={selectAll}>
            Select All
          </Button>
          <Button variant="outline" size="sm" onclick={selectNone}>
            Select None
          </Button>
        </div>
        <Button
          onclick={applyChanges}
          disabled={!progress.results.some((r) => r.isSelected)}
        >
          Apply Changes
        </Button>
      </div>

      <!-- Results List -->
      <div class="space-y-2 max-h-96 overflow-y-auto">
        {#each progress.results as result, index}
          <div
            class="flex items-start gap-3 p-3 border rounded-lg {result.status ===
            'error'
              ? 'bg-red-50 border-red-200'
              : 'bg-white'}"
          >
            {#if result.status !== "error"}
              <Checkbox
                checked={result.isSelected || false}
                onCheckedChange={() => toggleFileSelection(index)}
                class="mt-1"
              />
            {:else}
              <Icon icon={CircleAlert} size="sm" class="text-red-500 mt-1" />
            {/if}

            <div class="flex-1 min-w-0 space-y-1">
              <div class="space-y-1">
                <Text text="Original:" size="sm" color="muted" />
                <Text
                  text={result.originalFilename}
                  size="sm"
                  class="font-mono"
                />
              </div>

              {#if result.status !== "error"}
                <div class="space-y-1">
                  <Text text="New:" size="sm" color="muted" />
                  <Text
                    text={result.newFilename}
                    size="sm"
                    class="font-mono text-blue-600"
                  />
                </div>

                <Text text={result.summary} size="xs" color="muted" />
              {:else}
                <div class="space-y-1">
                  <Text text="Error:" size="sm" color="error" />
                  <Text
                    text={result.error || "Unknown error"}
                    size="xs"
                    color="error"
                  />
                </div>
              {/if}
            </div>

            {#if result.status === "completed"}
              <Icon icon={CircleCheck} size="sm" class="text-green-500 mt-1" />
            {/if}
          </div>
        {/each}
      </div>
    </div>
  {:else if progress.phase === "completed"}
    <div class="flex items-center justify-center py-12">
      <div class="text-center space-y-3">
        <Icon icon={CircleCheck} size="lg" class="text-green-500 mx-auto" />
        <Text text="Organization completed!" weight="medium" />
        <Text
          text={`${progress.results.filter((r) => r.status === "completed").length} files renamed successfully`}
          size="sm"
          color="muted"
        />
      </div>
    </div>
  {:else}
    <div class="flex items-center justify-center py-12">
      <Text text="No files found to organize" color="muted" />
    </div>
  {/if}
</div>
