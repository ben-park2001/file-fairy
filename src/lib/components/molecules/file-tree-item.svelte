<script lang="ts">
  import { Folder, File } from "lucide-svelte";
  import Icon from "$lib/components/atoms/icon.svelte";
  import Text from "$lib/components/atoms/text.svelte";
  import FileTreeItem from "./file-tree-item.svelte";
  import type { DirectoryItem, BaseProps } from "$lib/types/components";

  interface Props extends BaseProps {
    item: DirectoryItem;
    depth?: number;
    showPath?: boolean;
  }

  const {
    item,
    depth = 0,
    showPath = false,
    class: className,
  }: Props = $props();
</script>

<div class={depth > 0 ? "ml-6" : className}>
  <div class="flex items-center gap-2 py-1">
    <Icon
      icon={item.is_directory ? Folder : File}
      size="sm"
      class={item.is_directory ? "text-blue-500" : "text-gray-500"}
    />
    <Text
      text={item.name}
      weight={item.is_directory ? "medium" : "normal"}
      color={item.is_directory ? "primary" : "muted"}
    />
  </div>

  {#if showPath}
    <div class="ml-6">
      <Text text={item.path} size="xs" color="muted" />
    </div>
  {/if}

  {#if item.children}
    {#each item.children as child}
      <FileTreeItem item={child} depth={depth + 1} {showPath} />
    {/each}
  {/if}
</div>
