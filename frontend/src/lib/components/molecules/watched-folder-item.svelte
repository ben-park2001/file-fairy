<script lang="ts">
	import { Folder, X } from '@lucide/svelte';
	import { Button } from '$lib/components/ui/button';
	import { Switch } from '$lib/components/ui/switch';
	import Icon from '$lib/components/atoms/icon.svelte';
	import Text from '$lib/components/atoms/text.svelte';
	import type { WatchedFolderInfo } from '$lib/types';

	interface Props {
		folder: WatchedFolderInfo;
		onToggle: (folderPath: string, currentState: boolean) => Promise<void>;
		onRemove: (folderPath: string) => Promise<void>;
	}

	const { folder, onToggle, onRemove }: Props = $props();

	let isToggling = $state(false);
	let isRemoving = $state(false);

	const handleToggle = async () => {
		if (isToggling) return;
		isToggling = true;
		try {
			await onToggle(folder.path, folder.is_active);
		} catch (error) {
			console.error('Failed to toggle folder:', error);
		} finally {
			isToggling = false;
		}
	};

	const handleRemove = async () => {
		if (isRemoving) return;
		isRemoving = true;
		try {
			await onRemove(folder.path);
		} catch (error) {
			console.error('Failed to remove folder:', error);
		} finally {
			isRemoving = false;
		}
	};
</script>

<div class="flex items-center gap-3 rounded-lg border p-3">
	<Icon icon={Folder} size="sm" class="flex-shrink-0 text-blue-500" />
	<div class="flex min-w-0 flex-1 flex-col space-y-1">
		<Text text={folder.name} weight="medium" class="truncate" />
		<Text text={folder.path} size="sm" color="muted" class="truncate" />
	</div>
	<div class="flex items-center gap-2">
		<Button
			variant="ghost"
			size="sm"
			onclick={handleRemove}
			disabled={isRemoving}
			class="text-muted-foreground hover:text-destructive h-8 w-8 p-0"
		>
			<Icon icon={X} size="sm" />
		</Button>
	</div>
</div>
