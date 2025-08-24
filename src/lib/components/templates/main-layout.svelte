<script lang="ts">
  import MagicBar from "$lib/components/organisms/magic-bar.svelte";
  import OrganizeResults from "$lib/components/organisms/organize-results.svelte";
  import WatchFolderModal from "$lib/components/organisms/watch-folder-modal.svelte";
  import SearchResults from "$lib/components/organisms/search-results.svelte";
  import NoResults from "$lib/components/molecules/no-results.svelte";
  import type { FileChunkSchema } from "$lib/types";

  interface Props {
    searchResults?: FileChunkSchema[];
    isSearching?: boolean;
    hasSearched?: boolean;
    lastSearchQuery?: string;
    onSearch?: (query: string) => void;
    onOrganize?: () => void;
  }

  const {
    searchResults = [],
    isSearching = $bindable(),
    hasSearched = false,
    lastSearchQuery = "",
    onSearch,
    onOrganize,
  }: Props = $props();

  let showWatchModal = $state(false);
  let droppedFolder = $state<string | null>(null);
  let isOrganizing = $state(false);

  const handleFolderDrop = (folderPath: string) => {
    droppedFolder = folderPath;
    isOrganizing = true;
    // Clear any existing search results when organizing
    onSearch?.("");
  };

  const handleSearch = (query: string) => {
    onSearch?.(query);
    // Clear organization results when searching
    if (query.trim()) {
      isOrganizing = false;
      droppedFolder = null;
    }
  };

  const handleClearSearch = () => {
    onSearch?.("");
  };

  const handleClearOrganize = () => {
    isOrganizing = false;
    droppedFolder = null;
  };

  const handleWatchClick = () => {
    showWatchModal = true;
  };

  const handleOrganizeComplete = () => {
    isOrganizing = false;
    droppedFolder = null;
    onOrganize?.();
  };

  const handleCloseModals = () => {
    showWatchModal = false;
  };
</script>

<div class="min-h-screen bg-background">
  <div class="container mx-auto p-6 max-w-4xl">
    <div class="space-y-6">
      <!-- Header -->
      <div class="text-center space-y-2">
        <h1 class="text-3xl font-bold text-foreground">File Fairy</h1>
        <p class="text-muted-foreground">
          AI-powered file organization made simple
        </p>
      </div>

      <!-- Magic Bar -->
      <MagicBar
        onFolderDrop={handleFolderDrop}
        onSearch={handleSearch}
        onWatchClick={handleWatchClick}
        {isSearching}
      />

      <!-- Organize Results -->
      {#if isOrganizing && droppedFolder}
        <OrganizeResults
          folderPath={droppedFolder}
          onClear={handleClearOrganize}
          onComplete={handleOrganizeComplete}
        />
      {:else if searchResults.length > 0}
        <!-- Search Results -->
        <SearchResults results={searchResults} onClear={handleClearSearch} />
      {:else if hasSearched && !isSearching}
        <!-- No Results State -->
        <NoResults query={lastSearchQuery} onClear={handleClearSearch} />
      {/if}
    </div>
  </div>

  <!-- Modals -->
  <WatchFolderModal isOpen={showWatchModal} onClose={handleCloseModals} />
</div>
