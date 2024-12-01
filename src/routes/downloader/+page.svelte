<script>
	import WindowTitlebar from '../../component/Decoration/WindowTitlebar.svelte';
	import Downloader from '../../component/Downloader/Downloader.svelte';
</script>

<div class="background">
	<WindowTitlebar platform="gnome"></WindowTitlebar>
	<Downloader></Downloader>
</div>

<style>
	.background {
		width: calc(100vw - 200px);
		background-color: var(--background);
		display: flex;
		flex-direction: column;
		height: var(--page-content-height);
		width: var(--page-content-width);
	}

	.titlebarInsides {
		color: var(--text);
	}
</style>
