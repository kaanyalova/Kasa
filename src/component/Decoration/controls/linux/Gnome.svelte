<script>
	import Button from '../../components/Button.svelte';
	import Icons from '../../components/Icons.svelte';
	import { cn } from '../../utils/utils';
	import {
		closeWindow,
		initializeAppWindow,
		maximizeWindow,
		minimizeWindow
	} from '../../utils/window';
	import { onMount } from 'svelte';

	onMount(async () => {
		await initializeAppWindow();
	});

	const isWindowMaximized = 0;
</script>

<div
	data-tauri-drag-region
	{...$$props}
	class={cn('mx-[10px] h-auto items-center space-x-[13px]', $$props.class)}
>
	<Button
		on:click={async () => await minimizeWindow()}
		class="gnomeButton my-1 aspect-square h-6 w-6 cursor-default rounded-full"
	>
		<Icons icon="minimizeWin" class="h-[9px] w-[9px]" />
	</Button>
	<Button
		on:click={async () => await maximizeWindow()}
		class="gnomeButton my-1 aspect-square h-6 w-6 cursor-default rounded-full"
	>
		{#if isWindowMaximized}
			<Icons icon="maximizeRestoreWin" class="h-[9px] w-[9px]" />
		{:else}
			<Icons icon="maximizeWin" class="h-2 w-2" />
		{/if}
	</Button>
	<Button
		on:click={async () => await closeWindow()}
		class="gnomeButton my-1 aspect-square h-6 w-6 cursor-default rounded-full p-0"
	>
		<Icons icon="closeWin" class="h-2 w-2" />
	</Button>
</div>

<style>
	:global(.gnomeButton) {
		background-color: color-mix(in srgb, var(--background) 30%, transparent);
		color: var(--text);
	}
	:global(.gnomeButton:hover) {
		/*color: var(--text-opposite);*/
	}

	@media (prefers-color-scheme: light) {
		:global(.gnomeButton:hover) {
			background-color: color-mix(
				in srgb,
				color-mix(in srgb, var(--background) 30%, transparent) 90%,
				white 10%
			);
		}
	}
	@media (prefers-color-scheme: dark) {
		:global(.gnomeButton:hover) {
			background-color: color-mix(
				in srgb,
				color-mix(in srgb, var(--background) 30%, transparent) 90%,
				white 10%
			);
		}
	}
</style>
