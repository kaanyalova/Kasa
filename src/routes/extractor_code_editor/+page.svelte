<script>
	import WindowTitlebar from '../../component/Decoration/WindowTitlebar.svelte';
	import CodeEditor from '../../component/Extractors/CodeEditor.svelte';
	import ExtractorEditor from '../../component/Extractors/ExtractorEditor.svelte';
</script>

<div class="background">
	<WindowTitlebar platform="gnome"></WindowTitlebar>
	<ExtractorEditor></ExtractorEditor>
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
