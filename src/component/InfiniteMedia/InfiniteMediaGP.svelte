<script lang="ts">
	// This should "technically work" but

	import { invoke } from '@tauri-apps/api/core';
	import { debug, error, info, trace } from '@tauri-apps/plugin-log';
	import { onDestroy, onMount, tick } from 'svelte';
	import VirtualList, { type VirtualListEvents } from 'svelte-tiny-virtual-list';
	import { sidebarStore } from '../Sidebar/SidebarStore.svelte';
	import { appWindow } from '../Decoration/utils/window';
	import { getCurrentWindow, PhysicalSize } from '@tauri-apps/api/window';
	import { listen, type UnlistenFn } from '@tauri-apps/api/event';
	import MediaThumbnail from './MediaThumbnail.svelte';
	import { commands } from '$lib/tauri_bindings';
	import { SearchStore } from '../Sidebar/Search/SearchStore.svelte';

	let values: Array<ImageRow> = $state([]);
	let heights: Array<number> = $state([]);
	let tauri_width = $state(0); // TODO this should be set to initial window size
	let tauri_height = $state(0);

	let cooldown = $state(0);

	let virtualList: any;

	let window_size_unlisten: UnlistenFn;

	// on cache update run updateLayout();
	listen('cache_updated', async (_) => {
		await updateLayoutFromCache();
	});

	listen('media_updated', async (_) => {
		await initializeLayout();
		trace('media_updated event received');
	});

	// drag and drop support
	listen('tauri://drag-drop', (event: any) => {
		// what is the type for the drag and drop event?
		const paths: Array<string> = event.event.paths;

		paths.forEach((path) => {
			commands.addIndexSource;
		});
	});

	onMount(async () => {
		const initial_size = await getCurrentWindow().innerSize();
		tauri_height = initial_size.height;
		tauri_width = initial_size.width;
		// https://v2.tauri.app/reference/javascript/api/namespacewindow/#onresized
		window_size_unlisten = await getCurrentWindow().onResized(({ payload: size }) => {
			tauri_height = size.height;
			tauri_width = size.width;
		});
	});

	onDestroy(() => {
		trace('ondestroy called!');
		window_size_unlisten();
	});

	async function onResize() {
		cooldown = setTimeout(updateLayoutFromCache, 100);
	}

	/**
	 * Gets the media from the database possibly using cached values, sets the heights for the media and media themselves to
	 * the received values.
	 */
	async function updateLayoutFromCache() {
		let _values = await commands.getLayoutFromCache(
			tauri_width - sidebarStore.size * 3 - 20,
			tauri_height / 2,
			12
		);

		if (_values === null) {
			error('Could not get layout from the rust cache');
			return;
		}

		const _heights: Array<number> = _values.map((row) => {
			// first row should have the gaps height
			if (row.index === 0) {
				return row.height;
				//return (row.height += 12);
			} else {
				return row.height;
			}
		});

		heights = _heights;
		values = _values;
		trace(`calculating sizes w:${tauri_width}`);
	}

	/**
	 * Gets the initial layout and media by querying every piece of media, than sets the values and the heights,
	 * unlike updateLayout() it retries until the database is up and does not use the cached values.
	 */
	async function initializeLayout() {
		console.log(`call init layout size is ${values.length}`);
		try {
			console.log(values.length);
			if (await commands.areDbsMounted()) {
				await commands.search(
					SearchStore.searchContents,
					tauri_width - sidebarStore.size * 3 - 20,
					12
				);
			} else {
				setTimeout(initializeLayout, 500);
			}
		} catch (error) {
			// If there's an error, try again after a delay
			setTimeout(initializeLayout, 500);
		}
	}

	onMount(async () => {
		await initializeLayout();

		// reload the values
		values = values;
	});

	$effect(async () => {
		tauri_width;
		tauri_height;
		sidebarStore.isActive;

		await onResize();
	});

	$effect(async () => {});

	onMount(() => {
		initializeLayout();
		// reload the values
		values = values;
	});
</script>

<!-- TODO  overscanCount *WILL* cause problems on larger screens, change that accordingly -->
<div class="list">
	<VirtualList
		height="100%"
		width="100%"
		itemSize={heights}
		itemCount={values.length}
		overscanCount={8}
		bind:this={virtualList}
	>
		<div slot="item" let:index let:style {style}>
			{#each values[index].images as image}
				<MediaThumbnail
					isSelected={false}
					hash={image.hash}
					height={image.height}
					width={image.width}
					offset_x={image.x_relative}
					offset_y={image.y_relative}
				></MediaThumbnail>
			{/each}
		</div>
	</VirtualList>
</div>

<style>
	.list {
		height: calc(100% - 12px); /* onscroll events won't fire without this one, why... ?*/
		position: relative;
		top: 12px;
	}

	.list :global(.virtual-list-wrapper) {
		overflow-x: hidden; /*Don't show horizontal scroll bar when moving the sidebar*/
	}
</style>
