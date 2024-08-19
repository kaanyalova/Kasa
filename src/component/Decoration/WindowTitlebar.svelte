<script lang="ts">
	import { osType } from './utils/os';
	import { cn } from './utils/utils';
	import type { WindowControlsProps } from './types';
	import WindowControls from './WindowControls.svelte';

	export let controlsOrder = 'system';
	export let windowControlsProps: WindowControlsProps = {};
	export let platform: string;

	const left =
		controlsOrder === 'left' ||
		(controlsOrder === 'platform' && windowControlsProps?.platform === 'macos') ||
		(controlsOrder === 'system' && platform === 'macos');

	const props = (ml: string) => {
		if (windowControlsProps?.justify !== undefined) return windowControlsProps;

		const {
			justify: windowControlsJustify,
			class: windowControlsClass,
			...restProps
		} = windowControlsProps;
		return {
			justify: false,
			class: cn(windowControlsClass, ml),
			...restProps
		};
	};
</script>

<div
	{...$$props}
	class={cn(
		'bg-background flex select-none overflow-hidden flex-row-reverse titlebar',
		$$props.class
	)}
	data-tauri-drag-region
>
	{#if left}
		<WindowControls {...props('ml-0')} {platform} />
		<slot></slot>
	{:else}
		<WindowControls {...props('')} {platform} />
		<slot></slot>
	{/if}
</div>

<style>
	.titlebar {
		background-color: var(--secondary-alt);
		height: 32px;
		fill: var(--text);
		position: relative;
		z-index: 999;
	}
</style>
