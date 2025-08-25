<script lang="ts">
  import Text from "$lib/components/atoms/text.svelte";
  import { cn } from "$lib/utils";

  interface Props {
    query: string;
    onClear?: () => void;
    class?: string;
  }

  const { query, onClear, class: className }: Props = $props();
</script>

<div
  class={cn(
    "mt-4 bg-card rounded-xl border shadow-lg overflow-hidden",
    className
  )}
>
  <div class="flex items-center justify-between p-4 border-b">
    <Text text="Search Results" weight="medium" />
    {#if onClear}
      <button
        onclick={onClear}
        class="h-8 w-8 flex items-center justify-center rounded-md hover:bg-muted/50 transition-colors"
        title="Clear search"
      >
        <Text text="✕" size="sm" color="muted" />
      </button>
    {/if}
  </div>
  <div class="p-8 text-center">
    <Text text={`No results found for "${query}"`} size="sm" color="muted" />
  </div>
</div>
