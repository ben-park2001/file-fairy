<script lang="ts">
  import type { BaseProps } from "$lib/types/components";
  import type { Snippet } from "svelte";

  interface Props extends BaseProps {
    maxHeight?: string;
    header?: Snippet;
    children: Snippet;
  }

  const {
    maxHeight = "70vh",
    header,
    children,
    class: className,
  }: Props = $props();
</script>

<div
  class="mt-4 bg-card rounded-xl border shadow-lg overflow-hidden {className}"
>
  {#if header}
    {@render header()}
  {/if}

  <div class="max-h-[{maxHeight}] overflow-y-auto">
    {@render children()}
  </div>
</div>
