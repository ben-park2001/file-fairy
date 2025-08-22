<script lang="ts">
  import { Plus } from "lucide-svelte";
  import {
    Dialog,
    DialogContent,
    DialogHeader,
    DialogTitle,
  } from "$lib/components/ui/dialog";
  import { Button } from "$lib/components/ui/button";
  import { Text, Icon } from "$lib/components/atoms";
  import WatchedFolderItem from "$lib/components/molecules/watched-folder-item.svelte";
  import {
    watchedFolders,
    watchedFoldersActions,
    type WatchedFolder,
  } from "$lib/stores/watchedFolders";
  import { onMount } from "svelte";

  interface Props {
    isOpen: boolean;
    onClose: () => void;
  }

  const { isOpen, onClose }: Props = $props();

  let folders = $state<WatchedFolder[]>([]);
  let isAddingFolder = $state(false);
  let error = $state<string | null>(null);

  // Subscribe to the store
  $effect(() => {
    const unsubscribe = watchedFolders.subscribe((value) => {
      folders = value;
    });
    return unsubscribe;
  });

  onMount(async () => {
    // Initialize the store and load data from backend
    try {
      await watchedFoldersActions.initialize();
      await watchedFoldersActions.validateFolders();
    } catch (err) {
      console.error("Failed to initialize watch folders:", err);
      error = "Failed to load watched folders";
    }
  });

  const toggleFolder = async (folderPath: string, currentState: boolean) => {
    try {
      error = null;
      await watchedFoldersActions.toggleFolder(folderPath, currentState);
    } catch (err) {
      console.error("Failed to toggle folder:", err);
      error = err instanceof Error ? err.message : "Failed to toggle folder";
    }
  };

  const removeFolder = async (folderPath: string) => {
    try {
      error = null;
      await watchedFoldersActions.removeFolder(folderPath);
    } catch (err) {
      console.error("Failed to remove folder:", err);
      error = err instanceof Error ? err.message : "Failed to remove folder";
    }
  };

  const addFolder = async () => {
    isAddingFolder = true;
    error = null;
    try {
      const result = await watchedFoldersActions.addFolder();
      if (result) {
        console.log("Added folder:", result);
      }
    } catch (err) {
      console.error("Failed to add folder:", err);
      error = err instanceof Error ? err.message : "Failed to add folder";
    } finally {
      isAddingFolder = false;
    }
  };
</script>

<Dialog open={isOpen} onOpenChange={onClose}>
  <DialogContent class="max-w-2xl max-h-[90vh] flex flex-col">
    <DialogHeader class="flex-shrink-0">
      <DialogTitle>
        <div class="flex items-center justify-between">
          <Text text="Watched Folders" weight="medium" />
        </div>
      </DialogTitle>
    </DialogHeader>

    {#if error}
      <div class="bg-red-50 border border-red-200 rounded-lg p-3 mb-4">
        <Text text={error} color="error" size="sm" />
      </div>
    {/if}

    <div class="flex-1 overflow-hidden flex flex-col py-4">
      {#if folders.length > 0}
        <div class="flex-1 overflow-y-auto space-y-3 p-4">
          {#each folders as folder}
            <WatchedFolderItem
              {folder}
              onToggle={toggleFolder}
              onRemove={removeFolder}
            />
          {/each}
        </div>
      {:else}
        <div class="flex-1 flex items-center justify-center">
          <div class="text-center py-8">
            <Text text="No folders being watched" color="muted" />
            <div class="mt-2">
              <Text
                text="Add a folder to create your knowledge base."
                color="muted"
                size="sm"
              />
            </div>
          </div>
        </div>
      {/if}
    </div>

    <div class="flex gap-2 border-t pt-4 flex-shrink-0">
      <div class="flex-1">
        <Button
          variant="outline"
          onclick={addFolder}
          disabled={isAddingFolder}
          class="w-full"
        >
          <Icon icon={Plus} size="sm" class="mr-2" />
          <Text text={isAddingFolder ? "Selecting..." : "Add Folder"} />
        </Button>
      </div>
      <Button onclick={onClose}>Done</Button>
    </div>
  </DialogContent>
</Dialog>
