<script lang="ts">
  import { Progress } from "$lib/components/ui/progress";
  import Text from "$lib/components/atoms/text.svelte";
  import Spinner from "$lib/components/atoms/spinner.svelte";
  import type { BaseProps } from "$lib/types/components";

  interface Props extends BaseProps {
    title: string;
    current: number;
    total: number;
    showSpinner?: boolean;
    showProgress?: boolean;
  }

  const {
    title,
    current,
    total,
    showSpinner = false,
    showProgress = true,
    class: className,
  }: Props = $props();

  const progressValue = $derived(total > 0 ? (current / total) * 100 : 0);
  const progressPercentage = $derived(Math.round(progressValue));
</script>

<div class="w-full max-w-md mx-auto space-y-3 {className}">
  <!-- Title section with spinner -->
  <div class="flex items-start justify-center gap-2 min-h-[1.5rem]">
    {#if showSpinner}
      <Spinner show={true} size="sm" class="mt-0.5 flex-shrink-0" />
    {/if}
    <div class="text-center flex-1">
      <Text
        text={title}
        size="sm"
        class="break-words leading-relaxed"
        weight="medium"
      />
    </div>
  </div>

  <!-- Progress section -->
  {#if showProgress && total > 0}
    <div class="space-y-2">
      <!-- Progress bar -->
      <div class="w-full">
        <Progress value={progressValue} max={100} class="w-full h-2" />
      </div>

      <!-- Progress stats -->
      <div class="flex items-center justify-between text-xs">
        <Text text="{current} of {total}" size="xs" color="muted" />
        <Text
          text="{progressPercentage}%"
          size="xs"
          color="muted"
          weight="medium"
        />
      </div>
    </div>
  {/if}
</div>
