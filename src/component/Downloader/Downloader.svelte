<script lang="ts">
	import BorderedBox from '../Shared/BorderedBox.svelte';
	import '../../fonts.css';
	import HorizontalDivider from '../Shared/Dividers/HorizontalDivider.svelte';
	import { DividerSizes } from '../Shared/Dividers/DividerSizes';
	import { commands } from '$lib/tauri_bindings';
	import { comma } from 'postcss/lib/list';
	import { onMount } from 'svelte';

	let status = $state('');
	let downloadBox = $state('');

	async function onDownload() {
		const links = downloadBox.split('\n');

		console.log(links);
		links.forEach(async (link) => {
			console.log('processing link!');

			await commands.downloadAndIndex(link);
			console.log('done');
		});
	}

	async function updateStatus() {
		let newStatus = await commands.getDownloadProgress();
		status = JSON.stringify(newStatus);
	}
</script>

<div class="downloader">
	<div class=""></div>

	<div class="sections">
		<div class="leftSide">
			<textarea class="downloadInputBox borderedBox" bind:value={downloadBox}></textarea>
		</div>
		<div class="rightSide">
			Enter the URLs you want to download using gallery-dl separated by newlines

			<HorizontalDivider height={DividerSizes.Small}></HorizontalDivider>
			<button class="button" onclick={onDownload}> Download </button>
		</div>
	</div>

	<div class="status">
		<button
			onclick={async () => {
				updateStatus();
			}}
		>
			UpdateStatus</button
		>
		{status}
	</div>
</div>

<style>
	.button {
		padding: 8px;
		background-color: var(--accent);
		width: 100%;
		color: var(--text-opposite);
		border-radius: 6px;
		border: 1px solid color-mix(in srgb, var(--accent) 50%, black 50%);
	}

	.button:hover {
		background-color: color-mix(in srgb, var(--accent) 80%, black 20%);
	}

	.downloader {
		margin: 6px;
		display: flex;
		flex-grow: 1;
	}

	.sections {
		display: flex;
		flex-grow: 1;
	}

	.rightSide {
		width: 30vw;
		color: var(--text);
		padding: 4px;
		word-wrap: break-word;
		font-size: small;
	}

	.leftSide {
		flex-grow: 1;
		display: flex;
	}

	.downloadInputBox {
		flex-grow: 1;
		color: var(--text);
		resize: none;
		background-color: var(--background);
		font-family: 'UbuntuMono';
		font-size: small;
		outline: none;
		padding: 4px;
	}
	.downloadInputBox:focus {
		border: 1px solid var(--accent);
	}

	.borderedBox {
		border: 1px solid var(--secondary-alt);
		/*padding: 4px;*/
		width: 100%;
	}
	.status {
		color: var(--text);
	}
</style>
