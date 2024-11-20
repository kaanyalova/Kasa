<script lang="ts">
	import { invoke } from '@tauri-apps/api/core';
	import { MediaModalStatusStore } from './MediaModalStatusStore.svelte';
	import { info } from '@tauri-apps/plugin-log';
	import Pen from '../Vector/Pen.svelte';
	import { onMount } from 'svelte';
	import Clipboard from '../Vector/Clipboard.svelte';

	let { initialEditBoxContents, isInEditMode, updateTagsTextBoxContents, data }: TagDisplayProps =
		$props();
	let tagsTextLocal: string | null | undefined = $state();
	let tags = $state(data.tags);

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

	function copyTags() {}
</script>

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

	<button class="titleButton" onclick={() => copyTags()} title="Copy Tags">
		<Clipboard height={12} width={12}></Clipboard>
	</button>
</div>

{#if isInEditMode}
	<div contenteditable="true" class="tagsEdit" role="textbox" bind:textContent={tagsTextLocal}>
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
</style>
