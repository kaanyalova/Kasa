<script lang="ts">
	import { commands } from '$lib/tauri_bindings';
	import { mediaContext } from 'vidstack';
	import MediaThumbnail from '../../../component/InfiniteMedia/MediaThumbnail.svelte';
	import type { PageProps } from './$types';
	import { WindowTitlebar } from '../../../component/Decoration';
	import { MediaModalStatusStore } from '../../../component/MediaModal/MediaModalStatusStore.svelte';
	import { emit, emitTo } from '@tauri-apps/api/event';

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

<div class="wrapper">
	{#each data.closest as media}
		{#await getThumbnail(media.hash) then thumbnail}
			<div class="">
				<button onclick={async () => await onClickImage(media.hash)}>
					<img src={thumbnail} alt="" />
				</button>
				<span class="distance">{media.distance}</span>
			</div>
		{/await}
	{/each}
</div>

<style>
	.wrapper {
		background-color: var(--background);
		overflow-y: scroll;
		height: calc(100vh - 32px);
		color: var(--text);
	}
</style>
