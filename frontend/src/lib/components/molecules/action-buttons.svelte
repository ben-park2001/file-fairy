<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import Spinner from "$lib/components/atoms/spinner.svelte";
  import type { BaseProps } from "$lib/types/components";

  interface Props extends BaseProps {
    onCancel: () => void;
    onConfirm: () => void;
    isLoading?: boolean;
    isDisabled?: boolean;
    confirmText?: string;
    selectedCount?: number;
  }

  const {
    onCancel,
    onConfirm,
    isLoading = false,
    isDisabled = false,
    confirmText = "Confirm",
    selectedCount = 0,
    class: className,
  }: Props = $props();

  const confirmButtonText = $derived(
    selectedCount > 0 ? `${confirmText} (${selectedCount})` : confirmText
  );
</script>

<div class="flex gap-2 justify-end {className}">
  <Button variant="outline" onclick={onCancel} disabled={isLoading}>
    Cancel
  </Button>
  <Button onclick={onConfirm} disabled={isLoading || isDisabled}>
    {#if isLoading}
      <Spinner show={true} size="sm" class="mr-2" />
      Applying...
    {:else}
      {confirmButtonText}
    {/if}
  </Button>
</div>
