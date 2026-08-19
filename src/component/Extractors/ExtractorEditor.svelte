<script lang="ts">
	import { commands } from '$lib/tauri_bindings';
	import { onMount } from 'svelte';
	import CodeEditor from './CodeEditor.svelte';
	import '@alenaksu/json-viewer';
	import '../../fonts.css';
	import { Pane, Splitpanes } from 'svelte-splitpanes';
	import './SplitPanes.scss';

	let selectedExtractor = $state('');
	let exampleMetadataJson = $state('');
	let codeEditor: CodeEditor | undefined = $state(undefined);
	let savedTextVisible = $state(false);
	let savedTimeout: any | undefined = $state(undefined);

	onMount(async () => {
		document.addEventListener('keydown', async (e) => {
			if (e.ctrlKey && e.key == 's') {
				e.preventDefault();
				await onSave();
			}
		});
	});

	$effect(async () => {
		selectedExtractor;

		await updateExampleMetadataForSelectedExtractor();
	});

	function showSavedMessage() {
		savedTextVisible = true;
		if (savedTimeout) {
			clearTimeout(savedTimeout);
		}
		savedTimeout = setTimeout(() => {
			savedTextVisible = false;
		}, 1500);
	}

	async function getExistingExtractors(): Promise<Array<string>> {
		return await commands.getExistingExtractorNames();
	}

	async function updateExampleMetadataForSelectedExtractor() {
		const example = await commands.getExampleMetadataForExtractor(selectedExtractor);
		exampleMetadataJson = example;
	}

	function getFileName(path: string): string | undefined {
		if (path === '') {
			return undefined;
		}

		const splitBySlashes = path.split('/');
		const lastPart = splitBySlashes[splitBySlashes.length - 1];
		const splitByDots = lastPart.split('.');
		return splitByDots[0].trim();
	}

	async function onSave() {
		const contents = codeEditor?.getCode();
		let name = getFileName(selectedExtractor);
		if (contents && name) {
			await commands.setExtractorContents(name, 'py', contents);
			await commands.reloadExtractors();
		}
		showSavedMessage();
	}
</script>

