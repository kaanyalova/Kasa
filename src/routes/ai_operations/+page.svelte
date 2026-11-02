<script>
	import AiOperations from '../../component/AiOperations/AiOperations.svelte';
	import WindowTitlebar from '../../component/Decoration/WindowTitlebar.svelte';
	import Downloader from '../../component/Downloader/Downloader.svelte';
</script>

<div class="background">
	<WindowTitlebar platform="gnome"></WindowTitlebar>
	<AiOperations></AiOperations>
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
</style>
