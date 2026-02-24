<script lang="ts">
	import WindowTitlebar from '../../../component/Decoration/WindowTitlebar.svelte';
	import type { PageProps } from './$types';
	import '@alenaksu/json-viewer';

	let { data }: PageProps = $props();
</script>

{#each data.sources as source}
	<div class="tilebar">
		<WindowTitlebar platform="gnome"></WindowTitlebar>
	</div>

	<div class="container">
		<div class="innerContainer">
			<json-viewer class="dataViewer" data={JSON.parse(source.raw_data)}></json-viewer>
		</div>
	</div>
{/each}

<style>
	.dataViewer {
		width: 100vw;
		height: 100vh;
	}
	.innerContainer {
		margin: 8px;
		position: relative;
		top: 20px;
		overflow-y: scroll;
	}

	.tilebar {
		position: absolute;
		width: 100vw;
	}

	.container {
		width: calc(100vw);
		background-color: var(--background);
		display: flex;
		flex-direction: column;
	}
	.tilebarInsides {
		width: calc(100vw);
	}
</style>
