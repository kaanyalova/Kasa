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
		height: calc(100vh - var(--window-border-size) * 2);
		width: calc(100vw - var(--window-border-size) * 2);
	}

	.titlebarInsides {
		color: var(--text);
	}
</style>
