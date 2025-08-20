<script lang="ts">
  import { Folder, X } from "lucide-svelte";
  import { Button } from "$lib/components/ui/button";
  import { Switch } from "$lib/components/ui/switch";
  import Icon from "$lib/components/atoms/icon.svelte";
  import Text from "$lib/components/atoms/text.svelte";
  import type { WatchedFolder } from "$lib/stores/watchedFolders";

  interface Props {
    folder: WatchedFolder;
    onToggle: (id: string) => void;
    onRemove: (id: string) => void;
  }

  const { folder, onToggle, onRemove }: Props = $props();
</script>

<div class="flex items-center gap-3 p-3 rounded-lg border">
  <Icon icon={Folder} size="sm" class="text-blue-500 flex-shrink-0" />
  <div class="flex-1 min-w-0">
    <Text text={folder.name} weight="medium" class="truncate" />
    <Text text={folder.path} size="sm" color="muted" class="truncate" />
  </div>
  <div class="flex items-center gap-2">
    <Switch
      checked={folder.isActive}
      onCheckedChange={() => onToggle(folder.id)}
    />
    <Button
      variant="ghost"
      size="sm"
      onclick={() => onRemove(folder.id)}
      class="h-8 w-8 p-0 text-muted-foreground hover:text-destructive"
    >
      <Icon icon={X} size="sm" />
    </Button>
  </div>
</div>
