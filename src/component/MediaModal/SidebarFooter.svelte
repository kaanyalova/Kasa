<script lang="ts">
	import { readImage, writeImage, writeText } from '@tauri-apps/plugin-clipboard-manager';
	import { readFile, writeFile } from '@tauri-apps/plugin-fs';
	import { commands } from '$lib/tauri_bindings';
	import { Image } from '@tauri-apps/api/image';
	import { openFilePickerWithMultipleFolderSelection } from '$lib/openFilePicker';
	import type { SidebarFooterProps } from './SidebarFooter';
	import { emit } from '@tauri-apps/api/event';
	import { WebviewWindow } from '@tauri-apps/api/webviewWindow';

	let { data, mediaUrl }: SidebarFooterProps = $props();

	let showCopySuccessButton = $state(false);

	let favoriteState = $state(data.isFavorite);

	async function toggleFavorite() {
		favoriteState = !favoriteState;
		await commands.setMediaFavorite(data.hash, favoriteState);
	}

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
		await commands.openWithSystemDefaultApp(mediaUrl);
	}

	async function onShowOnFileManagerButtonClicked() {
		await commands.openFileManagerWithFileSelected(data.pathThatExists ?? data.paths[0]);
	}

	async function onClickShowSimilarButtonClicked() {
		const window = new WebviewWindow('source_data_viewer', {
			url: `/show_similar/${data.hash}/`,
			decorations: false
		});
	}
</script>

<div class="sidebarFooter">
	<button title="Copy {data.mediaType}" onclick={() => onCopyButtonClicked()}>
		{#if showCopySuccessButton}
			<span class="icon-[material-symbols--check] h-5 w-5"></span>
		{:else}
			<span class="icon-[material-symbols--content-copy] h-5 w-5"></span>
		{/if}
	</button>
	<button title="Open Externally" onclick={() => onOpenExternallyButtonClicked()}>
		<span class="icon-[material-symbols--open-in-new] h-5 w-5"></span>
	</button>
	<button title="Open Folder" onclick={() => onShowOnFileManagerButtonClicked()}>
		<span class="icon-[material-symbols--folder-open] h-5 w-5"></span>
	</button>
	<button title="Favorite" onclick={async () => await toggleFavorite()}>
		{#if favoriteState}
			<span class="icon-[material-symbols--favorite] h-5 w-5 bg-[#f14c45]"></span>
		{:else}
			<span class="icon-[material-symbols--favorite-outline] h-5 w-5"></span>
		{/if}
	</button>
	<button title="Show Similar" onclick={async () => onClickShowSimilarButtonClicked()}>
		<span class="icon-[material-symbols--image-search] h-5 w-5"></span>
	</button>
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
