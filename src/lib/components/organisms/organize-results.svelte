<script lang="ts">
  import { FileText, Folder } from "lucide-svelte";
  import type { BaseProps } from "$lib/types/components";

  // Import molecules
  import {
    CardHeader,
    CardContainer,
    ProgressDisplay,
    FileTreeItem,
    FileRenamePreview,
    ActionButtons,
  } from "$lib/components/molecules";
  import { Button } from "$lib/components/ui/button";
  import Text from "$lib/components/atoms/text.svelte";

  // Import composable
  import {
    createFileOrganizationState,
    createFileOrganizationActions,
    createFileOrganizationComputed,
    type UseFileOrganizationState,
  } from "$lib/services/use-file-organization";

  interface Props extends BaseProps {
    folderPath: string;
    onClear: () => void;
    onComplete?: () => void;
  }

  const { folderPath, onClear, onComplete, class: className }: Props = $props();

  // Use the composable for state management
  let state = $state<UseFileOrganizationState>(createFileOrganizationState());
  let actions = createFileOrganizationActions(state);
  let computed = createFileOrganizationComputed(state);

  // Initialize when component mounts
  $effect(() => {
    actions.loadFilenamePreviews(folderPath);
  });
</script>

<CardContainer class={className}>
  {#snippet header()}
    <CardHeader
      title={state.isTargetFile
        ? "File Organization Preview"
        : "Folder Organization Preview"}
      icon={state.isTargetFile ? FileText : Folder}
      onClose={onClear}
    />
  {/snippet}

  {#if state.isLoading || state.isApplying}
    <div class="flex items-center justify-center py-12">
      <ProgressDisplay
        title={state.progressText}
        current={state.progressCurrent}
        total={state.progressTotal}
        showSpinner={state.isLoading}
        showProgress={state.progressTotal > 0}
      />
    </div>
  {:else if state.error}
    <div class="p-6">
      <div class="p-4 bg-red-50 border border-red-200 rounded-lg">
        <Text
          text="Error"
          size="sm"
          weight="semibold"
          color="error"
          class="mb-1"
        />
        <Text text={state.error} size="sm" color="error" />
      </div>
    </div>
  {:else if state.structure}
    <div class="p-6 space-y-6">
      <!-- Original structure display -->
      <div>
        <Text text="Current structure:" size="sm" color="muted" class="mb-3" />
        <div class="bg-muted/30 rounded-lg p-4">
          {#if state.isTargetFile}
            <div class="p-3 rounded">
              <FileTreeItem item={state.structure} showPath={true} />
            </div>
          {:else}
            <FileTreeItem item={state.structure} />
          {/if}
        </div>
      </div>

      <!-- Filename previews -->
      {#if state.filenamePreviews.length > 0}
        <div>
          <div class="flex items-center justify-between mb-3">
            <div class="flex items-center space-x-4">
              <Text text="Suggested renames:" size="sm" color="muted" />
              {#if computed.validFileCount > 1}
                <Button
                  variant="outline"
                  size="sm"
                  onclick={actions.toggleAllFiles}
                  class="h-6 px-2 text-xs"
                >
                  {computed.selectedCount === computed.validFileCount
                    ? "Deselect All"
                    : "Select All"}
                </Button>
              {/if}
            </div>
            <Text
              text="{computed.selectedCount} of {state.filenamePreviews
                .length} selected"
              size="xs"
              color="muted"
            />
          </div>

          <div class="space-y-3 pr-2">
            {#each state.filenamePreviews as preview, index}
              <FileRenamePreview
                {preview}
                onToggle={() => actions.toggleFileSelection(index)}
              />
            {/each}
          </div>

          <!-- Action buttons -->
          <ActionButtons
            onCancel={onClear}
            onConfirm={() => actions.handleConfirm(onComplete)}
            isLoading={state.isApplying}
            isDisabled={state.isLoading || computed.selectedCount === 0}
            selectedCount={computed.selectedCount}
            class="mt-6 pt-4 border-t"
          />
        </div>
      {:else}
        <div class="text-center py-8">
          <Text text="No supported files found" color="muted" />
        </div>
      {/if}
    </div>
  {/if}
</CardContainer>
