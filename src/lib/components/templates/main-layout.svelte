<script lang="ts">
  import { Text } from "$lib/components/atoms";
  import MagicBar from "$lib/components/organisms/magic-bar.svelte";
  import OrganizeModal from "$lib/components/organisms/organize-modal.svelte";
  import WatchFolderModal from "$lib/components/organisms/watch-folder-modal.svelte";
  import SearchResults from "$lib/components/organisms/search-results.svelte";
  import NoResults from "$lib/components/molecules/no-results.svelte";
  import type { SearchResult } from "$lib/types";

  interface Props {
    searchResults?: SearchResult[];
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

  let showOrganizeModal = $state(false);
  let showWatchModal = $state(false);
  let droppedFolder = $state<string | null>(null);

  const handleFolderDrop = (folderPath: string) => {
    droppedFolder = folderPath;
    showOrganizeModal = true;
  };

  const handleSearch = (query: string) => {
    onSearch?.(query);
  };

  const handleClearSearch = () => {
    onSearch?.("");
  };

  const handleWatchClick = () => {
    showWatchModal = true;
  };

  const handleOrganize = () => {
    showOrganizeModal = false;
    onOrganize?.();
  };

  const handleCloseModals = () => {
    showOrganizeModal = false;
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

      <!-- Search Results -->
      {#if searchResults.length > 0}
        <SearchResults results={searchResults} onClear={handleClearSearch} />
      {:else if hasSearched && !isSearching}
        <!-- No Results State -->
        <NoResults query={lastSearchQuery} onClear={handleClearSearch} />
      {/if}
    </div>
  </div>

  <!-- Modals -->
  <OrganizeModal
    isOpen={showOrganizeModal}
    onClose={handleCloseModals}
    onOrganize={handleOrganize}
    folderPath={droppedFolder}
  />

  <WatchFolderModal isOpen={showWatchModal} onClose={handleCloseModals} />
</div>
