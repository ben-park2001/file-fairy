<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import { X } from "lucide-svelte";
  import Icon from "$lib/components/atoms/icon.svelte";
  import Text from "$lib/components/atoms/text.svelte";
  import SearchResultItem from "$lib/components/molecules/search-result-item.svelte";
  import type { FileChunkSchema } from "$lib/types";

  interface Props {
    results: FileChunkSchema[];
    onClear: () => void;
    onOpenFile?: (result: FileChunkSchema) => void;
    onShowInFinder?: (result: FileChunkSchema) => void;
    class?: string;
  }

  const {
    results,
    onClear,
    onOpenFile,
    onShowInFinder,
    class: className,
  }: Props = $props();

  const handleOpenFile = (result: FileChunkSchema) => {
    if (onOpenFile) {
      onOpenFile(result);
    } else {
      // Default behavior - open the file
      console.log("Opening file:", result.file_path);
    }
  };

  const handleShowInFinder = (result: FileChunkSchema) => {
    if (onShowInFinder) {
      onShowInFinder(result);
    } else {
      // Default behavior - show in finder
      console.log("Showing in finder:", result.file_path);
    }
  };
</script>

<div
  class="mt-4 bg-card rounded-xl border shadow-lg overflow-hidden {className}"
>
  <div class="flex items-center justify-between p-4 border-b">
    <Text text="Search Results" weight="medium" />
    <Button variant="ghost" size="sm" onclick={onClear} class="h-8 w-8 p-0">
      <Icon icon={X} size="sm" />
    </Button>
  </div>

  <div class="max-h-96 overflow-y-auto">
    {#each results as result}
      <SearchResultItem
        {result}
        onOpen={handleOpenFile}
        onShowInFinder={handleShowInFinder}
      />
    {/each}
  </div>
</div>
