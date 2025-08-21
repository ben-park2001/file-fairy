<script lang="ts">
  import { Checkbox } from "$lib/components/ui/checkbox";
  import Text from "$lib/components/atoms/text.svelte";
  import type {
    FileOrganizationResult,
    BaseProps,
  } from "$lib/types/components";

  interface Props extends BaseProps {
    preview: FileOrganizationResult;
    onToggle: () => void;
  }

  const { preview, onToggle, class: className }: Props = $props();
</script>

<div
  class="flex items-start space-x-3 p-4 bg-muted/30 rounded-lg border transition-all hover:shadow-sm {preview.isSelected
    ? 'border-blue-200 bg-blue-50'
    : 'border-gray-200'} {className}"
>
  <Checkbox
    bind:checked={preview.isSelected}
    onchange={onToggle}
    class="mt-1 flex-shrink-0"
    disabled={preview.status === "error"}
  />

  <div class="flex-1 min-w-0">
    {#if preview.status === "error"}
      <div class="text-red-600">
        <Text
          text={preview.originalFilename}
          size="sm"
          weight="medium"
          class="truncate block"
        />
        <Text
          text={preview.error || "Failed to analyze"}
          size="xs"
          color="error"
          class="mt-1"
        />
      </div>
    {:else}
      <div class="space-y-2">
        <div class="grid grid-cols-1 gap-1">
          <div class="flex items-start space-x-2">
            <Text
              text="From:"
              size="xs"
              color="muted"
              class="flex-shrink-0 mt-0.5"
            />
            <Text text={preview.originalFilename} size="sm" class="break-all" />
          </div>
          <div class="flex items-start space-x-2">
            <Text
              text="To:"
              size="xs"
              color="muted"
              class="flex-shrink-0 mt-0.5"
            />
            <Text
              text={preview.newFilename}
              size="sm"
              weight="medium"
              color="primary"
              class="break-all"
            />
          </div>
        </div>

        {#if preview.summary}
          <details class="group">
            <summary
              class="cursor-pointer text-xs text-muted-foreground hover:text-foreground transition-colors list-none"
            >
              <span class="group-open:hidden">▶ Show summary</span>
              <span class="hidden group-open:inline">▼ Hide summary</span>
            </summary>
            <div
              class="mt-2 p-2 bg-gray-50 rounded text-xs text-gray-600 leading-relaxed"
            >
              {preview.summary}
            </div>
          </details>
        {/if}
      </div>
    {/if}
  </div>
</div>
