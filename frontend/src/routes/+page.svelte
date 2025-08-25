<script lang="ts">
	import MainLayout from '$lib/components/templates/main-layout.svelte';
	import type { FileChunkSchema } from '$lib/types';
	// import { WatchServiceStore } from "$lib/services/watch-store";

	let searchResults = $state<FileChunkSchema[]>([]);
	let isSearching = $state(false);
	let hasSearched = $state(false);
	let lastSearchQuery = $state('');

	const handleSearch = async (query: string) => {
		if (!query.trim()) {
			searchResults = [];
			isSearching = false;
			hasSearched = false;
			lastSearchQuery = '';
			// WatchServiceStore.clearSearchResults();
			return;
		}

		hasSearched = true;
		lastSearchQuery = query;
		isSearching = true;

		try {
			// const results = await WatchServiceStore.searchDocuments(query, 10);
			// searchResults = results;
		} catch (error) {
			console.error('Search failed:', error);
			searchResults = [];
		} finally {
			isSearching = false;
		}
	};

	const handleOrganize = () => {
		console.log('Organizing files...');
		// File organization is handled in the OrganizeResults component
	};
</script>

<MainLayout
	{searchResults}
	{isSearching}
	{hasSearched}
	{lastSearchQuery}
	onSearch={handleSearch}
	onOrganize={handleOrganize}
/>
