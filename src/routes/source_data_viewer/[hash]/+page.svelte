<script lang="ts">
	import WindowTitlebar from '../../../component/Decoration/WindowTitlebar.svelte';
	import type { PageProps } from './$types';
	import '@alenaksu/json-viewer';
	import '../../../fonts.css';

	let { data }: PageProps = $props();

	function getFileName(path: string): string {
		let split = path.split('/');
		let last = split[split.length - 1];
		return last;
	}
</script>

{#each data.sources as source}
	<div class="tilebar">
		<WindowTitlebar platform="gnome"></WindowTitlebar>
	</div>

	<div class="mainContainer">
		<div class="title">
			Metadata for <span class="monospaced">{getFileName(JSON.parse(source.raw_data).path)}</span>
		</div>

		<json-viewer data={JSON.parse(source.raw_data)}></json-viewer>
	</div>
{/each}

<style>
	json-viewer {
		--background-color: var(--background);
		--font-family: 'UbuntuMono';
		margin: 8px;
		position: relative;
		border: 1px solid var(--secondary-alt);
		padding: 8px;
	}

	.mainContainer {
		flex-direction: column;
		background-color: var(--background);
		height: calc(100vh - 36px);
		overflow-y: auto;
		position: relative;
	}

	.title {
		margin: 8px;
		color: var(--text);
	}

	.monospaced {
		font-family: 'UbuntuMono';
	}
</style>
