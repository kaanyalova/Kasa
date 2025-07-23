<script lang="ts">
	import { invoke } from '@tauri-apps/api/core';
	import { MediaModalStatusStore } from '../MediaModalStatusStore.svelte';
	import { info, trace } from '@tauri-apps/plugin-log';
	import Pen from '../../Vector/Pen.svelte';
	import { onMount } from 'svelte';
	import Clipboard from '../../Vector/Clipboard.svelte';
	import { handleSelect } from '../../Sidebar/Search/HandleSelect';
	import { commands, type TagQueryOutput } from '$lib/tauri_bindings';
	import { getCursorPosition } from '$lib/getCaretPos';
	import TagDropDown from './TagDropDown.svelte';
	import Tag from './Tag.svelte';
	import NewTag from './NewTag/NewTagButton.svelte';
	import { comma } from 'postcss/lib/list';
	import type { CursorPosition, TagDisplayProps } from './TagDisplay';
	import { writeText } from '@tauri-apps/plugin-clipboard-manager';
	import NewTagButton from './NewTag/NewTagButton.svelte';
	let searchInput: HTMLDivElement;
	let { initialEditBoxContents, isInEditMode, updateTagsTextBoxContents, data }: TagDisplayProps =
		$props();
	let tagsTextLocal: string | null | undefined = $state();

	let tags = $state(data.tags);
	let sourceCategoryGroupedTags = $state(data.sourceCategoryGroupedTags);

	let cursorPosition: CursorPosition = $state({ top: null, left: null });
	let isTextBoxFocused = $state(false);
	let keyboardSelectedIndex = $state(-1);
	let entriesToShow: Array<TagQueryOutput> = $state([]);
	let cooldown = $state(0);
	let shouldShow = $state(false);

	$effect(() => {
		updateTagsTextBoxContents(tagsTextLocal);
	});

	onMount(() => {
		// Enable the edit mode if there is no tags present
		// It doesn't make sense to have an empty tags header
		if (data.tags.length === 0) {
			MediaModalStatusStore.setTagsEditModeActive(true);
		}
	});

	async function refreshTags() {
		const newTags = await commands.getTags(data.hash);
		const newGroupedTags = await commands.getTagsGroupedBySourceCategories(data.hash);

		if (newTags !== null) {
			tags = newTags;
		}

		if (newGroupedTags !== null) {
			sourceCategoryGroupedTags = newGroupedTags;
		}
	}

	async function toggleEditMode() {
		const editModeState = MediaModalStatusStore.tagsEditModeActive;

		// on editMode -> viewMode, save the tags
		if (editModeState === true) {
			info('updating tags');
			commands.updateTags(tagsTextLocal!!, data.hash);
		}

		// on viewMode -> editMode, update the tagText
		if (editModeState === false) {
			const tagsAsText = await commands.getTagsAsText(data.hash);

			if (tagsAsText !== null) {
				tagsTextLocal = tagsAsText;
			}
		}

		await refreshTags();

		MediaModalStatusStore.setTagsEditModeActive(!editModeState);
	}

	function handleTagSearch() {
		if (tagsTextLocal!!.length > 0) {
			clearTimeout(cooldown);
			cooldown = setTimeout(getDropdownEntriesFromDB, 200);
		} else {
			entriesToShow = [];
		}
	}

	async function getDropdownEntriesFromDB() {
		const searchContentsSplit = tagsTextLocal!!.split(',');
		const lastEntry = searchContentsSplit[searchContentsSplit.length - 1].trim();

		info(`${lastEntry} le`);

		// If the last entry is empty, don't show anything
		if (lastEntry.length === 0) {
			entriesToShow = [];
			return;
		}
		const entries: Array<TagQueryOutput> = await commands.queryTags(lastEntry, 10);

		if (entries.length > 0) {
			shouldShow = true;
		}

		entriesToShow = entries;
	}

	function setCursorToEnd() {
		if (searchInput) {
			const length = searchInput.textContent!!.length;
			searchInput.focus();

			// Create a new range and set it to the end of the content
			const range = document.createRange();
			range.selectNodeContents(searchInput);
			range.collapse(false);

			// Apply the range to the selection
			const selection = window.getSelection();
			selection!!.removeAllRanges();
			selection!!.addRange(range);
		} else {
			console.error('searchInput is null');
		}
	}

	async function onDeleteTag(name: string) {
		await commands.deleteTags(data.hash, [name]);
		trace('delte tags');
		await refreshTags();
	}

	async function onCopyTags() {
		const tags = await commands.getTagsAsText(data.hash);
		console.log(tags);
		if (tags !== null) {
			writeText(tags);
		}
	}
</script>

