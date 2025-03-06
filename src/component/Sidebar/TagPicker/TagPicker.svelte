<script lang="ts">
	import { onDestroy, onMount } from 'svelte';
	import BorderedBox from '../../Shared/BorderedBox.svelte';
	import { commands } from '$lib/tauri_bindings';
	import { stat } from '@tauri-apps/plugin-fs';
	import type { TagWithCount, TagWithDetails } from '$lib/tauri_bindings';
	import { formatCount, getCountColor } from '$lib/colorUtils';
	import TagPickerCheckBox from './TagPickerCheckBox.svelte';
	import VirtualList, { type VirtualListProps } from 'svelte-tiny-virtual-list';
	import { error } from '@tauri-apps/plugin-log';

	let tags: Array<TagWithCount> | undefined | null = $state();
	let filterInput = $state('');
	let textWidthCanvas: CanvasRenderingContext2D | null;
	let virtualList: VirtualList;

	let filteredTags: Array<TagWithCount> = $derived(
		tags?.filter((tag) => {
			return tag.tag_name.startsWith(filterInput);
		}) ?? []
	);

	async function loadTags() {
		tags = await commands.getListOfAllTagsWithDetails('TagCount');
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
		const canvas = document.getElementsByTagName('canvas')[0];
		canvas.remove();
	});

	function getTextWidth(text: string): number | undefined {
		return textWidthCanvas?.measureText(text).width;
	}

	function calculateHeight(index: number): number {
		// width is 154 px
		const tag = filteredTags[index].tag_name;
		const width = getTextWidth(tag);
		console.log(width);
		return Math.ceil((width ?? 154) / 154) * 26 + 4;
		//return 26;
	}
</script>

<div class="tagPicker">
	<div class="tagPickerList">
		<VirtualList
			width="100%"
			height={500}
			itemCount={filteredTags.length}
			itemSize={calculateHeight}
			bind:this={virtualList}
			scrollToIndex={/*How does this even work?*/ 0}
		>
			<div slot="item" let:index let:style {style}>
				<div class="tag">
					<TagPickerCheckBox tagName={filteredTags!![index].tag_name}></TagPickerCheckBox>
					<label for="tag-{filteredTags!![index].tag_name}">
						<div class="tagName">
							{filteredTags!![index].tag_name}
						</div>
					</label>

					<div class="count" style="background-color: {getCountColor(filteredTags!![index].count)}">
						{formatCount(filteredTags!![index].count)}
					</div>
				</div>
			</div>
		</VirtualList>
	</div>

	<div class="search">
		Search Tags
		<input type="text" bind:value={filterInput} />
	</div>

	<div class="tagPresets">
		Tag Presets
		<div class="tagPresetsButtonContainer">
			<button class="tagPresetButton">Save</button>
			<button class="tagPresetButton">Load</button>
			<button class="tagPresetButton">Hello</button>
		</div>
	</div>
</div>

<style>
	.tagPickerList {
		display: flex;
		flex-grow: 1;
		margin: 8px;
		padding: 4px;
		padding-right: 18px;
		flex-direction: column;
		color: var(--text);
		height: 500px;
		overflow-y: auto;
		border: 1px solid var(--secondary-alt);
		width: 274px;
		user-select: none;
		-webkit-user-select: none;
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
		padding: 8px;
		padding-left: 12px;
		flex-direction: column;
		color: var(--text);
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
		padding-left: 4px;
	}
</style>
