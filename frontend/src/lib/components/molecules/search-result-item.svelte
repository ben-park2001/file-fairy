<script lang="ts">
	import { Button } from '$lib/components/ui/button';
	import { Folder, ExternalLink } from '@lucide/svelte';
	import Icon from '$lib/components/atoms/icon.svelte';
	import Text from '$lib/components/atoms/text.svelte';
	import type { FileChunkSchema } from '$lib/types';

	interface Props {
		result: FileChunkSchema;
		onOpen: (result: FileChunkSchema) => void;
		onShowInFinder: (result: FileChunkSchema) => void;
	}

	const { result, onOpen, onShowInFinder }: Props = $props();

	const getFileIcon = (filename: string) => {
		const extension = filename.split('.').pop()?.toLowerCase();
		switch (extension) {
			case 'pdf':
				return '📄';
			case 'xlsx':
			case 'xls':
				return '📊';
			case 'docx':
			case 'doc':
				return '📝';
			case 'txt':
				return '📝';
			case 'png':
			case 'jpg':
			case 'jpeg':
			case 'gif':
				return '🖼️';
			default:
				return '📄';
		}
	};

	const getDirectoryPath = (filePath: string) => {
		const pathParts = filePath.split('/');
		return pathParts.slice(0, -1).join('/') || '/';
	};

	const handleClick = () => onOpen(result);
	const handleKeyDown = (e: KeyboardEvent) => e.key === 'Enter' && onOpen(result);
	const handleShowInFinder = (e: Event) => {
		e.stopPropagation();
		onShowInFinder(result);
	};
</script>

<div
	role="button"
	tabindex="0"
	class="hover:bg-muted/30 group cursor-pointer border-b p-4 transition-colors last:border-b-0"
	onclick={handleClick}
	onkeydown={handleKeyDown}
>
	<div class="flex items-start gap-3">
		<div class="mt-0.5 flex-shrink-0 text-xl">
			{getFileIcon(result.file_name)}
		</div>
		<div class="min-w-0 flex-1">
			<div class="flex items-center gap-2">
				<Text text={result.file_name} weight="medium" class="truncate" />
				<Icon
					icon={ExternalLink}
					size="sm"
					class="text-muted-foreground h-3 w-3 opacity-0 transition-opacity group-hover:opacity-100"
				/>
			</div>
			<div class="text-muted-foreground mb-1 flex items-center gap-1 text-sm">
				<Icon icon={Folder} size="sm" class="h-3 w-3" />
				<Text text={getDirectoryPath(result.file_path)} size="sm" color="muted" class="truncate" />
			</div>
			<Text text={result.text} size="sm" color="muted" class="line-clamp-2" />
		</div>
		<Button
			variant="ghost"
			size="sm"
			onclick={handleShowInFinder}
			class="h-8 w-8 p-0 opacity-0 transition-opacity group-hover:opacity-100"
			title="Show in Finder"
		>
			<Icon icon={ExternalLink} size="sm" />
		</Button>
	</div>
</div>
