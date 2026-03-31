<script lang="ts">
	import { commands } from '$lib/tauri_bindings';
	import { onMount } from 'svelte';
	import CodeEditor from './CodeEditor.svelte';
	import '@alenaksu/json-viewer';
	import '../../fonts.css';

	let selectedExtractor = $state('reddit');
	let exampleMetadataJson = $state('');

	onMount(async () => {});

	$effect(async () => {
		selectedExtractor;

		await updateExampleMetadataForSelectedExtractor();
	});

	async function getExistingExtractors(): Promise<Array<string>> {
		return await commands.getExistingExtractorNames();
	}

	async function updateExampleMetadataForSelectedExtractor() {
		const example = await commands.getExampleMetadataForExtractor(selectedExtractor);
		exampleMetadataJson = example;
	}

	function getFileName(path: string): string {
		const splitBySlashes = path.split('/');
		const lastPart = splitBySlashes[splitBySlashes.length - 1];
		const splitByDots = lastPart.split('.');
		return splitByDots[0];
	}
</script>

<div class="extractorEditor">
	<div class="selectionSection">
		<div class="selectionContainer">
			<div class="selectionContainerTop">
				<div class="extractorsTitle">Extractors</div>
				<button class="addExtractor">+</button>
			</div>

			<ul class="selectionList multicolorRows">
				{#await getExistingExtractors() then extractors}
					{#each extractors as extractor}
						<li class="selectionListEntry">
							{getFileName(extractor)}
						</li>
					{/each}
				{/await}
			</ul>
		</div>
	</div>
	<div class="codeEditorSection">
		<div class="editorContainer">
			<CodeEditor extractorName={selectedExtractor} fileExtension="py"></CodeEditor>
		</div>
	</div>

	<div class="exampleMetadataJson">
		{#await updateExampleMetadataForSelectedExtractor() then}
			<div class="metadataViewerContainer">
				<json-viewer class="metadataViewer" data={JSON.parse(exampleMetadataJson)}></json-viewer>
			</div>
		{/await}
	</div>
</div>

<style>
	.metadataViewer {
		border: 1px solid var(--secondary-alt);
		--background-color: var(--background);
		--font-family: 'UbuntuMono';
		padding: 4px;
	}

	.metadataViewerContainer {
		position: relative;
		left: 4px;
		padding: 4px;
		overflow-y: auto;
	}

	.extractorEditor {
		display: flex;
		flex-grow: 1;
	}

	.selectionSection {
		flex: 2;
		display: flex;
	}

	.codeEditorSection {
		flex: 6;
		display: flex;
	}

	.exampleMetadataJson {
		flex: 6;
		display: flex;
		color: var(--text);
	}

	.editorContainer {
		display: flex;
		margin: 4px;
		border: 1px solid var(--secondary-alt);
		flex-grow: 1;
	}

	.selectionContainer {
		border: 1px solid var(--secondary-alt);
		margin: 4px;
		flex-grow: 1;
	}

	.extractorsTitle {
		color: var(--text);
		font-weight: bold;
	}

	.selectionContainerTop {
		display: flex;
		border-bottom: 1px solid var(--secondary-alt);

		justify-content: space-between;
	}
	.addExtractor {
		height: 24px;
		width: 24px;
		background-color: var(--accent);
	}

	.addExtractor:hover {
		background-color: var(--accent-hover);
	}

	.multicolorRows > li:nth-child(2n) {
		background-color: var(--secondary-alt);
	}

	.selectionList {
	}

	.selectionListEntry {
		color: white;
	}

	:global(.cm-editor) {
		height: calc(100vh - 45px);
		overflow: auto;
		flex-grow: 1;
	}

	:global(.cm-scroller) {
		overflow: auto;
	}
</style>
