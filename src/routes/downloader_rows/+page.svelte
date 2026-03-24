<script>
	import DownloaderRow from '../../component/Downloader/DownloaderRow.svelte';
	import ProgressBar from '../../component/Downloader/ProgressBar.svelte';
</script>

<ul class="multicolorRows downloaderRows">
	<DownloaderRow
		downloadSpeed={1000}
		downloadedSize={1024 * 10}
		fileSize={1024 * 100}
		url="test.com"
		extractorName="extractor"
	></DownloaderRow>

	<DownloaderRow
		downloadSpeed={1000}
		downloadedSize={1024 * 10}
		fileSize={1024 * 100}
		url="testasjdasddsajkdajksdnajksjndaksdnajkdjtestasjdasddsajkdajksdnajksjndaksdnajkdjtestasjdasddsajkdajksdnajksjndaksdnajkdj.com"
		extractorName="extractor"
	></DownloaderRow>

	<DownloaderRow
		downloadSpeed={1000}
		downloadedSize={1024 * 10}
		fileSize={1024 * 100}
		url="test.com"
		extractorName="extractor"
	></DownloaderRow>
</ul>

<style>
	.multicolorRows :global(li:nth-child(2n)) {
		background-color: var(--secondary-alt);
	}

	.downloaderRows {
		background-color: var(--background);
		width: 500px;
	}
</style>
