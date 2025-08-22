<script lang="ts">
  import { Folder, X } from "lucide-svelte";
  import { Button } from "$lib/components/ui/button";
  import { Switch } from "$lib/components/ui/switch";
  import Icon from "$lib/components/atoms/icon.svelte";
  import Text from "$lib/components/atoms/text.svelte";
  import type { WatchedFolder } from "$lib/stores/watchedFolders";

  interface Props {
    folder: WatchedFolder;
    onToggle: (folderPath: string, currentState: boolean) => Promise<void>;
    onRemove: (folderPath: string) => Promise<void>;
  }

  const { folder, onToggle, onRemove }: Props = $props();

  let isToggling = $state(false);
  let isRemoving = $state(false);

  const handleToggle = async () => {
    if (isToggling) return;
    isToggling = true;
    try {
      await onToggle(folder.path, folder.is_active);
    } catch (error) {
      console.error("Failed to toggle folder:", error);
    } finally {
      isToggling = false;
    }
  };

  const handleRemove = async () => {
    if (isRemoving) return;
    isRemoving = true;
    try {
      await onRemove(folder.path);
    } catch (error) {
      console.error("Failed to remove folder:", error);
    } finally {
      isRemoving = false;
    }
  };
</script>

<div class="flex items-center gap-3 p-3 rounded-lg border">
  <Icon icon={Folder} size="sm" class="text-blue-500 flex-shrink-0" />
  <div class="flex flex-col flex-1 min-w-0 space-y-1">
    <Text text={folder.name} weight="medium" class="truncate" />
    <Text text={folder.path} size="sm" color="muted" class="truncate" />
  </div>
  <div class="flex items-center gap-2">
    <Button
      variant="ghost"
      size="sm"
      onclick={handleRemove}
      disabled={isRemoving}
      class="h-8 w-8 p-0 text-muted-foreground hover:text-destructive"
    >
      <Icon icon={X} size="sm" />
    </Button>
  </div>
</div>
