<script lang="ts">
	// This should "technically work" but

	import { invoke } from '@tauri-apps/api/core';
	import { info } from '@tauri-apps/plugin-log';
	import { onDestroy, onMount, tick } from 'svelte';
	import VirtualList, { type VirtualListEvents } from 'svelte-tiny-virtual-list';
	import { sidebarStore } from '../Sidebar/SidebarStore.svelte';
	import { appWindow } from '../Decoration/utils/window';
	import { getCurrentWindow, PhysicalSize } from '@tauri-apps/api/window';
	import type { UnlistenFn } from '@tauri-apps/api/event';
	import Image from './Image.svelte';

	let values: Array<ImageRow> = $state([]);
	let heights: Array<number> = $state([]);
	let tauri_width = $state(0); // TODO this should be set to initial window size
	let tauri_height = $state(0);

	let is_db_mounted = $state(false);
	let cooldown = $state(0);

	let virtualList: any;

	let window_size_unlisten: UnlistenFn;

	onMount(async () => {
		const initial_size = await getCurrentWindow().innerSize();
		tauri_height = initial_size.height;
		tauri_width = initial_size.width;
		// https://v2.tauri.app/reference/javascript/api/namespacewindow/#onresized
		window_size_unlisten = await getCurrentWindow().onResized(({ payload: size }) => {
			// Rounded because floating window sizes causes it to break
			//tauri_height = Math.round(size.height);
			//tauri_width = Math.round(size.width);
			tauri_height = size.height;
			tauri_width = size.width;
		});
	});

	onDestroy(() => {
		window_size_unlisten();
	});

	async function onResize() {
		cooldown = setTimeout(updateLayout, 200);
	}

	async function updateLayout() {
		// webkit render bug?
		//values = [];

		info(`calculating sizes w:${tauri_width}`);

		const _values: Array<ImageRow> = await invoke('get_layout_from_cache', {
			width: tauri_width - sidebarStore.size * 3 - 10,
			imgHeight: tauri_height / 2,
			gaps: 12
		});

		const _heights: Array<number> = _values.map((row) => row.height);

		heights = _heights;
		values = _values;
	}

	$effect(async () => {
		tauri_width;
		tauri_height;
		sidebarStore.size;

		await onResize();
	});

	onMount(() => {
		const checkDatabase = async () => {
			try {
				const _values: Array<ImageRow> = await invoke('query_all', {
					width: tauri_width - sidebarStore.size * 3 - 10,
					imgHeight: 200,
					gaps: 12
				});

				if (!(await invoke('are_dbs_mounted'))) {
					/*(_values.length === 0)*/ setTimeout(checkDatabase, 500);
				} else {
					is_db_mounted = true;
					const _heights: Array<number> = _values.map((row) => row.height);

					values = _values;
					heights = _heights;
				}
			} catch (error) {
				console.error('Error querying database:', error);
				// If there's an error, try again after a delay
				setTimeout(checkDatabase, 500);
			}
		};

		checkDatabase();
		values = values;

		// Optional: Return a cleanup function if needed
		return () => {
			// Any cleanup code here
		};
	});
</script>

<!--
<div
	onresize={async () => {
		await onResize();
	}}
	class="infiniteMedia"
>
	{#each values as row}
		<div class="imageRow">
			{#each row.images as image}
				<Image
					height={image.height}
					width={image.width}
					hash={image.hash}
					offset_y={image.y_relative}
					offset_x={image.x_relative}
				></Image>
			{/each}
		</div>
	{/each}
</div>




-->

<!-- TODO  overscanCount *WILL* cause problems on larger screens, change that accordingly -->
<VirtualList
	width="100%"
	itemSize={heights}
	height="100%"
	itemCount={values.length}
	overscanCount={8}
	bind:this={virtualList}
>
	<div slot="item" let:index let:style {style}>
		{#each values[index].images as image}
			<Image
				isSelected={false}
				hash={image.hash}
				height={image.height}
				width={image.width}
				offset_x={image.x_relative}
				offset_y={image.y_relative}
			></Image>
		{/each}
	</div>
</VirtualList>

<style>
	.infiniteMedia {
		position: relative;
		user-select: none;
		-webkit-user-select: none;
	}
</style>
