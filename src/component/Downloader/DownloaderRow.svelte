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

		<div class="bottomRow">
			<div class="bottomRowLeftSide">
				{humanFileSize(downloadedSize)}
				/
				{humanFileSize(fileSize)}
				({humanFileSize(downloadSpeed)}/s)
			</div>

			<div class="bottomRowRightSide">
				<button class="cancelButton">Cancel</button>
			</div>
		</div>
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

	.bottomRow {
		display: flex;
		justify-content: space-between;
	}

	.cancelButton {
		font-size: small;
		padding: 2px;
		background-color: var(--destructive);
		border-radius: 2px;
		border: 1px solid var(--destructive-border);
		padding-right: 8px;
		padding-left: 8px;
		cursor: pointer;
		margin-bottom: 2px;
	}

	.cancelButton:hover {
		background-color: var(--destructive-border);
	}

	.bottomRowRightSide {
	}

	.bottomRowLeftSide {
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
		margin: 2px;
		text-align: bottom;
		font-size: small;
		font-family: 'UbuntuMono';
		position: relative;
		top: 4px;
		border: 1px solid var(--secondary-alt);
	}
</style>
