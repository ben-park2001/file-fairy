<script lang="ts">
  import {
    Dialog,
    DialogContent,
    DialogHeader,
    DialogTitle,
  } from "$lib/components/ui/dialog";
  import { Button } from "$lib/components/ui/button";
  import FileTreeItem from "$lib/components/molecules/file-tree-item.svelte";
  import Spinner from "$lib/components/atoms/spinner.svelte";
  import Text from "$lib/components/atoms/text.svelte";
  import { invoke } from "@tauri-apps/api/core";

  interface DirectoryItem {
    name: string;
    path: string;
    is_directory: boolean;
    children?: DirectoryItem[];
  }

  interface Props {
    isOpen: boolean;
    onClose: () => void;
    onOrganize: () => void;
    folderPath: string | null;
  }

  const { isOpen, onClose, onOrganize, folderPath }: Props = $props();

  let isLoading = $state(false);
  let structure = $state<DirectoryItem | null>(null);
  let isTargetFile = $state(false);

  $effect(() => {
    if (isOpen && folderPath) {
      isLoading = true;
      structure = null;
      isTargetFile = false;

      invoke<[string, boolean]>("get_path_info", { path: folderPath })
        .then(([name, isDirectory]) => {
          isTargetFile = !isDirectory;

          if (isDirectory) {
            return invoke<DirectoryItem>("read_directory_structure", {
              path: folderPath,
            });
          } else {
            return Promise.resolve({
              name: name,
              path: folderPath,
              is_directory: false,
              children: undefined,
            } as DirectoryItem);
          }
        })
        .then((result) => {
          structure = result;
          isLoading = false;
        })
        .catch((error) => {
          console.error("Failed to read path:", error);
          const pathName = folderPath.split("/").pop() || "Unknown";
          structure = {
            name: pathName,
            path: folderPath,
            is_directory: true,
            children: [],
          };
          isLoading = false;
        });
    }
  });
</script>

<Dialog open={isOpen} onOpenChange={onClose}>
  <DialogContent class="max-w-md">
    <DialogHeader>
      <DialogTitle>
        <Text
          text={isTargetFile
            ? "File Organization Preview"
            : "Folder Organization Preview"}
          size="lg"
          weight="semibold"
        />
      </DialogTitle>
    </DialogHeader>

    <div class="py-4">
      {#if isLoading}
        <div class="flex items-center justify-center py-8">
          <Spinner show={true} />
          <Text
            text={isTargetFile
              ? "Reading file info..."
              : "Reading folder contents..."}
            class="ml-2"
          />
        </div>
      {:else if structure}
        <div class="bg-muted/30 rounded-lg p-4 max-h-64 overflow-y-auto">
          {#if isTargetFile}
            <Text
              text="Selected file for organization:"
              size="sm"
              color="muted"
              class="mb-2"
            />
            <div class="p-3 rounded">
              <FileTreeItem item={structure} showPath={true} />
            </div>
          {:else}
            <FileTreeItem item={structure} />
          {/if}
        </div>
      {/if}
    </div>

    <div class="flex gap-2 justify-end">
      <Button variant="outline" onclick={onClose}>Cancel</Button>
      <Button onclick={onOrganize} disabled={isLoading}>Organize</Button>
    </div>
  </DialogContent>
</Dialog>
