<script lang="ts">
	import { onDestroy, onMount } from 'svelte';
	import BorderedBox from '../../Shared/BorderedBox.svelte';
	import { commands, events } from '$lib/tauri_bindings';
	import { stat } from '@tauri-apps/plugin-fs';
	import type { TagWithCount, TagWithDetails } from '$lib/tauri_bindings';
	import { formatCount, getCountColor } from '$lib/colorUtils';
	import TagPickerEntry from './TagPickerEntry.svelte';
	import VirtualList from 'svelte-tiny-virtual-list';
	import { error, trace } from '@tauri-apps/plugin-log';
	import { SearchStore } from '../Search/SearchStore.svelte';
	import { emit, listen } from '@tauri-apps/api/event';
	import { SvelteMap } from 'svelte/reactivity';
	import TagPresets from './TagPresets.svelte';
	import { sidebarStore } from '../SidebarStore.svelte';

	let tags: Array<TagWithCount> | undefined | null = $state();
	let checkedTags: Map<string, TagPickerCheckboxState> = $state(new SvelteMap());
	let filterInput = $state('');
	let textWidthCanvas: CanvasRenderingContext2D | null;
	let filterFavorites = $state(false);

	async function toggleFavorites() {
		filterFavorites = !filterFavorites;
		// set actual filtering

		await commands.setSearchCriteria({
			contains_tags: Array.from(checkedTags.entries())
				.filter(([_tag, state]) => state === 'selected')
				.map(([tag, _state]) => tag),
			contains_tags_or_group: [],
			excludes_tags: Array.from(checkedTags.entries())
				.filter(([_tag, state]) => state === 'exclude')
				.map(([tag, _state]) => tag),
			order_by_date: 'NewestFirst',
			order_by_resolution: 'None',
			date_range: null,
			favorites_only: filterFavorites
		});

		await commands.searchAndReload();
	}

	async function onCheck(state: TagPickerCheckboxState, tagName: string) {
		if (state === 'unselected') {
			checkedTags.delete(tagName);
		} else {
			checkedTags.set(tagName, state);
		}
		await commands.setSearchCriteria({
			contains_tags: Array.from(checkedTags.entries())
				.filter(([_tag, state]) => state === 'selected')
				.map(([tag, _state]) => tag),
			contains_tags_or_group: [],
			excludes_tags: Array.from(checkedTags.entries())
				.filter(([_tag, state]) => state === 'exclude')
				.map(([tag, _state]) => tag),
			order_by_date: 'NewestFirst',
			order_by_resolution: 'None',
			date_range: null,
			favorites_only: filterFavorites
		});

		trace('search via tag picker check');
		await commands.searchAndReload();
	}

	let filteredTags: Array<TagWithCount> = $derived(
		tags?.filter((tag) => {
			return tag.tag_name.includes(filterInput);
		}) ?? []
	);

	async function loadTags() {
		tags = await commands.getListOfAllTagsWithDetails('TagCount');
		trace('load tags');
		console.log('loading tags');
	}

	$effect(async () => {
		sidebarStore.isActive;

		if (sidebarStore.isActive) {
			await loadTags();
		}
	});

	// Prepare the canvas for text width calculations
	onMount(async () => {
		await events.tagsUpdatedEvent.listen(async (_) => {
			trace('tags_updated emitted');
			await loadTags();
		});

		await events.databaseConnectionEvent.listen(async (e) => {
			switch (e.payload.type) {
				case 'RemoteConnected':
				case 'LocalConnected':
					await loadTags();
					break;
			}
		});

		const canvas = document.createElement('canvas');
		textWidthCanvas = canvas.getContext('2d');

		const body = document.body;
		const style = window.getComputedStyle(body);

		const fontWeight = style.getPropertyValue('font-weight');
		const fontSize = style.getPropertyValue('font-size');
		const fontFamily = style.getPropertyValue('font-family');

		const font = `${fontWeight} ${fontSize} ${fontFamily}`.trim() || '14px sans-serif';

		if (textWidthCanvas) {
			textWidthCanvas.font = font;
		} else {
			error('Cannot create canvas for tag picker text width calculations');
		}
	});

	onDestroy(() => {
		//const canvas = document.getElementsByTagName('canvas')[0];
		//canvas.remove();
	});

	function measureTextWidth(text: string): number | undefined {
		if (textWidthCanvas === null) {
			return undefined;
		}

		return textWidthCanvas.measureText(text).width;
	}

	const CONTAINER_WIDTH = 150;
	const LINE_HEIGHT = 26;
	const VERTICAL_PADDING = 4;

	function calculateHeight(index: number): number {
		const tag = filteredTags[index];

		if (tag === undefined) {
			return LINE_HEIGHT + VERTICAL_PADDING;
		}

		const width = measureTextWidth(tag.tag_name) ?? 0;
		const lines = Math.max(1, Math.ceil(width / CONTAINER_WIDTH));
		return lines * LINE_HEIGHT + VERTICAL_PADDING;
	}

	async function resetTags() {
		checkedTags.clear();
		filterInput = '';
		await commands.setSearchCriteria({
			contains_tags: [],
			contains_tags_or_group: [],
			excludes_tags: [],
			order_by_date: 'NewestFirst',
			order_by_resolution: 'None',
			date_range: null,
			favorites_only: false
		});

		SearchStore.searchContents = '';
		await commands.setSearchInput('');
		filterFavorites = false;

		trace('search via tag picker reset');
		await commands.searchAndReload();
	}
