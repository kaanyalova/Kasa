<script lang="ts">
	import { convertFileSrc, invoke } from '@tauri-apps/api/core';
	import { onMount } from 'svelte';
	import { MediaModalStatusStore } from '../MediaModal/MediaModalStatusStore.svelte';
	import { info } from '@tauri-apps/plugin-log';

	import { commands } from '$lib/tauri_bindings';
	import Swf from '../Vector/Swf.svelte';
	import { InfiniteMediaStore } from './InfiniteMediaStore.svelte';
	import {
		clickOutside,
		clickOutsideClass,
		clickOutsideExcludingTagName,
		clickOutsideTagName
	} from '$lib/clickOutside';
	import { formatDurationShort } from '$lib/formatDuration';

	let { hash, width, height, offset_x, offset_y }: ImageProps = $props();

	let image: string = $state('');
	let promise = $derived(getThumbnail(hash));
	let mediaType = $state('');
	let isSelected = $derived(InfiniteMediaStore.selectedHashes.includes(hash));

	/**
	 * Returns the base64 encoded image from the db with `data:image/png;base64,` appended
	 * @param hash
	 * Hash of the image
	 */
	async function getThumbnail(hash: string): Promise<string> {
		return (await commands.getThumbnailFromDb(hash)) || 'rust_side_returned_null';
	}

	onMount(async () => {
		image = await getThumbnail(hash);
		mediaType = await commands.getMediaType(hash);
	});

	function onClick(e: MouseEvent) {
		if (e.ctrlKey) {
			InfiniteMediaStore.addMedia(hash);
		} else {
			MediaModalStatusStore.open(hash);
		}
	}

	function onClickOutside(node: Node, onEventFunction: any) {
		//clickOutsideExcludingTagName(node, onEventFunction, 'IMG');
		clickOutsideClass(node, onEventFunction, 'virtual-list-wrapper');
	}

	async function getMediaName(): Promise<string> {
		return commands.getMediaName(hash);
	}
</script>

<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<!-- svelte-ignore a11y_click_events_have_key_events -->

{#await promise}
	<div
		class="fakeImage"
		style="transform:translate3d({offset_x}px,0px, 0px); height:{height}px; width:{width}px"
	></div>
{:then thumbnail}
	<img
		class="thumbnail"
		onclick={(e) => onClick(e)}
		src={thumbnail}
		alt=""
		style="transform:translate3d({offset_x}px,0px, 0px); height:{height}px; width:{width}px"
		role="figure"
		class:selected={isSelected}
		decoding="async"
		loading="lazy"
	/>

	{#if mediaType === 'Video'}
		<div class="videoLength" style="transform: translate3d({offset_x + 8}px, 0px, 0px);">
			{#await commands.getVideoLength(hash) then length}
				{#if length}
					<span>
						{formatDurationShort(length)}
					</span>
				{/if}
			{/await}
		</div>
	{:else if mediaType === 'Flash'}
		<div class="mediaTypeIcon" style="transform: translate3d({offset_x + 8}px, 0px, 0px);">
			<Swf height={32} width={32}></Swf>
		</div>
	{/if}

	<!--
	TODO figure a way of setting the padding and sizes dynamically
	-->
	{#if InfiniteMediaStore.getShowNames()}
		<div
			class="nameInfoBox"
			style="transform: translate3d({offset_x}px, {height}px, 0px); width: {width}px; height: 30px"
		>
			{#await getMediaName() then name}
				{name}
			{/await}
		</div>
	{/if}
{/await}

<style>
	.mediaTypeIcon {
		position: absolute;
		padding: 8px;
		border-radius: 8px;
		top: 8px;
		fill: var(--text);
		background-color: color-mix(in srgb, black 60%, transparent 40%);
	}

	img {
		position: absolute;
		cursor: pointer;
	}

	img:hover {
		border: var(--secondary) solid 3px;
		transition: 100ms;
	}

	.selected {
		border: var(--primary) solid 3px !important;
	}

	.fakeImage {
		border: 1px solid var(--secondary-alt);
		display: flex;
		flex-grow: 1;
		align-items: center;
		justify-content: center;
		position: absolute;
	}

	.nameInfoBox {
		background-color: var(--background);
		border: 1px solid var(--secondary-alt);
		border-top: none;
		border-radius: 0px 0px 4px 4px;
		position: absolute;
		color: var(--text);
		padding: 4px;
		padding-bottom: 0px !important;
		overflow: hidden;
	}

	.videoLength {
		position: absolute;
		padding: 4px;
		border-radius: 4px;
		top: 8px;
		fill: var(--text);
		background-color: color-mix(in srgb, black 60%, transparent 40%);
		color: var(--text);
	}

	.thumbnail {
		content-visibility: auto;
	}
</style>
