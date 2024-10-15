<script lang="ts">
	import { convertFileSrc, invoke } from '@tauri-apps/api/core';
	import '../../fonts.css';
	import { info } from '@tauri-apps/plugin-log';
	import clickOutside from '$lib/clickOutside';
	import Sidebar from './Sidebar.svelte';
	import { MediaModalStatusStore } from './MediaModalStatusStore.svelte';
	import { onMount } from 'svelte';
	import { commands } from '$lib/tauri_bindings';
	import 'vidstack/bundle';
	import { MediaPlayer } from 'vidstack';
	import { stat } from '@tauri-apps/plugin-fs';

	let { imageHash }: MediaModalProps = $props();

	let isOpen = $state(true);
	let tagsTextBoxContents = $state('');

	function updateTagsTextBoxContents(text: string) {
		tagsTextBoxContents = text;
	}

	async function getData(): Promise<MediaInfo> {
		const info: MediaInfo = await invoke('get_info', {
			hash: imageHash
		});

		return info;
	}

	onMount(async () => {
		// we want to get the media server up as soon as possible, parsing all the meta can take a while so
		// we have a function that just gets the type
		const mediaType = await commands.getMediaType(imageHash);

		if (mediaType === 'Video') {
			await commands.serveMedia(imageHash);
		}
		//await commands.serveMedia(imageHash);
	});

	async function onClose() {
		if (MediaModalStatusStore.tagsEditModeActive) {
			await invoke('update_tags', {
				rawInput: tagsTextBoxContents,
				hash: imageHash
			});
		} else {
		}
		commands.closeServer();
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
				{#if data!.mediaType == 'Image'}
					<img
						src={convertFileSrc(data!.paths[0])}
						alt="An image provided by user"
						style="aspect-ratio: {data!.aspectRatio};"
					/>
				{:else if data!.mediaType == 'Video'}
					<!--There is a slight pop-in when videos are first loaded-->

					<media-player
						autoplay
						controlsDelay={1000}
						title={data!.fileName}
						class="mediaPlayer"
						style="aspect-ratio: {data!.aspectRatio};"
					>
						<media-provider>
							<!--
							Video player refuses video/x-matkroska wtf, but video/webm works for all videos
							https://stackoverflow.com/questions/17018119/how-to-play-mkv-file-in-browser 
							-->
							<source type="video/webm" src="http://localhost:3169" />
						</media-provider>
						<media-video-layout></media-video-layout>
					</media-player>

					<!--
					<video src="http://localhost:3169" controls></video>
					-->
				{/if}
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

	.mediaPlayer {
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
