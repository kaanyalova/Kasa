<script lang="ts">
	import { onDestroy, onMount } from 'svelte';
	import BorderedBox from '../../Shared/BorderedBox.svelte';
	import { commands } from '$lib/tauri_bindings';
	import { stat } from '@tauri-apps/plugin-fs';
	import type { TagWithCount, TagWithDetails } from '$lib/tauri_bindings';
	import { formatCount, getCountColor } from '$lib/colorUtils';
	import TagPickerCheckBox from './TagPickerCheckBox.svelte';
	import VirtualList, { type VirtualListProps } from 'svelte-tiny-virtual-list';
	import { error, trace } from '@tauri-apps/plugin-log';
	import { comma } from 'postcss/lib/list';
	import { SearchStore } from '../Search/SearchStore.svelte';
	import { listen } from '@tauri-apps/api/event';
	import FilterAltOff from '../../Vector/FilterAltOff.svelte';
	import { SvelteMap } from 'svelte/reactivity';
	import TagPresets from './TagPresets.svelte';

	let tags: Array<TagWithCount> | undefined | null = $state();
	let checkedTags: Map<string, TagPickerCheckboxState> = $state(new SvelteMap());
	let filterInput = $state('');
	let textWidthCanvas: CanvasRenderingContext2D | null;
	let virtualList: VirtualList;

	async function onCheck(state: TagPickerCheckboxState, tagName: string) {
		if (state === 'unselected') {
			checkedTags.delete(tagName);
		} else {
			checkedTags.set(tagName, state);
		}
		await commands.setSearchStore({
			contains_tags: Array.from(checkedTags.entries())
				.filter(([_tag, state]) => state === 'selected')
				.map(([tag, _state]) => tag),
			contains_tags_or_group: [],
			excludes_tags: Array.from(checkedTags.entries())
				.filter(([_tag, state]) => state === 'exclude')
				.map(([tag, _state]) => tag),
			order_by: 'NewestFirst',
			date_range: null
		});

		trace('search via tag picker check');
		await commands.search(SearchStore.searchContents);
	}

	let filteredTags: Array<TagWithCount> = $derived(
		tags?.filter((tag) => {
			return tag.tag_name.startsWith(filterInput);
		}) ?? []
	);

	async function loadTags() {
		tags = await commands.getListOfAllTagsWithDetails('TagCount');
		trace('load tags');
	}

	let loadTagsPromise = $state(loadTags);

	// Try every one second until the tags are first loaded, the as db is usually not instantly mounted
	const initialLoadInterval = setInterval(() => {
		if (!tags) {
			loadTags();
		} else {
			clearInterval(initialLoadInterval);
		}
	}, 500);

	listen('tags_updated', (_) => {
		trace('tags_updated emitted');
		loadTags();
	});

	// Prepare the canvas for text width calculations
	onMount(() => {
		const canvas = document.createElement('canvas');
		textWidthCanvas = canvas.getContext('2d');

		const body = document.body;
		const style = window.getComputedStyle(body);

		const fontWeight = style.getPropertyValue('font-weight');
		const fontSize = style.getPropertyValue('font-size');
		const fontFamily = style.getPropertyValue('font-family');

		const font = `${fontWeight} ${fontSize} ${fontFamily}`;

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

	function getTextWidth(text: string): number | undefined {
		return textWidthCanvas?.measureText(text).width;
	}

	const TEXT_MAX_HEIGHT = 150;

	function calculateHeight(index: number): number {
		const tag = filteredTags[index].tag_name;
		const width = getTextWidth(tag);
		return Math.ceil((width ?? TEXT_MAX_HEIGHT) / TEXT_MAX_HEIGHT) * 26 + 4;
	}

	async function resetTags() {
		checkedTags.clear();
		filterInput = '';
		await commands.setSearchStore({
			contains_tags: [],
			contains_tags_or_group: [],
			excludes_tags: [],
			order_by: 'NewestFirst',
			date_range: null
		});

		trace('search via tag picker reset');
		await commands.search(SearchStore.searchContents);
	}
</script>

<div class="tagPicker">
	<div class="tagPickerList">
		{#if filteredTags.length > 0}
			<VirtualList
				height={500}
				itemCount={filteredTags.length}
				itemSize={calculateHeight}
				bind:this={virtualList}
				scrollToIndex={/*How does this even work?*/ 0}
			>
				<div slot="item" let:index let:style {style}>
					<div class="tag">
						<TagPickerCheckBox
							tagName={filteredTags!![index].tag_name}
							checkboxState={checkedTags.get(filteredTags!![index].tag_name) ?? 'unselected'}
							{onCheck}
						></TagPickerCheckBox>
						<label for="tag-{filteredTags!![index].tag_name}">
							<div class="tagName">
								{filteredTags!![index].tag_name}
							</div>
						</label>

						<div
							class="count"
							style="background-color: {getCountColor(filteredTags!![index].count)}"
						>
							{formatCount(filteredTags!![index].count)}
						</div>
					</div>
				</div>
			</VirtualList>
		{/if}
	</div>

	<div class="search">
		<div class="searchUpper">
			<div class="searchLabel">Search Tags</div>
			<button class="resetFilter" onclick={async () => await resetTags()}>
				Clear
				<FilterAltOff height={24} width={24}></FilterAltOff>
			</button>
		</div>
		<input type="text" bind:value={filterInput} />
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
		overflow-y: auto;
		border: 1px solid var(--secondary-alt);
		width: 274px;
		user-select: none;
		-webkit-user-select: none;
	}

	.tagPickerList :global(.virtual-list-wrapper) {
		padding-right: 18px;
	}

	.count {
		margin-left: auto;
		min-width: 50px;
		color: black;
		text-align: center;
		border-radius: 0px 10px 10px 0px;
	}

	.tag {
		display: flex;
		border: 1px solid var(--secondary-alt);
		margin: 2px;
		padding-left: 4px;
		border-radius: 0px 10px 10px 0px;
	}

	.tagName {
		overflow-wrap: break-word;
		word-break: break-all;
		margin-right: 8px;
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

	.search > input {
		width: 100%;
		background-color: var(--secondary-alt);
		padding: 4px;
		padding-left: 8px;
		padding-right: 8px;
		border-radius: 8px;
	}

	.search > input:focus {
		outline: var(--accent) 1px solid;
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

	.tagPresetButton {
		color: var(--text);
		flex: 1;
		margin: 8px;
		margin-top: 4px;
		border-radius: 8px;
		border: 1px solid var(--secondary-alt);
		font-size: 16px;
	}

	.tagPresetButton:hover {
		background-color: var(--secondary-alt);
	}

	.tagPresets {
		border: 1px solid var(--secondary-alt);
		margin: 0px 8px 8px 8px;
		color: var(--text);
	}

	.tagPresetsText {
		padding-left: 4px;
	}
</style>
