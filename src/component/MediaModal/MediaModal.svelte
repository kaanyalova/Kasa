<script lang="ts">
	import { convertFileSrc, invoke } from '@tauri-apps/api/core';
	import '../../fonts.css';
	import { info, trace } from '@tauri-apps/plugin-log';
	import {
		clickOutsideMediaModal,
		clickOutsideModal,
		clickOutsideTagName
	} from '$lib/clickOutside';
	import Sidebar from './Sidebar.svelte';
	import { MediaModalStatusStore } from './MediaModalStatusStore.svelte';
	import { onMount } from 'svelte';
	import { commands, type MediaInfo } from '$lib/tauri_bindings';
	import { stat } from '@tauri-apps/plugin-fs';

	import 'vidstack/player';
	import 'vidstack/player/layouts/default';
	import 'vidstack/player/ui';

	import 'vidstack/player/styles/default/theme.css';
	import 'vidstack/player/styles/default/layouts/video.css';

	let { imageHash: mediaHash }: MediaModalProps = $props();

	let isOpen = $state(true);
	let tagsTextBoxContents = $state('');

	let groupModeSelectedIdx = $state(0);

	let flashResolutionX = $state(0);
	let flashResolutionY = $state(0);

	function updateTagsTextBoxContents(text: string) {
		tagsTextBoxContents = text;
	}

	async function getData(): Promise<MediaInfo> {
		const info = await commands.getInfo(mediaHash);
		//const info: MediaInfo = await invoke('get_info', {
		//	hash: mediaHash
		//});

		if (info?.mediaType === 'Video' && !(await commands.isRemoteDb())) {
			await commands.serveMedia(mediaHash);
		}

		return info!!;
	}

	async function onClose() {
		trace('close media modal');
		if (MediaModalStatusStore.tagsEditModeActive) {
			trace('update tags on close');
			await invoke('update_tags', {
				rawInput: tagsTextBoxContents,
				hash: mediaHash
			});
		} else {
		}
		commands.closeServer();
		MediaModalStatusStore.close();
	}

	async function getFlashResolution(data: MediaInfo) {
		if (data.mediaType === 'Flash') {
			const path = data.paths[0];
			const [resolutionX, resolutionY] = await commands.getSwfResolution(path);
			flashResolutionX = resolutionX;
			flashResolutionY = resolutionY;
		}
	}

	$effect(() => {
		console.log(flashResolutionX, flashResolutionY);
	});

	async function getFileUrl(data: MediaInfo): Promise<string> {
		if (await commands.isRemoteDb()) {
			return `${await commands.getRemoteServerUrl()}/media?hash=${mediaHash}`;
		} else if (data.mediaType === 'Video') {
			return 'http://localhost:3169';
		} else {
			return convertFileSrc(data.pathThatExists!);
		}
	}
</script>

<svelte:window
	onkeydown={(e) => {
		if (e.code === 'Escape') {
			onClose();
		}
	}}
/>

<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- TODO keyboard navigation -->
<dialog open={false}>
	{#await getData() then data}
		{#await getFileUrl(data) then fileUrl}
			<div class="modalWrapper"></div>

			<div
				class="dialogContents"
				use:clickOutsideMediaModal={async () => {
					await onClose();
					trace('click outside modal');
				}}
			>
				<div class="imageWrapper">
					<!-- svelte-ignore a11y_img_redundant_alt -->
					{#if data!.mediaType === 'Image'}
						<img
							src={fileUrl}
							alt="An image provided by user"
							style="aspect-ratio: {data!.aspectRatio};"
						/>
					{:else if data!.mediaType === 'Video'}
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
								<source
									type="video/webm"
									src={fileUrl}
									style="aspect-ratio: {data.aspectRatio} !important;"
								/>
							</media-provider>
							<media-video-layout></media-video-layout>
						</media-player>

						<!--
					<video src="http://localhost:3169" controls></video>
					-->
					{:else if data!.mediaType === 'Group'}
						<!--TODO-->
					{:else if data!.mediaType === 'Flash'}
						{#await getFlashResolution(data) then}
							<script src="ruffle/ruffle.js"></script>

							<div class="overflow-hidden">
								<object aria-label="User provided flash content">
									<!-- TODO Make the scaling configurable-->
									<embed src={fileUrl} width={flashResolutionX * 2} height={flashResolutionY * 2} />
								</object>
							</div>
						{/await}
					{/if}
				</div>

				<Sidebar {data} {updateTagsTextBoxContents} mediaUrl={fileUrl}></Sidebar>
			</div>
		{/await}
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
		z-index: 3;
		align-items: center;
		justify-content: center;
	}
	img {
		height: 100%;
		object-fit: contain;
	}

	.mediaPlayer {
		height: 100%;
	}

	/*
	When aspect-ratio is set on media-player, vidstack crops the video to fill
	the container (object-fit: cover on the inner video). Force contain instead.
	*/
	:global(media-player video) {
		object-fit: contain !important;
	}

	:global(media-player video) {
		object-fit: contain !important;
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

	@media (width < 1250px) {
		.dialogContents {
			left: 0 !important;
		}
	}

	/* 
	 videos with somewhat uncommon aspect ratios are "cropped" 
	 for some reason?
	 https://github.com/vidstack/player/issues/799
	 */
	:global(video) {
		height: 100%;
	}

	svg {
		display: block;
		margin: auto;
	}
</style>
