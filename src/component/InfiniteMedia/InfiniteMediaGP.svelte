<script lang="ts">
	import { error, trace } from '@tauri-apps/plugin-log';
	import { onDestroy, onMount, untrack } from 'svelte';
	import VirtualList from 'svelte-tiny-virtual-list';
	import { sidebarStore } from '../Sidebar/SidebarStore.svelte';
	import { getCurrentWindow } from '@tauri-apps/api/window';
	import { listen, type UnlistenFn } from '@tauri-apps/api/event';
	import MediaThumbnail from './MediaThumbnail.svelte';
	import DbFileMissing from '../Shared/DbProblemDialog.svelte';
	import { commands, events } from '$lib/tauri_bindings';
	import { InfiniteMediaStore } from './InfiniteMediaStore.svelte';
	import '../../fonts.css';
	import { MediaModalStatusStore } from '../MediaModal/MediaModalStatusStore.svelte';
	import functionThrottle from '$lib/justThrottle';

	let values: Array<ImageRow> = $state([]);
	let width = $state(0); // TODO this should be set to initial window size
	let height = $state(0);
	let isDatabaseOk = $state(true); // set the true so it doesnt flash if the db does exist
	let reloadLayoutKey = $state(0);
	let problemText = $state('');

	let previousWidth = $state(0);
	let previousHeight = $state(0);

	let virtualList: VirtualList | undefined = $state(undefined);

	let windowSizeUnlisten: UnlistenFn;
	let sidebarResizeUnlisten: () => void | undefined;
	let layoutChangeUnlisten: () => void | undefined;

	const throttledResize = functionThrottle(
		() => {
			onResize();
		},
		250,
		{ leading: false, trailing: true }
	);

	onMount(async () => {
		await InfiniteMediaStore.loadSettings();

		// load the window sizes before the events are listened to not reload for an extra time when
		// this gets set
		const initial_size = await getCurrentWindow().innerSize();
		height = initial_size.height;
		width = initial_size.width;

		await events.databaseConnectionEvent.listen(async (e) => {
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

		// never loads.
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

		// this has to come after the db & cache updated event listeners
		// otherwise the first load will fail
		await commands.connectToDbInConfig();

		windowSizeUnlisten = await getCurrentWindow().onResized(({ payload: size }) => {
			previousHeight = height;
			previousWidth = width;

			// don't reload on same size window reloads (like minimizing the window etc.)
			if (previousHeight !== size.height || previousWidth !== size.width) {
				height = size.height;
				width = size.width;
				throttledResize();
			}
		});

		sidebarResizeUnlisten = sidebarStore.subscribeForResizes(() => {
			onResize();
		});

		layoutChangeUnlisten = InfiniteMediaStore.subscribeForLayoutChanges(() => {
			onResize();
		});
	});

	onDestroy(() => {
		trace('ondestroy called!');
		windowSizeUnlisten();
		sidebarResizeUnlisten?.();
		layoutChangeUnlisten?.();
	});

	async function onResize() {
		updateLayoutFromCache(false);
	}

	function calculateRowHeight(height: number): number {
		if (InfiniteMediaStore.getShowNames()) {
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
		if (!InfiniteMediaStore.getIsLoaded() || width <= 0) {
			return;
		}

		const layout = await commands.getLayoutFromCache(
			width - sidebarStore.size * 3 - 20,
			12,
			InfiniteMediaStore.getThumbnailScale()!! // its better erroring out than reloading an whole new layout
		);

		if (layout === null) {
			error('Could not get layout from the rust cache');
		}

		values = layout ?? [];

		virtualList?.recomputeSizes(0);

		if (reloadVirtualList) {
			reloadLayoutKey += 1;
		}

		trace(`calculating sizes w:${width}, h:${height}`);
		console.log(`calculating sizes w:${width} h:${height}`);
	}

	function estimateRowHeight(): number {
		console.log('estimate row height');
		const scale = InfiniteMediaStore.getThumbnailScale() ?? 1.5;
		const rowHeight = 200 / scale + 12;
		return calculateRowHeight(rowHeight);
	}
</script>

<div class="list">
	{#if isDatabaseOk && InfiniteMediaStore.getIsLoaded()}
		{#key reloadLayoutKey}
			<VirtualList
				bind:this={virtualList}
				width="100%"
				height="100%"
				itemCount={values.length}
				itemSize={getItemSize}
				estimatedItemSize={estimateRowHeight()}
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
	{:else if InfiniteMediaStore.getIsLoaded()}
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
