<script lang="ts">
	import SearchInput from '$lib/components/molecules/search-input.svelte';
	import WatchButton from '$lib/components/molecules/watch-button.svelte';
	import DropOverlay from '$lib/components/atoms/drop-overlay.svelte';
	import { cn } from '$lib/utils';
	import { onMount } from 'svelte';

	interface Props {
		onFolderDrop: (folderPath: string) => void;
		onSearch: (query: string) => void;
		onWatchClick: () => void;
		isSearching?: boolean;
		class?: string;
	}

	const {
		onFolderDrop,
		onSearch,
		onWatchClick,
		isSearching = false,
		class: className
	}: Props = $props();

	let query = $state<string>('');
	let isDragOver = $state<boolean>(false);

	// onMount(() => {
	// 	let unlisten: (() => void) | undefined;

	// 	const setup = async () => {
	// 		const webview = getCurrentWebview();

	// 		unlisten = await webview.onDragDropEvent(async (event) => {
	// 			if (event.payload.type === 'over') {
	// 				isDragOver = true;
	// 			} else if (event.payload.type === 'drop') {
	// 				isDragOver = false;
	// 				const paths = event.payload.paths as string[];
	// 				if (paths && paths.length > 0) {
	// 					const droppedPath = paths[0];
	// 					try {
	// 						onFolderDrop(droppedPath);
	// 					} catch (error) {
	// 						console.error('Failed to process dropped path:', error);
	// 						onFolderDrop(droppedPath);
	// 					}
	// 				}
	// 			} else {
	// 				isDragOver = false;
	// 			}
	// 		});
	// 	};

	// 	setup();
	// 	return () => unlisten?.();
	// });

	const handleSearch = (newQuery: string) => {
		query = newQuery;
		// Don't trigger search on input change, only update the query state
	};

	const handleSubmit = () => {
		if (query.trim()) {
			onSearch(query);
		}
	};

	const handleKeyDown = (e: KeyboardEvent) => {
		if (e.key === 'Escape') {
			query = '';
			onSearch('');
		} else if (e.key === 'Enter') {
			e.preventDefault();
			handleSubmit();
		}
	};

	const handleDragOver = (e: DragEvent) => e.preventDefault();
	const handleDragLeave = (e: DragEvent) => e.preventDefault();
	const handleDrop = (e: DragEvent) => e.preventDefault();
</script>

<div class={cn('relative', className)}>
	<div
		role="button"
		tabindex="0"
		class={cn(
			'relative flex items-center gap-3 rounded-xl border-2 p-4 transition-all duration-200',
			'bg-card shadow-lg hover:shadow-xl',
			isDragOver
				? 'border-blue-500 bg-blue-50 dark:bg-blue-950/20'
				: 'border-border hover:border-blue-300'
		)}
		ondragover={handleDragOver}
		ondragleave={handleDragLeave}
		ondrop={handleDrop}
	>
		<SearchInput
			value={query}
			placeholder={isDragOver
				? 'Drop file or folder to organize...'
				: 'Search files or drop file/folder to organize...'}
			isLoading={isSearching}
			onInput={handleSearch}
			onSubmit={handleSubmit}
			onKeyDown={handleKeyDown}
		/>

		<div class="flex items-center gap-2">
			<WatchButton onClick={onWatchClick} />
		</div>
	</div>

	<DropOverlay {isDragOver} />
</div>
