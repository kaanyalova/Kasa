<script lang="ts">
	// This should "technically work" but

	import { invoke } from '@tauri-apps/api/core';
	import { error, info, trace } from '@tauri-apps/plugin-log';
	import { onDestroy, onMount, tick } from 'svelte';
	import VirtualList, { type VirtualListEvents } from 'svelte-tiny-virtual-list';
	import { sidebarStore } from '../Sidebar/SidebarStore.svelte';
	import { appWindow } from '../Decoration/utils/window';
	import { getCurrentWindow, PhysicalSize } from '@tauri-apps/api/window';
	import { listen, type UnlistenFn } from '@tauri-apps/api/event';
	import MediaThumbnail from './MediaThumbnail.svelte';
	import { commands } from '$lib/tauri_bindings';

	let values: Array<ImageRow> = $state([]);
	let heights: Array<number> = $state([]);
	let tauri_width = $state(0); // TODO this should be set to initial window size
	let tauri_height = $state(0);

	let is_db_mounted = $state(false);
	let cooldown = $state(0);

	let virtualList: any;

	let window_size_unlisten: UnlistenFn;

	// on cache update run updateLayout();
	listen('cache_updated', async (_) => {
		await updateLayoutFromCache();
	});

	listen('media_updated', async (_) => {
		await updateLayout();
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
		cooldown = setTimeout(updateLayoutFromCache, 500);
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

	$effect(async () => {
		tauri_width;
		tauri_height;
		sidebarStore.size;

		await onResize();
	});

	/**
	 * Gets the initial layout and media by querying every piece of media, than sets the values and the heights,
	 * unlike updateLayout() it retries until the database is up and does not use the cached values.
	 */
	async function updateLayout() {
		try {
			let _values = await commands.queryAll(tauri_width - sidebarStore.size * 3 - 10, 200, 12);

			if ((await !commands.areDbsMounted()) || _values === null) {
				/*(_values.length === 0)*/ setTimeout(updateLayout, 500);
			} else {
				is_db_mounted = true;
				const _heights: Array<number> = _values.map((row) => row.height);

				values = _values;
				heights = _heights;
			}
		} catch (error) {
			// If there's an error, try again after a delay
			setTimeout(updateLayout, 500);
		}
	}

	onMount(() => {
		updateLayout();

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
