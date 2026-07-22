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
	import { commands, type GlobalConfig } from '$lib/tauri_bindings';
	import { SearchStore } from '../Sidebar/Search/SearchStore.svelte';
	import { InfiniteMediaStore } from './InfiniteMediaStore.svelte';
	import { comma } from 'postcss/lib/list';
	import '../../fonts.css';
	import { onNewDb, onOpenDb } from '$lib/dbSelection';

	let values: Array<ImageRow> = $state([]);
	let heights: Array<number> = $state([]);
	let tauri_width = $state(0); // TODO this should be set to initial window size
	let tauri_height = $state(0);
	let isDatabaseOk = $state(true); // set the true so it doesnt flash if the db does exist
	let config: GlobalConfig | null = $state(null);

	let cooldown = $state(0);

	let virtualList: any;

	let window_size_unlisten: UnlistenFn;

	// on cache update run updateLayout();
	listen('cache_updated', async (_) => {
		await updateLayoutFromCache();
		trace('cache_updated event received');
	});

	listen('media_updated', async (_) => {
		await initializeLayout();
		trace('media_updated event received');
	});

	$effect(() => {
		InfiniteMediaStore.showNames;
		console.log('setshownames updated');
		updateLayoutFromCache();
	});
	onMount(async () => {
		console.log('onmount');
		await listen('dbs_updated', async (e) => {
			const createNewDb = (e.payload as any).newDb as boolean;
			const doesTheDbFileExist = await commands.doesTheDbFileExist();

			// Show the files only if the database file actually exists or
			// the user is creating a new database,
			// TODO: i should also check for migrations
			isDatabaseOk = doesTheDbFileExist || createNewDb;

			info(`dbs_updated fileExists: ${isDatabaseOk}, newDb: ${createNewDb}`);
			console.log(
				`dbs_updated fileExists: ${doesTheDbFileExist}, newDb: ${createNewDb}, isDbOk:${isDatabaseOk}`
			);

			if (!isDatabaseOk) {
				return;
			}

			console.log();
			await commands.connectDbs();
			await initializeLayout();
			await emit('tags_updated');
			values = values;
			trace('dbs_updated event received');
		});

		// drag and drop support
		await listen('tauri://drag-drop', (event: any) => {
			// what is the type for the drag and drop event?
			const paths: Array<string> = event.event.paths;

			paths.forEach((path) => {
				commands.addIndexSource;
			});
		});

		config = await commands.getConfig();
		// check if the db actually exists first, prompt to user to create/select a database that exists if it doesn't
		isDatabaseOk = await commands.doesTheDbFileExist();

		const initial_size = await getCurrentWindow().innerSize();
		tauri_height = initial_size.height;
		tauri_width = initial_size.width;
		// https://v2.tauri.app/reference/javascript/api/namespacewindow/#onresized
		window_size_unlisten = await getCurrentWindow().onResized(({ payload: size }) => {
			tauri_height = size.height;
			tauri_width = size.width;
		});

		if (!isDatabaseOk) {
			console.log('the db file doesnt exist ');
			return;
		}

		console.log('called connect_dbs()');
		await commands.connectDbs();
		await InfiniteMediaStore.loadSettings();
		await initializeLayout();
	});
	//

	onDestroy(() => {
		trace('ondestroy called!');
		window_size_unlisten();
	});

	async function onResize() {
		cooldown = setTimeout(updateLayoutFromCache, 100);
	}

	function calculateRowHeight(height: number): number {
		if (InfiniteMediaStore.showNames) {
			return height + 30;
		} else {
			return height;
		}
	}

	/**
	 * Gets the media from the database possibly using cached values, sets the heights for the media and media themselves to
	 * the received values.
	 */
	async function updateLayoutFromCache() {
		let _values = await commands.getLayoutFromCache(
			tauri_width - sidebarStore.size * 3 - 20,
			12,
			InfiniteMediaStore.thumbnailScale
		);

		if (_values === null) {
			error('Could not get layout from the rust cache');
			return;
		}

		const _heights: Array<number> = _values.map((row) => {
			// first row should have the gaps height
			return calculateRowHeight(row.height);
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
				trace('search via initialize layout');
				await commands.search(SearchStore.searchContents);
			} else {
				setTimeout(initializeLayout, 500);
			}
		} catch (error) {
			// If there's an error, try again after a delay
			setTimeout(initializeLayout, 500);
		}
	}

	$effect(async () => {
		tauri_width;
		tauri_height;
		sidebarStore.isActive;

		await onResize();
	});

	$effect(async () => {});
</script>

<!-- TODO  overscanCount *WILL* cause problems on larger screens, change that accordingly -->
<div class="list">
	{#if isDatabaseOk}
		<VirtualList
			height="100%"
			width="100%"
			itemSize={heights}
			itemCount={values.length}
			overscanCount={Math.round(8 * InfiniteMediaStore.thumbnailScale)}
			bind:this={virtualList}
		>
			<div class="mediaRow" slot="item" let:index let:style {style}>
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
	{:else}
		<div class="dbFileMissingWrapper">
			<div class="dbFileMissingContainer">
				<div class="dbFileMissingText">
					The selected database does not exist in
					<span class="filePath">
						{config?.Database.db_path}
					</span>
				</div>

				<div class="dbMissingButtonRow">
					<button
						class="dbMissingButton"
						onclick={async () => {
							await onNewDb();
						}}>New DB</button
					>
					<button
						class="dbMissingButton"
						onclick={async () => {
							await onOpenDb();
						}}>Open DB</button
					>
				</div>
			</div>
		</div>
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

	.dbFileMissingText {
		color: var(--text);

		font-size: 20px;
	}

	.filePath {
		font-family: 'UbuntuMono';
		border: 1px solid var(--secondary-alt);
		padding: 2px;
		margin-left: 4px;
		margin-right: 4px;
	}

	.dbFileMissingWrapper {
		display: flex;
		flex-grow: 1;
		align-items: center;
		justify-content: center;
		height: calc(100% - 32px);
		flex-direction: column;
	}

	.dbMissingButton {
		background-color: var(--accent);
		border: 1px solid var(--accent-border);
		color: var(--text-opposite);
		padding: 8px;
		margin-top: 16px;
		margin-left: 4px;
		margin-right: 4px;
		font-weight: bold;
		border-radius: 4px;
	}
	.dbMissingButton:hover {
		background-color: var(--accent-hover);
	}

	.dbFileMissingContainer {
		display: flex;
		align-items: center;
		justify-self: center;
		flex-direction: column;
		border: 1px solid var(--secondary-alt);
		padding: 32px;
	}
</style>
