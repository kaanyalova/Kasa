<script lang="ts">
	import FileManager from '../../Vector/FileManager.svelte';
	import '../../../fonts.css';
	import { info } from '@tauri-apps/plugin-log';
	import Button from './Button.svelte';
	import { commands } from '$lib/tauri_bindings';
	import { openFilePickerWithSelectDialog } from '$lib/openFilePicker';

	let { details, path = $bindable(), onConfirm }: PathInputProps = $props();

	function onClickBrowse() {}
</script>

<div class="pathInput">
	<input type="text" bind:value={path} class="dbPathInput textInput monoFont" />
	<button class="fileSelectButton">
		<span class="details">Browse</span>
		<div class="icon">
			<FileManager height={18} width={18}></FileManager>
		</div></button
	>
</div>
<div class="bottomRow">
	<div class="details dbInfoText">{details}</div>
	<div>
		<Button onClick={() => onConfirm()}>Confirm</Button>
	</div>
</div>

<style>
	.pathInput {
		display: flex;
		flex-direction: row;
		margin-bottom: 4px;
	}

	.dbPathInput {
		flex-grow: 1;
	}

	.textInput {
		background-color: var(--background);
		border: 1px solid var(--secondary-alt);
		padding-right: 6px;
		padding-left: 6px;
	}

	.monoFont {
		font-family: 'UbuntuMono';
	}

	.fileSelectButton {
		display: flex;
		align-items: center;
		fill: var(--text);
		border: 1px solid var(--secondary-alt);
		margin-left: 4px;
		margin-right: 4px;
		padding-left: 4px;
		padding-right: 4px;
	}

	.details {
		font-size: small;
	}

	.icon {
		display: flex;
		padding-left: 4px;
	}

	.bottomRow {
		display: flex;
	}

	.dbInfoText {
		display: flex;
		/*justify-content: center;
		align-items: center;*/
	}
	.confirmButton {
		font-size: smaller;
		background-color: var(--accent);
		color: black;
		padding: 2px;
		padding-left: 4px;
		padding-right: 4px;
		border-radius: 6px;
		margin: 4px;
		position: relative;
		width: 76px;
	}

	.bottomRow div:last-child {
		margin-left: auto;
	}

	.fileSelectButton:hover {
		background-color: var(--secondary-alt);
	}
</style>
