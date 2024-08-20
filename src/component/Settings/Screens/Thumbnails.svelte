<script lang="ts">
	import { commands } from '$lib/tauri_bindings';
	import FileManager from '../../Vector/FileManager.svelte';
	import '../../../fonts.css';
	import Select from 'svelte-select';

	let promise = $state(commands.getThumbsDbInfo());

	let db_path = $state('');

	// TODO move the confirm button to bottom right
</script>

{#await promise then info}
	<div class="thumbnails">
		<span class="title">Thumbnail Database</span>
		<div class="borderedBox">
			<div class="pathInput">
				<input type="text" value={info?.path} class="dbPathInput textInput monoFont" />
				<button class="fileSelectButton">
					<span class="details">Browse</span>
					<div class="icon">
						<FileManager height={18} width={18}></FileManager>
					</div></button
				>
			</div>
			<div class="bottomRow">
				<div class="details dbInfoText">{info?.image_count} Images {info?.size}</div>
				<div>
					<button class="confirmButton"> Confirm </button>
				</div>
			</div>
		</div>

		<div class="horizontalDivider"></div>
		<span class="title">Thumbnail Format</span>

		<div class="borderedBox">
			<label for="imageFormatSelect">Image Format</label>

			<!--
			Default <select> does not respect dark theme on webkit, it doesn't work with background-color 
			either should we blame Gnome devs or Apple devs for this one?
			-->
			<div class="selectContainer">
				<select id="imageFormatSelect" class="imageFormatSelect">
					<option value="png" class="imageFormatSelectOption">PNG</option>
					<option value="jpeg" class="imageFormatSelectOption">JPEG</option>
					<option value="avif" class="imageFormatSelectOption">AVIF (slow)</option>
				</select>

				<span class="cursedSelectIcon"> v </span>
			</div>

			<div class="smallHorizontalDivider"></div>
			Image Resolution

			<div class="resolutionContainer">
				<label class="resolutionInputLabel" for="resolutionInputWidth">Width</label>
				<input type="text" class="resolutionInput textInput monoFont" id="resolutionInputWidth" />

				<div class="smallVerticalDivider"></div>

				<label class="resolutionInputLabel" for="resolutionInputHeight">Height</label>
				<input type="text" class="resolutionInput textInput monoFont" id="resolutionInputHeight" />
			</div>

			<div class="smallHorizontalDivider"></div>
			<div class="bigImageExceptionCheckboxContainer">
				<input type="checkbox" id="bigImageExceptionCheckbox" class="bigImageExceptionCheckbox" />
				<label for="bigImageExceptionCheckbox">
					Generate larger resolution thumbnails for images with high aspect ratio (recommended)
				</label>
			</div>
		</div>
	</div>
{/await}

<style>
	.bigImageExceptionCheckboxContainer {
		display: flex;
		align-items: center;
		line-height: 15px;
	}
	.bigImageExceptionCheckbox {
		width: 15px;
		height: 15px;
		margin-right: 4px;
	}

	.smallVerticalDivider {
		width: 10px;
	}
	.resolutionInputLabel {
		padding-left: 2px;
		padding-right: 2px;
		background-color: var(--secondary-alt);
		display: flex;
		align-items: center;
		justify-content: center;
	}

	.resolutionInput {
		width: 100px;
	}

	.resolutionContainer {
		display: flex;
	}

	.selectContainer {
		display: flex;
	}

	.cursedSelectIcon {
		position: relative;
		right: 16px;
		user-select: none;
		-webkit-user-select: none;
	}

	.imageFormatSelectOption {
		appearance: none;
		background-color: var(--background);
		color: var(--text);
		border-radius: 0px;
	}
	.imageFormatSelect {
		appearance: none;
		background-color: var(--background);
		color: var(--text);
		width: 100px;
		border: 1px solid var(--secondary-alt);
		padding-left: 4px;
		padding-right: 4px;
		border-radius: 0px;
	}

	.imageFormatSelect:focus {
		outline: 1px solid var(--accent);
	}

	.smallHorizontalDivider {
		height: 10px;
	}

	.horizontalDivider {
		height: 20px;
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

	.confirmButton:hover {
		background-color: var(--accent-hover);
	}
	.bottomRow {
		display: flex;
	}

	.bottomRow div:last-child {
		margin-left: auto;
	}

	.thumbnails {
		display: flex;
		flex-grow: 100;
		flex-direction: column;
	}

	.title {
		font-weight: bold;
		font-size: 16px;
	}

	.textInput {
		background-color: var(--background);
		border: 1px solid var(--secondary-alt);
		padding-right: 6px;
		padding-left: 6px;
	}

	.dbPathInput {
		flex-grow: 100;
	}

	.textInput:focus {
		outline: 1px solid var(--accent);
	}

	.borderedBox {
		border: 1px solid var(--secondary-alt);
		padding: 4px;
		display: flex;
		flex-direction: column;
		font-size: smaller;
	}

	.details {
		font-size: small;
	}

	.pathInput {
		display: flex;
		flex-direction: row;
		margin-bottom: 4px;
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

	.fileSelectButton:hover {
		background-color: var(--secondary-alt);
	}

	.icon {
		display: flex;
		padding-left: 4px;
	}
	.monoFont {
		font-family: 'UbuntuMono';
	}
</style>
