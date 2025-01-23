<script lang="ts">
	import { readImage, writeImage, writeText } from '@tauri-apps/plugin-clipboard-manager';
	import ArrowUpRightFromSquare from '../Vector/ArrowUpRightFromSquare.svelte';
	import Clipboard from '../Vector/Clipboard.svelte';
	import { readFile, writeFile } from '@tauri-apps/plugin-fs';
	import Tick from '../Vector/Tick.svelte';
	import { commands } from '$lib/tauri_bindings';
	import { Image } from '@tauri-apps/api/image';

	let { data }: SidebarFooterProps = $props();

	let showCopySuccessButton = $state(false);

	async function onCopyButtonClicked() {
		if (data.mediaType === 'Image') {
			// TODO: Copy the actual image data
			// https://github.com/tauri-apps/plugins-workspace/issues/2208

			const rawImage = await commands.imagePathToRgbaBytes(data.paths[0]);
			const image = await Image.new(rawImage.bytes, rawImage.width, rawImage.height);
			await writeImage(image);

			showCopySuccessButton = true;
			setTimeout(() => {
				showCopySuccessButton = false;
			}, 1000);
		} else {
			await writeText(data.paths[0]);
		}
	}

	async function onOpenExternallyButtonClicked() {
		await commands.openWithSystemDefaultApp(data.paths[0]);
	}
</script>

<div class="sidebarFooter">
	<button title="Copy {data.mediaType}" onclick={() => onCopyButtonClicked()}>
		{#if showCopySuccessButton}
			<Tick height={20} width={20}></Tick>
		{:else}
			<Clipboard height={20} width={20}></Clipboard>
		{/if}
	</button>
	<button title="Open Externally" onclick={() => onOpenExternallyButtonClicked()}>
		<ArrowUpRightFromSquare height={20} width={20}></ArrowUpRightFromSquare></button
	>
	<button title="Action"> Act </button>
	<button title="Action"> Act </button>
	<button title="Action"> Act </button>
</div>

<style>
	.sidebarFooter {
		height: 50px;
		display: flex;
		align-items: center;
		justify-content: center;
		border-top: 1px solid var(--secondary-alt);
	}

	button {
		margin: 5px;
		padding: 5px;
		fill: var(--text);
		display: flex;
		align-items: center;
		justify-content: center;
		border: 1px solid var(--secondary-alt);
		border-radius: 4px;
		height: 35px;
		width: 35px;
	}
	button:hover {
		background-color: var(--secondary-alt);
	}
</style>