</script>

<div class="tagPicker">
	<div class="tagPickerList">
		{#if filteredTags.length > 0}
			{#key filteredTags}
				<VirtualList height={500} itemCount={filteredTags.length} itemSize={calculateHeight}>
					{#snippet item({ style, index })}
						<TagPickerEntry
							tagName={filteredTags!![index].tag_name}
							count={filteredTags!![index].count}
							checkboxState={checkedTags.get(filteredTags!![index].tag_name) ?? 'unselected'}
							{onCheck}
							{style}
						/>
					{/snippet}
				</VirtualList>
			{/key}
		{/if}
	</div>

	<div class="search">
		<div class="searchUpper">
			<div class="searchLabel">Search Tags</div>
			<button class="resetFilter" onclick={async () => await resetTags()}>
				Clear
				<span class="icon-[material-symbols--filter-alt-off] w-5 h-5 pl-2"></span>
			</button>
		</div>

		<div class="tagFilter">
			<div class="tagFilterIconContainer">
				<span class="icon-[material-symbols--label-outline]"></span>
			</div>
			<input type="text" bind:value={filterInput} class="tagFilterBar" />
		</div>

		<button
			class="favoritesButton"
			class:favoritesButtonActive={filterFavorites}
			onclick={async () => await toggleFavorites()}
		>
			{#if filterFavorites}
				<span class="icon-[material-symbols--favorite] w-5 h-5 bg-[#f14c45] mr-1"></span>
			{:else}
				<span class="icon-[material-symbols--favorite-outline] w-5 h-5 mr-1"></span>
			{/if}
			Favorites
		</button>
	</div>

	<div class="tagPresets">
		<div class="tagPresetsText">Tag Presets</div>
		<div class="tagPresetsButtonContainer">
			<TagPresets></TagPresets>
		</div>
	</div>
</div>

<style>
	.tagPickerList {
		display: flex;
		flex-grow: 1;
		margin: 8px;
		margin-bottom: 4px;
		padding: 4px;
		flex-direction: column;
		color: var(--text);
		height: 500px;
		border: 1px solid var(--secondary-alt);
		width: 274px;
		user-select: none;
		-webkit-user-select: none;
	}

	.tagPickerList :global(.virtual-list-wrapper) {
		padding-right: 18px;
	}

	.search {
		display: flex;
		justify-content: center;
		padding-left: 20px;
		padding-right: 20px;
		padding: 4px;
		padding-right: 8px;
		padding-left: 12px;
		flex-direction: column;
		color: var(--text);
		margin-bottom: 4px;
	}

	.tagFilterBar {
		width: 100%;
		background-color: var(--secondary-alt);
		outline: 1px solid var(--border-on-secondary-alt);
		padding: 2px;
		padding-left: 8px;
		padding-right: 8px;
		height: 30px;
	}

	.tagFilterBar:focus {
		outline: var(--accent) 1px solid;
	}

	.tagFilter {
		display: flex;
	}

	.tagFilterIconContainer {
		background-color: var(--accent);
		width: 30px;
		height: 30px;
		aspect-ratio: 1/1;
		outline: 1px solid var(--accent-border);
		display: flex;
		justify-content: center;
		align-items: center;
		color: var(--text-opposite);
	}

	.searchUpper {
		display: flex;
		flex-direction: row;
		justify-content: space-between;
	}

	.resetFilter {
		color: var(--text);
		background-color: var(--background);
		border-radius: 8px;
		border: 1px solid var(--secondary-alt);
		padding: 2px;
		margin-bottom: 4px;
		fill: var(--text);
		display: flex;
		flex-direction: row;
		padding-left: 4px;
		padding-right: 4px;
		display: flex;
		align-items: center;
		justify-content: center;
	}
	.resetFilter:hover {
		background-color: var(--secondary-alt);
	}

	.searchLabel {
		display: inline-block;
		align-self: flex-end;
	}
	.tagPicker {
		border: 1px solid var(--secondary-alt);
		margin: 4px;
	}

	.tagPresetsButtonContainer {
		display: flex;
	}

	.tagPresets {
		border: 1px solid var(--secondary-alt);
		margin: 0px 8px 8px 8px;
		color: var(--text);
	}

	.tagPresetsText {
		padding-left: 4px;
	}

	.favoritesButton {
		border: 1px solid var(--secondary-alt);
		padding: 2px;
		margin: 4px;
		border-radius: 4px;
		margin-top: 8px;
		text-align: center;
		cursor: pointer;
		display: flex;
		align-items: center;
		justify-content: center;
	}

	.favoritesButton:hover {
		background-color: var(--secondary-alt);
	}
	.favoritesButtonActive {
		background-color: var(--secondary-alt);
	}
</style>
