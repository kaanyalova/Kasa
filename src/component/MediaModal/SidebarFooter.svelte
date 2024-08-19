<script lang="ts">
	import { writeImage } from '@tauri-apps/plugin-clipboard-manager';
	import ArrowUpRightFromSquare from '../Vector/ArrowUpRightFromSquare.svelte';
	import Clipboard from '../Vector/Clipboard.svelte';
	import { writeFile } from '@tauri-apps/plugin-fs';

	let { data }: SidebarFooterProps = $props();

	function onCopyButtonClicked() {}
</script>

<div class="sidebarFooter">
	<button title="Copy {data.mediaType}"> <Clipboard height={20} width={20}></Clipboard></button>
	<button title="Open Externally">
		<ArrowUpRightFromSquare height={20} width={20}></ArrowUpRightFromSquare></button
	>
	<button title="Action"> Act </button>
	<button title="Action"> Act </button>
	<button title="Action"> Act </button>
</div>

<style>
	.sidebarFooter {
		height: 50px;
		display: flex;
		align-items: center;
		justify-content: center;
		border-top: 1px solid var(--secondary-alt);
	}

	button {
		margin: 5px;
		padding: 5px;
		fill: var(--text);
		display: flex;
		align-items: center;
		justify-content: center;
		border: 1px solid var(--secondary-alt);
		border-radius: 4px;
		height: 35px;
		width: 35px;
	}
	button:hover {
		background-color: var(--secondary-alt);
	}
</style>
