<script lang="ts">
	import { convertFileSrc, invoke } from '@tauri-apps/api/core';
	import '../../fonts.css';
	import { info } from '@tauri-apps/plugin-log';
	import clickOutside from '$lib/clickOutside';
	import Sidebar from './Sidebar.svelte';
	import { MediaModalStatusStore } from './MediaModalStatusStore.svelte';

	let { imageHash }: MediaModalProps = $props();

	let isOpen = $state(true);
	let tagsTextBoxContents = $state('');

	function updateTagsTextBoxContents(text: string) {
		tagsTextBoxContents = text;
	}

	async function getData(): Promise<MediaInfo> {
		return await invoke('get_info', {
			hash: imageHash
		});
	}

	async function reloadData() {}

	async function onClose() {
		if (MediaModalStatusStore.tagsEditModeActive) {
			await invoke('update_tags', {
				rawInput: tagsTextBoxContents,
				hash: imageHash
			});
		} else {
		}

		MediaModalStatusStore.close();
	}
</script>

<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- TODO keyboard navigation -->
<dialog open={MediaModalStatusStore.isOpen}>
	{#await getData() then data}
		<div
			class="dialogContents"
			use:clickOutside={async () => {
				await onClose();
				info('close!');
			}}
		>
			<div class="imageWrapper">
				<!-- svelte-ignore a11y_img_redundant_alt -->
				<img src={convertFileSrc(data.paths[0])} alt="An image provided by user" />
			</div>

			<Sidebar {data} {updateTagsTextBoxContents}></Sidebar>
		</div>
	{/await}
</dialog>

<style>
	dialog {
		position: fixed;
		top: 0;
		right: 0;
		bottom: 0;
		left: 0;
		width: 100%;
		height: calc(100%);
		flex-direction: row;
		display: flex;
		background: rgba(0, 0, 0, 0.7);
		z-index: 2;
		align-items: center;
		justify-content: center;
	}
	img {
		height: 100%;
		object-fit: contain;
	}

	.imageWrapper {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		border: solid 1px var(--secondary-alt);

		/* Images take up at least 60vw of horizontal space*/
		/*
		width: 60vw;
		max-height: 80vh;
		*
		/*  
		Alternative ,images take up minimum horizontal space 
		*/
		height: 80vh;
		max-width: 60vw;
		flex-direction: row-reverse;
	}

	.dialogContents {
		display: flex;
		background-color: var(--background);
		position: relative;
		left: calc(var(--sidebar-width) / 2);
		min-height: 80vh;
		border-radius: 0px 10px 10px 0px;

		/*
		box-shadow:  TODO looks bad 
			rgba(60, 60, 60, 0.25) 0px 14px 28px,
			rgba(60, 60, 60, 0.22) 0px 10px 10px;

		*/
	}
	svg {
		display: block;
		margin: auto;
	}
</style>
