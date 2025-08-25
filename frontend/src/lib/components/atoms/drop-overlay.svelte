<script lang="ts">
  import { cn } from "$lib/utils";

  interface Props {
    isDragOver: boolean;
    class?: string;
  }

  const { isDragOver, class: className }: Props = $props();
</script>

{#if isDragOver}
  <div
    class={cn(
      "absolute inset-0 rounded-xl border-2 border-dashed border-blue-500",
      "bg-blue-50/50 dark:bg-blue-950/10 flex items-center justify-center",
      "pointer-events-none",
      className
    )}
  >
    <div class="text-blue-600 font-medium">
      Drop file or folder to organize with AI
    </div>
  </div>
{/if}