{#if cursorPosition.top !== null && cursorPosition.left !== null && isTextBoxFocused && shouldShow && entriesToShow.length > 0}
	<TagDropDown
		top={cursorPosition.top + 24}
		left={cursorPosition.left}
		selectedIndex={keyboardSelectedIndex}
		tags={entriesToShow}
		onTagClick={(index) => {
			tagsTextLocal = handleSelect(entriesToShow[index].name, tagsTextLocal as string);
			keyboardSelectedIndex = -1;
			shouldShow = false;

			// shitty hack, updating the textarea moves the cursor to the start for some reason
			setTimeout(() => {
				setCursorToEnd();
			}, 50);
		}}
	></TagDropDown>
{/if}

<div class="titleRow">
	<div class="title">Tags</div>

	<button
		class="titleButton"
		onclick={async () => toggleEditMode()}
		title="Edit tags"
		class:titleButtonActive={isInEditMode}
	>
		<Pen height={12} width={12}></Pen>
	</button>

	<button
		class="titleButton"
		onclick={async () => {
			await onCopyTags();
		}}
		title="Copy Tags"
	>
		<Clipboard height={12} width={12}></Clipboard>
	</button>
</div>

{#if isInEditMode}
	<div
		contenteditable="true"
		class="tagsEdit"
		role="textbox"
		bind:textContent={tagsTextLocal}
		onmousedown={() => {
			setTimeout(() => {
				const tagsEditElement = document.querySelector('.tagsEdit');
				if (tagsEditElement) {
					const position = getCursorPosition();
					if (position) {
						cursorPosition = position;
					}
				}
			}, 0);
		}}
		tabindex="0"
		onfocus={() => {
			isTextBoxFocused = true;
		}}
		onblur={() => {
			isTextBoxFocused = false;
		}}
		onkeydown={(event) => {
			switch (event.key) {
				case 'ArrowUp':
					if (keyboardSelectedIndex >= 0) {
						keyboardSelectedIndex -= 1;

						console.log(`index minus one, current index ${keyboardSelectedIndex}`);
					}
					break;

				case 'ArrowDown':
					if (keyboardSelectedIndex === -1) {
						keyboardSelectedIndex = 0;
						console.log(`initial current index ${keyboardSelectedIndex}`);
					} else if (keyboardSelectedIndex === entriesToShow.length - 1) {
						return;
					} else if (keyboardSelectedIndex < entriesToShow.length - 1) {
						keyboardSelectedIndex += 1;
						console.log(`index plus one, current index ${keyboardSelectedIndex}`);
					}
					break;

				case 'Enter':
					if (keyboardSelectedIndex >= 0) {
						console.log(`selected with keyboard ,index: ${keyboardSelectedIndex}`);
						event.preventDefault();
						tagsTextLocal = handleSelect(
							entriesToShow[keyboardSelectedIndex].name,
							tagsTextLocal as string
						);
						keyboardSelectedIndex = -1;
						shouldShow = false;
						// shitty hack
						setTimeout(() => {
							setCursorToEnd();
						});
					} else {
						// no index selected user should want to search stuff
						//onSearch();
					}
					break;
			}
		}}
		oninput={() => handleTagSearch()}
		bind:this={searchInput}
	>
		{initialEditBoxContents}
	</div>
{:else}
	{#each Object.entries(sourceCategoryGroupedTags.source_categories).sort() as [category, tagsWithCategory]}
		<h3>{category}</h3>
		<div class="tagsList">
			{#each tagsWithCategory as tagWithCategory}
				<Tag name={tagWithCategory.tag_name} onDelete={async (name: string) => onDeleteTag(name)}
				></Tag>
			{/each}

			<!--
			Not sure how intuative is this compared to the edit mode, don't show it for now

			<NewTagButton></NewTagButton>
			-->
		</div>
	{/each}

	<!-- Only show the Uncategorized category if there is elements inside of it -->
	{#if sourceCategoryGroupedTags.uncategorized.length > 0}
		<h3>Uncategorized</h3>

		<div class="tagsList">
			{#each sourceCategoryGroupedTags.uncategorized as uncategorizedTag}
				<!--Check if the tag exists in the main tag array, only display it if it does-->
				<Tag
					name={uncategorizedTag.tag_name}
					onDelete={async (name: string) => await onDeleteTag(name)}
				></Tag>
			{/each}
			<NewTagButton></NewTagButton>
		</div>
	{/if}

	{#if sourceCategoryGroupedTags.uncategorized.length === 0 && Object.keys(sourceCategoryGroupedTags.source_categories).length === 0}
		<div class="tagsDisplay">
			<NewTagButton></NewTagButton>
		</div>
	{/if}
{/if}

<style>
	.tagsList {
		padding: 4px;
		margin: 4px;
		border: var(--secondary-alt) 1px solid;
		overflow-y: auto;
		display: flex;
		flex-wrap: wrap;
	}

	.tagsEdit {
		background-color: var(--background);
		border: var(--secondary-alt) 1px solid;
		border-radius: 2px;
		font-size: small;
		resize: none;
		margin: 4px;
		padding-left: 4px;
		padding-right: 4px;
		overflow-y: auto;
	}

	.tagsEdit:focus {
		outline: none;
		border: solid 1px var(--accent);
	}

	.tagsDisplay {
		margin: 2px;
		font-size: small;
	}

	.header {
		padding-left: 2px;
		color: var(--text);
		display: flex;
		flex-direction: row;
		align-items: center;
	}

	.titleButton {
		fill: var(--text);
		padding: 3px;
		display: flex;
		justify-content: center;
		align-items: center;
		margin: 2px;
		border-radius: 2px;
		border: solid 1px var(--secondary-alt);
		left: 4px;
		position: relative;
	}

	.titleButton:hover {
		background: var(--secondary-alt);
	}

	.titleButtonActive {
		background: var(--secondary-alt);
	}

	.separator {
		height: 10px;
	}

	.titleRow {
		display: flex;
		flex-direction: row;
	}

	.autoCompleteBox {
		position: fixed;
		height: 50px;
		width: 200px;
		background-color: red;
	}

	.title {
		padding-left: 4px;
		font-weight: bold;
		font-size: large;
	}

	h3 {
		margin: 4px;
	}
</style>
