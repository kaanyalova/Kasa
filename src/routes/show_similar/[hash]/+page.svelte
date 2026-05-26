<script lang="ts">
	import WindowTitlebar from '../../../component/Decoration/WindowTitlebar.svelte';
	import Downloader from '../../../component/Downloader/Downloader.svelte';
	import ShowSimilar from '../../../component/ShowSimilar/ShowSimilar.svelte';

	let { data } = $props();
</script>

<div class="background">
	<WindowTitlebar platform="gnome"></WindowTitlebar>
	<ShowSimilar closest={data.closest}></ShowSimilar>
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
