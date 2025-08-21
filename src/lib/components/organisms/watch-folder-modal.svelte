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

  // Subscribe to the store
  $effect(() => {
    const unsubscribe = watchedFolders.subscribe((value) => {
      folders = value;
    });
    return unsubscribe;
  });

  onMount(() => {
    // Initialize default folders and validate existing ones when component mounts
    watchedFoldersActions.validateFolders();
  });

  const toggleFolder = (id: string) => {
    watchedFoldersActions.toggleFolder(id);
  };

  const removeFolder = (id: string) => {
    watchedFoldersActions.removeFolder(id);
  };

  const addFolder = async () => {
    isAddingFolder = true;
    try {
      const result = await watchedFoldersActions.addFolder();
      if (result) {
        console.log("Added folder:", result);
      }
    } catch (error) {
      console.error("Failed to add folder:", error);
    } finally {
      isAddingFolder = false;
    }
  };
</script>

<Dialog open={isOpen} onOpenChange={onClose}>
  <DialogContent class="max-w-2xl max-h-[90vh] flex flex-col">
    <DialogHeader class="flex-shrink-0">
      <DialogTitle>
        <Text text="Watched Folders" weight="medium" />
      </DialogTitle>
    </DialogHeader>

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
          </div>
        </div>
      {/if}
    </div>

    <div class="flex gap-2 justify-between border-t pt-4 flex-shrink-0">
      <Button
        variant="outline"
        onclick={addFolder}
        disabled={isAddingFolder}
        class="flex-1 bg-transparent"
      >
        <Icon icon={Plus} size="sm" class="mr-2" />
        <Text text={isAddingFolder ? "Selecting..." : "Add Folder to Watch"} />
      </Button>
      <Button onclick={onClose}>Done</Button>
    </div>
  </DialogContent>
</Dialog>
