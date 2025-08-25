<script lang="ts">
	import { Search } from '@lucide/svelte';
	import { Input } from '$lib/components/ui/input';
	import { Button } from '$lib/components/ui/button';
	import Icon from '$lib/components/atoms/icon.svelte';
	import Spinner from '$lib/components/atoms/spinner.svelte';
	import { cn } from '$lib/utils';

	interface Props {
		value: string;
		placeholder?: string;
		isLoading?: boolean;
		showSearchButton?: boolean;
		onInput?: (value: string) => void;
		onSubmit?: () => void;
		onKeyDown?: (e: KeyboardEvent) => void;
		class?: string;
	}

	const {
		value = '',
		placeholder = 'Search...',
		isLoading = false,
		showSearchButton = true,
		onInput,
		onSubmit,
		onKeyDown,
		class: className
	}: Props = $props();

	const handleInput = (e: Event) => {
		const target = e.target as HTMLInputElement;
		onInput?.(target.value);
	};

	const handleSubmit = (e: Event) => {
		e.preventDefault();
		if (value.trim()) {
			onSubmit?.();
		}
	};

	const handleKeyDown = (e: KeyboardEvent) => {
		onKeyDown?.(e);
	};
</script>

<form class={cn('relative flex-1', className)} onsubmit={handleSubmit}>
	<Input
		{value}
		{placeholder}
		oninput={handleInput}
		onkeydown={handleKeyDown}
		class="placeholder:text-muted-foreground border-0 bg-transparent focus-visible:ring-0 focus-visible:ring-offset-0"
	/>

	{#if isLoading}
		<div class="absolute right-3 top-1/2 -translate-y-1/2">
			<Spinner show={true} size="sm" />
		</div>
	{/if}

	{#if showSearchButton && value && !isLoading}
		<div class="absolute right-3 top-1/2 -translate-y-1/2">
			<Button type="submit" size="sm" class="h-8 w-8 p-0">
				<Icon icon={Search} size="sm" />
			</Button>
		</div>
	{/if}
</form>
