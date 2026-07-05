<script lang="ts">
	import { commands } from '$lib/tauri_bindings';
	import { show } from '@tauri-apps/api/app';
	let { hash }: ImportInfoProps = $props();

	//let importInfo = commands.getMediaSource();
	import '../../fonts.css';
	import { WebviewWindow } from '@tauri-apps/api/webviewWindow';
	import { open } from '@tauri-apps/plugin-shell';

	function formatSource(source: string): string {
		return source.charAt(0).toUpperCase() + source.slice(1);
	}

	function showFullImportMetadata() {
		const window = new WebviewWindow('source_data_viewer', {
			url: `/source_data_viewer/${hash}/`,
			decorations: false
		});

		window.once('tauri://created', function () {
			console.log('created');
		});

		window.once('tauri://error', function (e) {
			console.log(e);
		});
	}

	async function onClickSourceLink(link: string) {
		await open(link);
	}
</script>

{#await commands.getMediaSources(hash) then sources}
	{#if sources.length !== 0}
		Import Info
	{/if}

	{#each sources as source}
		<ul class="details multicolorRows">
			<li class="paddedLi">
				Imported by <span class="monospaced importerType">{source.importer_type}</span>
			</li>

			<li class="paddedLi">
				Imported from <span class="importSource">{formatSource(source.source)}</span>
			</li>
			<li class="paddedLi linkContainer">
				<span class="linkHeader">Link: </span>
				<button class="link" onclick={async () => onClickSourceLink(source.link_or_path)}>
					{source.link_or_path}</button
				>
			</li>

			<li class="noColoredRow">
				<button
					class="fullImportMetadataButton"
					onclick={() => {
						showFullImportMetadata();
					}}
				>
					See full import metadata
				</button>
			</li>
		</ul>
	{/each}
{/await}

<style>
	.header {
		padding-left: 4px;
		padding-right: 4px;
	}

	.details {
		font-size: small;
		border: var(--secondary-alt) 1px solid;
		padding: 2px;
		border-radius: 2px;
		margin: 4px;
	}

	.paddedLi {
		padding: 2px;
	}

	.importSource {
		padding: 2px;
		color: var(--text);
		border-radius: 2px;
		font-family: 'UbuntuMono';
		border: var(--background) 1px solid;
		background-color: var(--background);
	}

	.monospaced {
		font-family: 'UbuntuMono';
	}

	.importerType {
		border: 1px solid var(--secondary-alt);
		padding: 2px;
	}

	.linkContainer {
		word-break: break-all;
	}

	.link {
		text-decoration: underline;
		text-align: left;
	}

	.linkHeader {
		font-weight: bold;
	}

	.fullImportMetadataButton {
		background-color: var(--accent);
		color: var(--text-opposite);
		padding: 2px;
		padding-left: 4px;
		padding-right: 4px;
		margin: 2px;
		border-radius: 2px;
	}

	.fullImportMetadataButton:hover {
		background-color: color-mix(in srgb, var(--accent) 90%, black 10%);
	}

	.multicolorRows > li:nth-child(2n) {
		background-color: var(--secondary-alt);
	}

	.noColoredRow {
		background-color: var(--background) !important;
	}
</style>
