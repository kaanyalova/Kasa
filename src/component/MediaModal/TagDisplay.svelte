<script lang="ts">
	import { invoke } from '@tauri-apps/api/core';
	import { MediaModalStatusStore } from './MediaModalStatusStore.svelte';
	import { info, trace } from '@tauri-apps/plugin-log';
	import Pen from '../Vector/Pen.svelte';
	import { onMount } from 'svelte';
	import Clipboard from '../Vector/Clipboard.svelte';
	import { handleSelect } from '../Sidebar/Search/HandleSelect';
	import { commands, type TagQueryOutput } from '$lib/tauri_bindings';
	import { getCursorPosition } from '$lib/getCaretPos';
	import TagDropDown from './TagDropDown.svelte';
	let searchInput: HTMLDivElement;
	let { initialEditBoxContents, isInEditMode, updateTagsTextBoxContents, data }: TagDisplayProps =
		$props();
	let tagsTextLocal: string | null | undefined = $state();
	let tags = $state(data.tags);

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

	async function toggleEditMode() {
		const editModeState = MediaModalStatusStore.tagsEditModeActive;

		// on editMode -> viewMode, save the tags
		if (editModeState === true) {
			info('updating tags');
			await invoke('update_tags', {
				rawInput: tagsTextLocal,
				hash: data.hash
			});
		}

		tags = await invoke('get_tags', {
			hash: data.hash
		});

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

	function updateTagsTextBoxContentsLocal(contents: string) {
		tagsTextLocal = contents;
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
	Tags

	<button
		class="titleButton"
		onclick={async () => toggleEditMode()}
		title="Edit tags"
		class:titleButtonActive={isInEditMode}
	>
		<Pen height={12} width={12}></Pen>
	</button>

	<button class="titleButton" onclick={() => {}} title="Copy Tags">
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
	<ul class="tagsList">
		{#each tags as tag}
			<li class="tagsDisplay">{tag.name}</li>
		{/each}
	</ul>
{/if}

<style>
	.tagsList {
		padding: 4px;
		margin: 4px;
		border: var(--secondary-alt) 1px solid;
		max-height: 30vh;
		overflow-y: auto;
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
		max-height: 30vh;
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
</style>
