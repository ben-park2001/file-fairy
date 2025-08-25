<script lang="ts">
	import { Plus } from '@lucide/svelte';
	import { Dialog, DialogContent, DialogHeader, DialogTitle } from '$lib/components/ui/dialog';
	import { Button } from '$lib/components/ui/button';
	import { Text, Icon } from '$lib/components/atoms';
	import WatchedFolderItem from '$lib/components/molecules/watched-folder-item.svelte';
	import type { WatchedFolderInfo } from '$lib/types';
	import { onMount } from 'svelte';

	interface Props {
		isOpen: boolean;
		onClose: () => void;
	}

	const { isOpen, onClose }: Props = $props();

	let folders = $state<WatchedFolderInfo[]>([]);
	let isAddingFolder = $state(false);
	let error = $state<string | null>(null);

	// Subscribe to the store
	// $effect(() => {
	// 	const unsubscribe = watchedFolders.subscribe((value) => {
	// 		folders = value;
	// 	});
	// 	return unsubscribe;
	// });

	// onMount(async () => {
	// 	// Load watched folders from backend
	// 	try {
	// 		await WatchServiceStore.refreshWatchedFolders();
	// 	} catch (err) {
	// 		console.error('Failed to load watched folders:', err);
	// 		error = 'Failed to load watched folders';
	// 	}
	// });

	const toggleFolder = async (folderPath: string, currentState: boolean) => {
		try {
			error = null;
			if (currentState) {
				// await WatchServiceStore.pauseFolder(folderPath);
			} else {
				// await WatchServiceStore.resumeFolder(folderPath);
			}
		} catch (err) {
			console.error('Failed to toggle folder:', err);
			error = err instanceof Error ? err.message : 'Failed to toggle folder';
		}
	};

	const removeFolder = async (folderPath: string) => {
		try {
			error = null;
			// await WatchServiceStore.removeFolder(folderPath);
		} catch (err) {
			console.error('Failed to remove folder:', err);
			error = err instanceof Error ? err.message : 'Failed to remove folder';
		}
	};

	const addFolder = async () => {
		isAddingFolder = true;
		error = null;
		try {
			// Open folder selection dialog
			// const selected = await open({
			// 	// directory: true,
			// 	multiple: false,
			// 	title: 'Select folder to watch'
			// });
			// if (selected) {
			// 	// await WatchServiceStore.registerFolder(selected as string);
			// 	console.log('Added folder:', selected);
			// }
		} catch (err) {
			console.error('Failed to add folder:', err);
			error = err instanceof Error ? err.message : 'Failed to add folder';
		} finally {
			isAddingFolder = false;
		}
	};
</script>

<Dialog open={isOpen} onOpenChange={onClose}>
	<DialogContent class="flex max-h-[90vh] max-w-2xl flex-col">
		<DialogHeader class="flex-shrink-0">
			<DialogTitle>
				<div class="flex items-center justify-between">
					<Text text="Watched Folders" weight="medium" />
				</div>
			</DialogTitle>
		</DialogHeader>

		{#if error}
			<div class="mb-4 rounded-lg border border-red-200 bg-red-50 p-3">
				<Text text={error} color="error" size="sm" />
			</div>
		{/if}

		<div class="flex flex-1 flex-col overflow-hidden py-4">
			{#if folders.length > 0}
				<div class="flex-1 space-y-3 overflow-y-auto p-4">
					{#each folders as folder}
						<WatchedFolderItem {folder} onToggle={toggleFolder} onRemove={removeFolder} />
					{/each}
				</div>
			{:else}
				<div class="flex flex-1 items-center justify-center">
					<div class="py-8 text-center">
						<Text text="No folders being watched" color="muted" />
						<div class="mt-2">
							<Text text="Add a folder to create your knowledge base." color="muted" size="sm" />
						</div>
					</div>
				</div>
			{/if}
		</div>

		<div class="flex flex-shrink-0 gap-2 border-t pt-4">
			<div class="flex-1">
				<Button variant="outline" onclick={addFolder} disabled={isAddingFolder} class="w-full">
					<Icon icon={Plus} size="sm" class="mr-2" />
					<Text text={isAddingFolder ? 'Selecting...' : 'Add Folder'} />
				</Button>
			</div>
			<Button onclick={onClose}>Done</Button>
		</div>
	</DialogContent>
</Dialog>