<div class="extractorEditor">
	<Splitpanes style="height:calc(100vh - 31px);" class="extractorEditor" theme="kasa-theme">
		<!--
		selection
		-->
		<Pane class="selectionSection" size={15}>
			<div class="selectionWrapper">
				<div class="selectionContainer">
					<div class="selectionContainerTop">
						<div class="extractorsTitle">Extractors</div>
						<button class="addExtractor">+</button>
					</div>

					<ul class="selectionList multicolorRows">
						{#await getExistingExtractors() then extractors}
							{#each extractors as extractor}
								<li class="selectionListEntry">
									<button
										class="extractorButton"
										onclick={() => (selectedExtractor = extractor)}
										class:selectedExtractor={selectedExtractor === extractor}
									>
										{getFileName(extractor)}
									</button>
								</li>
							{/each}
						{/await}
					</ul>
				</div>
			</div>
		</Pane>
		<!--
		code editor
		-->
		<Pane size={55}>
			<div class="codeEditorSection">
				<div class="codeEditorActions">
					<button class="codeEditorActionButton" onclick={async () => onSave()}
						>Save (Ctrl + S)</button
					>
					<button class="codeEditorActionButton">Zoom In (todo)</button>
					<button class="codeEditorActionButton">Zoom Out (todo)</button>
					<div class="savedMessageContainer">
						<div class="savedMessage" class:savedMessageVisible={savedTextVisible}>Saved</div>
					</div>
				</div>

				<div class="editorContainer">
					<CodeEditor
						extractorName={getFileName(selectedExtractor)}
						fileExtension="py"
						bind:this={codeEditor}
					></CodeEditor>
				</div>
			</div>
		</Pane>

		<!--
		example
		-->
		<Pane size={30}>
			<div class="exampleMetadataWrapper">
				<div class="exampleMetadataJson">
					<div class="metadataTitle">
						Example data for extractor <span class="extractorName">{selectedExtractor}</span>
					</div>
					{#await updateExampleMetadataForSelectedExtractor() then}
						<div class="metadataViewerContainer">
							<json-viewer class="metadataViewer" data={JSON.parse(exampleMetadataJson)}
							></json-viewer>
						</div>
					{/await}
				</div>
			</div>
		</Pane>
	</Splitpanes>
</div>

<style>
	:global(.extractorEditor) {
		display: flex;
		flex-grow: 1;
		height: 100%;
		background-color: var(--background);
	}

	.metadataViewer {
		--background-color: var(--background);
		--font-family: 'UbuntuMono';
		padding: 4px;
	}

	.metadataTitle {
		font-weight: bold;
		margin: 4px;
	}

	.metadataViewerContainer {
		left: 4px;
		overflow-y: auto;
		margin: 4px;
		flex-direction: column;
		height: calc(100vh - 68px);
		flex-grow: 1;
		margin-top: 0;
		display: flex;
		background-color: var(--background);
		border: 1px solid var(--secondary-alt);
	}

	.extractorName {
		font-family: 'UbuntuMono';
		border: 1px solid var(--secondary-alt);
		font-weight: normal;
	}

	.selectionSection {
		flex: 2;
		display: flex;
		flex-direction: column;
	}

	.selectionWrapper {
		display: flex;
		flex-direction: column;
		height: 100%;
		background-color: var(--background);
	}

	.selectionContainer {
		display: flex;
		flex-direction: column;
		flex-grow: 1;
		border: 1px solid var(--secondary-alt);
		margin: 4px;
		height: 100%;
	}

	.codeEditorSection {
		flex: 6;
		display: flex;
		flex-direction: column;
	}

	.exampleMetadataJson {
		flex: 6;
		display: flex;
		flex-direction: column;
		color: var(--text);
	}

	.editorContainer {
		display: flex;
		margin: 4px;
		margin-top: 0;
		border: 1px solid var(--secondary-alt);
		flex-grow: 1;
	}

	.extractorsTitle {
		color: var(--text);
		font-weight: bold;
		padding-left: 8px;
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

	.selectionListEntry {
		color: white;
		display: flex;
	}

	.selectionList {
		border: 1px solid var(--secondary-alt);
		margin: 8px;
		height: 100%;
		overflow-y: scroll;
		padding: 1px;
	}

	:global(.cm-editor) {
		height: calc(100vh - 71px);
		overflow: auto;
		flex-grow: 1;
	}

	:global(.cm-scroller) {
		overflow: auto;
	}

	.exampleMetadataWrapper {
		background-color: var(--background);
	}

	.codeEditorActions {
		height: 34px;
		display: flex;
		align-items: center;
		padding-left: 8px;
		padding-right: 4px;
		gap: 8px;
		border: 1px solid transparent;
	}

	.codeEditorActionButton {
		background-color: var(--accent);
		color: var(--text-opposite);
		height: 24px;
		padding-left: 4px;
		padding-right: 4px;
		border-radius: 4px;
	}

	.codeEditorActionButton:hover {
		background-color: var(--accent-hover);
	}

	.selectedExtractor {
		border: 1px solid var(--accent) !important;
	}

	.extractorButton {
		display: flex;
		flex-grow: 1;
		padding-right: 4px;
		padding-left: 4px;
		border: 1px solid transparent;
	}

	.extractorButton:hover {
		border: 1px solid var(--secondary);
	}

	.savedMessageContainer {
		display: flex;
		align-items: center;
		flex-grow: 1;
		justify-content: end;
	}

	.savedMessage {
		align-self: center;
		color: white;
		border: 1px solid var(--secondary-alt);
		padding: 1px;
		transition:
			opacity 300ms,
			content-visibility 300ms allow-discrete;
		opacity: 0;
	}

	.savedMessageVisible {
		opacity: 1;
	}
</style>
