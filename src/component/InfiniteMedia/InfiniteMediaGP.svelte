<script lang="ts">
	// This should "technically work" but

	import { invoke } from '@tauri-apps/api/core';
	import { debug, error, info, trace } from '@tauri-apps/plugin-log';
	import { onDestroy, onMount, tick } from 'svelte';
	import VirtualList, { type VirtualListEvents } from 'svelte-tiny-virtual-list';
	import { sidebarStore } from '../Sidebar/SidebarStore.svelte';
	import { appWindow } from '../Decoration/utils/window';
	import { getCurrentWindow, PhysicalSize } from '@tauri-apps/api/window';
	import { emit, listen, type UnlistenFn } from '@tauri-apps/api/event';
	import MediaThumbnail from './MediaThumbnail.svelte';
	import DbFileMissing from '../Shared/DbProblemDialog.svelte';
	import { commands, events, type GlobalConfig } from '$lib/tauri_bindings';
	import { SearchStore } from '../Sidebar/Search/SearchStore.svelte';
	import { InfiniteMediaStore } from './InfiniteMediaStore.svelte';
	import '../../fonts.css';
	import { MediaModalStatusStore } from '../MediaModal/MediaModalStatusStore.svelte';

	let values: Array<ImageRow> = $state([]);
	let tauri_width = $state(0); // TODO this should be set to initial window size
	let tauri_height = $state(0);
	let isDatabaseOk = $state(true); // set the true so it doesnt flash if the db does exist
	let reloadLayoutKey = $state(0);
	let problemText = $state('');
	let coolDown = $state(0);
	let config: GlobalConfig | undefined = $state();

	let virtualList: VirtualList | undefined = $state(undefined);

	let window_size_unlisten: UnlistenFn;

	// on cache update run updateLayout();

	$effect(() => {
		InfiniteMediaStore.showNames;
		console.log('setshownames updated');
		updateLayoutFromCache(false);
	});

	onMount(async () => {
		await InfiniteMediaStore.loadSettings();
		config = await commands.getConfig();

		events.databaseConnectionEvent.listen(async (e) => {
			switch (e.payload.type) {
				case 'RemoteConnected':
				case 'LocalConnected':
					isDatabaseOk = true;
					await commands.searchAndReload();
					break;
				case 'Uninitialize':
					isDatabaseOk = false;
					problemText = 'Create a new db';

					values = [];
					virtualList?.recomputeSizes(0);
					break;
				case 'Failed':
					isDatabaseOk = false;
					problemText = e.payload.data;
					break;
			}
		});

		// this has to come after the db event listener otherwise the first
		// load will fail
		await commands.connectToDbInConfig();

		// drag and drop support
		await listen('tauri://drag-drop', (event: any) => {
			// what is the type for the drag and drop event?
			const paths: Array<string> = event.event.paths;

			paths.forEach((path) => {
				commands.addIndexSource;
			});
		});

		await events.cacheUpdatedEvent.listen(async (e) => {
			console.log(`event is -> ${e} `);
			await updateLayoutFromCache(e.payload.reload_virtual_list);
			trace('cache_updated event received');
		});

		await events.openMediaModalEvent.listen((e) => {
			MediaModalStatusStore.open(e.payload.hash);
		});

		await events.closeMediaModalEvent.listen((e) => {
			MediaModalStatusStore.close();
		});

		const initial_size = await getCurrentWindow().innerSize();
		tauri_height = initial_size.height;
		tauri_width = initial_size.width;

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
		//clearTimeout(coolDown);
		coolDown = setTimeout(updateLayoutFromCache, 100);
	}

	function calculateRowHeight(height: number): number {
		if (InfiniteMediaStore.showNames) {
			return height + 30;
		} else {
			return height;
		}
	}

	function getItemSize(index: number): number {
		return calculateRowHeight(values[index].height);
	}

	/**
	 * Gets the media from the database possibly using cached values, sets the heights for the media and media themselves to
	 * the received values.
	 */
	async function updateLayoutFromCache(reloadVirtualList: boolean) {
		if (!InfiniteMediaStore.isLoaded || tauri_width <= 0) {
			return;
		}

		values =
			(await commands.getLayoutFromCache(
				tauri_width - sidebarStore.size * 3 - 20,
				12,
				InfiniteMediaStore.thumbnailScale!! // its better erroring out than reloading an whole new layout
			)) ?? [];

		if (values === null || values.length === 0) {
			error('Could not get layout from the rust cache');
		}

		virtualList?.recomputeSizes(0);

		if (reloadVirtualList) {
			reloadLayoutKey += 1;
		}

		trace(`calculating sizes w:${tauri_width}, h:${tauri_height}`);
		console.log(`calculating sizes w:${tauri_width} h:${tauri_height}`);
	}

	$effect(async () => {
		tauri_width;
		tauri_height;
		sidebarStore.isActive;

		await onResize();
	});
</script>

<div class="list">
	{#if isDatabaseOk && InfiniteMediaStore.isLoaded}
		{#key reloadLayoutKey}
			<VirtualList
				bind:this={virtualList}
				width="100%"
				height="100%"
				itemCount={values.length}
				itemSize={getItemSize}
			>
				{#snippet item({ style, index })}
					<div class="mediaRow" {style}>
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
				{/snippet}
			</VirtualList>
		{/key}
		<!--Avoid flashing the dialog for the config file load-->
	{:else if InfiniteMediaStore.isLoaded}
		<DbFileMissing {problemText} />
	{/if}
</div>

<style>
	.list {
		height: calc(100%); /* onscroll events won't fire without this one, why... ?*/
		position: relative;
	}

	.list :global(.virtual-list-wrapper) {
		overflow-x: hidden; /*Don't show horizontal scroll bar when moving the sidebar*/
	}
</style>
