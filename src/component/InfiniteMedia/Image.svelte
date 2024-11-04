<script lang="ts">
	import { convertFileSrc, invoke } from '@tauri-apps/api/core';
	import { onMount } from 'svelte';
	import { MediaModalStatusStore } from '../MediaModal/MediaModalStatusStore.svelte';
	import { info } from '@tauri-apps/plugin-log';
	import { commands } from '$lib/tauri_bindings';

	let { hash, width, height, offset_x, offset_y, isSelected }: ImageProps = $props();

	let image: string = $state('');
	let previous_hash = $state(hash);
	let promise = $state(getThumbnail(hash));

	/**
	 * Returns the base64 encoded image from the db with `data:image/png;base64,` appended
	 * @param hash
	 * Hash of the image
	 */
	async function getThumbnail(hash: string): Promise<string> {
		const thumbnail_bytes = await commands.getThumbnailFromDb(hash);
		// TODO support other image formats than png
		const thumbnail = 'data:image/png;base64, ' + thumbnail_bytes;
		return thumbnail;
	}

	onMount(async () => {
		image = await getThumbnail(hash);
	});

	function onClick() {
		MediaModalStatusStore.open(hash);
	}

	// the rust side seems to reassign hashes to existing rows, we load images only once in the onMount(), additional
	// changes to the hash doesn't get reflected to the image, we could reload the image every time the hash
	// gets set but that would cause all images to rerender on every single resize "tick", we only want to
	// rerender a few images  that was changed, there is no useMemo equivalent in svelte so we do this
	$effect(async () => {
		hash;

		if (previous_hash !== hash) {
			promise = getThumbnail(hash);
		} else {
		}
		previous_hash = hash;
	});

	/*
	$effect(async () => {
		hash;
		if (hash === )
		const _path: string = await invoke('get_thumbnail', { hash: hash });
		const path = `/home/kaan/Belgeler/0000_Projects/Kasa/__dev_thumbs/${_path}`;

		image = convertFileSrc(path).toString();
	});


*/
</script>

<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<!-- svelte-ignore a11y_click_events_have_key_events -->

{#await promise}
	<div
		class="fakeImage"
		style="transform:translate3d({offset_x}px,0px, 0px); height:{height}px; width:{width}px"
	>
		<div class="loader"></div>
	</div>
{:then thumbnail}
	<img
		onclick={onClick}
		src={thumbnail}
		ondragstart={(e) => {
			// Disable dragging of images on grid
			// Why is there a more convenient way of doing this wtf
			e.preventDefault();
		}}
		alt=""
		style="transform:translate3d({offset_x}px,0px, 0px); height:{height}px; width:{width}px"
		role="figure"
	/>
{/await}

<style>
	img {
		position: absolute;
		cursor: pointer;
	}

	img:hover {
		border: var(--secondary) solid 3px;
		transition: 100ms;
	}

	.fakeImage {
		border: 1px solid var(--secondary-alt);
		display: flex;
		flex-grow: 1;
		align-items: center;
		justify-content: center;
		position: absolute;
	}

	/* HTML: <div class="loader"></div> */
	.loader {
		width: 48px;
		height: 48px;
		border: 5px solid var(--text);
		border-bottom-color: transparent;
		border-radius: 50%;
		display: inline-block;
		box-sizing: border-box;
		animation: rotation 1s linear infinite;
	}

	@keyframes rotation {
		0% {
			transform: rotate(0deg);
		}
		100% {
			transform: rotate(360deg);
		}
	}
</style>
