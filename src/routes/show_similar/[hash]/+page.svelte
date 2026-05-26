<script lang="ts">
	import { commands } from '$lib/tauri_bindings';
	import { mediaContext } from 'vidstack';
	import MediaThumbnail from '../../../component/InfiniteMedia/MediaThumbnail.svelte';
	import type { PageProps } from './$types';
	import { WindowTitlebar } from '../../../component/Decoration';
	import { MediaModalStatusStore } from '../../../component/MediaModal/MediaModalStatusStore.svelte';
	import { emit, emitTo } from '@tauri-apps/api/event';
	import ProgressBar from '../../../component/Downloader/ProgressBar.svelte';

	let { data }: PageProps = $props();

	async function getThumbnail(hash: string): Promise<string> {
		const thumbnail_bytes = await commands.getThumbnailFromDb(hash);
		// TODO support other image formats than png
		const thumbnail = 'data:image/png;base64, ' + thumbnail_bytes;
		return thumbnail;
	}

	async function onClickImage(hash: string) {
		await emit('open_media_modal', { hash: hash });
	}
</script>

<div class="tilebar">
	<WindowTitlebar platform="gnome"></WindowTitlebar>
</div>

<!--
<ul class="wrapper multicolorRows">
	{#each data.closest as media}
		{#await getThumbnail(media.hash) then thumbnail}
			<li class="row">
				<button onclick={async () => await onClickImage(media.hash)}>
					<img src={thumbnail} alt="" class="thumnail" />
				</button>
				<span class="distance">{media.distance}</span>
			</li>
		{/await}
	{/each}
</ul>

-->

<div class="container">
	{#each data.closest as media}
		{#await getThumbnail(media.hash) then thumbnail}
			<div class="item">
				<button class="image" onclick={() => onClickImage(media.hash)}>
					<img src={thumbnail} alt="" class="thumbnail" />
				</button>
				<div class="info">
					<div class="progressBarContainer">
						Similarity
						<ProgressBar progress={1 - media.distance}></ProgressBar>
					</div>
					<button onclick={() => onClickImage(media.hash)} class="goButton">Go</button>
				</div>
			</div>
		{/await}
	{/each}
</div>

<style>
	.container {
		display: flex;
		flex-direction: column;
		gap: 8px;
		overflow-y: scroll;
		height: calc(100vh - 36px);
		padding: 8px;
		background-color: var(--background);
		color: var(--text);
	}

	.item {
		display: flex;
		flex-direction: row;
		align-items: center;
		gap: 16px;
		border: 1px solid var(--secondary-alt);
		border-radius: 4px;
		padding: 8px;
	}

	.image {
		flex: 3;
		display: flex;
		justify-content: center;
	}

	.thumbnail {
		width: 100%;
		box-sizing: border-box;

		outline: 1px solid var(--secondary-alt);
	}
	.thumbnail:hover {
		outline: 3px solid var(--accent);
	}

	.info {
		flex: 7;
		word-break: break-all;
		display: flex;
		align-items: flex-end;
		border: 1px solid var(--secondary-alt);
		padding: 8px;
		padding-right: 0;
		border-radius: 2px;
	}

	.progressBarContainer {
		flex: 8;
		display: flex;
		justify-self: center;
		flex-direction: column;
	}

	.goButton {
		background-color: var(--accent);
		margin-left: 8px;
		margin-right: 8px;
		color: var(--text-opposite);
		border: 1px solid var(--accent-border);
		border-radius: 4px;
		padding: 4px 24px;
		justify-content: center;
		align-items: center;
		flex-shrink: 0;
	}

	.goButton:hover {
		background-color: var(--accent-hover);
	}
</style>
