<script lang="ts">
	import { humanFileSize } from '$lib/formatBytes';
	import ProgressBar from './ProgressBar.svelte';
	let { fileSize, downloadedSize, downloadSpeed, url, extractorName }: DownloaderRowProps =
		$props();
</script>

<li class="downloaderRow">
	<div class="progressBarContainer">
		<div class="topPart">
			<a href="url" class="urlText">{url}</a>
			<div class="extractor">{extractorName}</div>
		</div>

		<ProgressBar progress={downloadedSize / fileSize} />
		<span class="valueText">
			{humanFileSize(downloadedSize)}
			/
			{humanFileSize(fileSize)}
			({humanFileSize(downloadSpeed)}/s)
		</span>
	</div>
</li>

<style>
	.progressBarContainer {
		color: var(--text);
		padding-left: 4px;
		padding-right: 4px;
	}

	.urlText {
		text-decoration: underline;
		font-size: smaller;
		padding-left: 4px;
		position: relative;
		top: 4px;

		display: inline-block;
		max-width: 60%;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
		vertical-align: bottom;
	}

	.valueText {
		font-family: 'UbuntuMono';
		font-size: small;
		position: relative;
		bottom: 8px;
	}

	.topPart {
		display: flex;
		justify-content: space-between;
		min-width: 0;
	}

	.extractor {
		padding-right: 2px;
		text-align: bottom;
		font-size: small;
		font-family: 'UbuntuMono';
		position: relative;
		top: 4px;
	}
</style>
