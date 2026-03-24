<script lang="ts">
	import BorderedBox from '../Shared/BorderedBox.svelte';
	import '../../fonts.css';
	import HorizontalDivider from '../Shared/Dividers/HorizontalDivider.svelte';
	import { DividerSizes } from '../Shared/Dividers/DividerSizes';
	import { commands, type GalleryDlStatus } from '$lib/tauri_bindings';
	import { comma } from 'postcss/lib/list';
	import { onDestroy, onMount } from 'svelte';
	import Download from '../Vector/Download.svelte';
	import { stat } from '@tauri-apps/plugin-fs';
	import { listen, type UnlistenFn } from '@tauri-apps/api/event';
	import DownloaderRow from './DownloaderRow.svelte';

	let downloadStatuses: Record<string, GalleryDlStatus> = $state({});
	let downloadBox = $state('');
	let unlistenFn: UnlistenFn | undefined;

	async function onDownload() {
		await commands.queueDownloadJob(downloadBox);
		downloadBox = '';
	}

	onMount(async () => {
		unlistenFn = await listen('downloader_progress_updated', async () => {
			downloadStatuses = await commands.getDownloaderStatuses();
			console.log('updating download statuses');
			console.log(downloadStatuses);
		});
	});

	onDestroy(() => {
		if (unlistenFn) {
			unlistenFn();
		}
	});

	async function onOpenGalleryDlConfigFile() {
		const config = await commands.getConfig();
		const path = config.Downloader.gdl_config_path;

		if (path) {
			await commands.openWithSystemDefaultApp(path);
		}
	}
</script>

<div class="downloader">
	<div class="sections">
		<div class="leftSide">
			<div class="downloadBar">
				<div class="downloadIcon">
					<Download height={12} width={12}></Download>
				</div>
				<input
					type="text"
					class="urlInput"
					placeholder="URL to download"
					bind:value={downloadBox}
				/>
			</div>

			<div class="downloadStatusesTitle">Downloads</div>
			<div class="downloadStatuses">
				<ul class="multicolorRows downloaderRow">
					{#each Object.entries(downloadStatuses) as [_, status]}
						<DownloaderRow
							downloadSpeed={status.bytes_per_second}
							downloadedSize={status.bytes_downloaded}
							fileSize={status.bytes_total}
							extractorName={status.extractor}
							url={status.url}
						></DownloaderRow>
					{/each}
				</ul>
			</div>
		</div>

		<div class="rightSide">
			<button class="downloadButton" onclick={onDownload}> Download </button>
			<button class="galleryDlConfigButton" onclick={onOpenGalleryDlConfigFile}>
				Open gallery_dl config
			</button>

			<div class="versions">
				<span class="code">gallery_dl</span> version <span class="code"> 1.31.10</span>
				<span class="code">rustpython</span> version <span class="code">0eddee5</span>
			</div>
		</div>
	</div>

	<!--
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
	-->
</div>

<style>
	.downloadButton {
		padding: 4px;
		background-color: var(--accent);
		width: 100%;
		color: var(--text-opposite);
		border-radius: 6px;
		border: 1px solid color-mix(in srgb, var(--accent) 50%, black 50%);
		font-weight: bold;
		margin-top: 2px;
		margin-bottom: 2px;
	}

	.downloadButton:hover {
		background-color: color-mix(in srgb, var(--accent) 80%, black 20%);
	}

	.galleryDlConfigButton {
		padding: 4px;
		color: var(--text);
		border: 1px solid var(--secondary-alt);
		width: 100%;
		margin-top: 2px;
		margin-bottom: 2px;
		border-radius: 6px;
	}

	.galleryDlConfigButton:hover {
		background-color: var(--secondary-alt);
	}

	.downloader {
		display: flex;
		flex-direction: column;
		height: 100%;
	}

	.sections {
		display: flex;
		flex-grow: 1;
	}

	.rightSide {
		display: flex;
		flex-direction: column;
		width: 30vw;
		color: var(--text);
		padding: 4px;
		word-wrap: break-word;
		font-size: small;
		border-left: 1px solid var(--secondary-alt);
	}

	.leftSide {
		flex-grow: 1;
		display: flex;
		flex-direction: column;
		margin: 4px;
		min-width: 0;
	}

	.downloadInputBox {
		flex-grow: 1;
		color: var(--text);
		resize: none;
		background-color: var(--background);
		font-family: 'UbuntuMono';
		font-size: small;
		outline: none;
	}

	.borderedBox {
		border: 1px solid var(--secondary-alt);
		/*padding: 4px;*/
		width: 100%;
	}
	.status {
		color: var(--text);
	}

	.urlInput {
		background-color: var(--background);
		outline: none;
		border: 1px solid var(--secondary-alt);
		color: var(--text);
		padding-left: 4px;
		padding-right: 4px;
		flex-grow: 1;
		min-width: 0;
		box-sizing: border-box;
	}

	.urlInput:focus {
		border: 1px solid var(--accent);
	}

	.code {
		font-family: 'UbuntuMono';
		border: 1px solid var(--secondary-alt);
		padding: 2px;
	}

	.versions {
		font-size: 14px;
		line-height: 24px;
		padding: 4px;
		border: 1px solid var(--secondary-alt);
		margin-top: 2px;
		margin-bottom: 2px;
	}

	.downloadBar {
		display: flex;
		width: 100%;
		margin-top: 4px;
		margin-bottom: 4px;
	}

	.downloadIcon {
		background-color: var(--accent);
		width: 32px;
		height: 32px;
		display: flex;
		align-items: center;
		border: 1px solid color-mix(in srgb, var(--accent) 50%, black 50%);
	}

	.downloadStatuses {
		display: flex;
		flex-direction: column;
		flex-grow: 1;
		border: 1px solid var(--secondary-alt);
		color: var(--text);
		overflow-y: auto;
		min-width: 0;
	}

	.downloadStatusesTitle {
		color: var(--text);
	}

	.multicolorRows :global(li:nth-child(2n)) {
		background-color: var(--secondary-alt);
	}
</style>
