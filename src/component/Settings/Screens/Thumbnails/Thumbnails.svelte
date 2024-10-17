<script lang="ts">
	import { commands } from '$lib/tauri_bindings';
	import FileManager from '../../../Vector/FileManager.svelte';
	import '../../../../fonts.css';
	import Select from 'svelte-select';
	import { comma } from 'postcss/lib/list';
	import type { ThumbsDBInfo } from '$lib/tauri_bindings';
	import { dataDir } from '@tauri-apps/api/path';
	import { isNumericString } from '$lib/isNumbericString';
	import { debug, info, trace } from '@tauri-apps/plugin-log';
	import BorderedBox from '../../../Shared/BorderedBox.svelte';
	import HorizontalDivider from '../../../Shared/Dividers/HorizontalDivider.svelte';
	import { DividerSizes } from '../../../Shared/Dividers/DividerSizes';
	import VerticalDivider from '../../../Shared/Dividers/VerticalDivider.svelte';
	import ResolutionInput from './ResolutionInput.svelte';

	const THUMBNAIL_MAX = 8192;

	let promise = $state(onLoad());

	let db_path = $state('');
	let thumb_height = $state(0);
	let thumb_width = $state(0);
	let thumb_format = $state('');
	let image_count = $state(0);
	let db_size = $state('');

	async function onLoad(): Promise<ThumbsDBInfo | null> {
		const info = await commands.getThumbsDbInfo();
		thumb_height = info!.height;
		thumb_width = info!.width;
		thumb_format = info!.format;
		db_path = info!.path;
		db_size = info!.size;
		image_count = info!.image_count;

		return info;
	}

	async function onConfirmThumbnailDatabase() {
		await commands.setConfigValue('Thumbnails', 'thumbs_db_path', db_path);
		await commands.connectDbs();
		await onLoad();
	}

	$effect(() => {
		console.log(db_path);
	});

	// Handle thumbnail resolution changes
	$effect(() => {
		thumb_height;
		thumb_width;

		if (isNumericString(thumb_height.toString())) {
			const height = Math.min(THUMBNAIL_MAX, thumb_height);
			const width = Math.min(THUMBNAIL_MAX, thumb_width);
			commands.setConfigResolutionValue(height, width);
		}
	});

	// Handle thumbnail file format changes
	$effect(() => {
		thumb_format;
		// Don't send the initial empty value breaking the config parsing
		if (thumb_format !== '') {
			// They are lowercase on config file
			commands.setConfigValue('Thumbnails', 'thumbnail_format', thumb_format.toLowerCase());
		}
	});
</script>

{#await promise then info}
	<div class="thumbnails">
		<span class="title">Thumbnail Database</span>
		<BorderedBox>
			<div class="pathInput">
				<input type="text" bind:value={db_path} class="dbPathInput textInput monoFont" />
				<button class="fileSelectButton">
					<span class="details">Browse</span>
					<div class="icon">
						<FileManager height={18} width={18}></FileManager>
					</div></button
				>
			</div>
			<div class="bottomRow">
				<div class="details dbInfoText">{image_count} Images {db_size}</div>
				<div>
					<button class="confirmButton" onclick={onConfirmThumbnailDatabase}> Confirm </button>
				</div>
			</div>
		</BorderedBox>

		<HorizontalDivider height={DividerSizes.Normal}></HorizontalDivider>
		<span class="title">Thumbnail Format</span>

		<BorderedBox>
			<label for="imageFormatSelect">Image Format</label>

			<!--
			Default <select> does not respect dark theme on webkit, it doesn't work with background-color 
			either should we blame Gnome devs or Apple devs for this one?
			-->
			<div class="selectContainer">
				<select id="imageFormatSelect" class="imageFormatSelect" bind:value={thumb_format}>
					<option value="PNG" class="imageFormatSelectOption">PNG</option>
					<option value="JPEG" class="imageFormatSelectOption">JPEG</option>
					<option value="AVIF" class="imageFormatSelectOption">AVIF (slow)</option>
				</select>

				<span class="cursedSelectIcon"> v </span>
			</div>

			<HorizontalDivider height={DividerSizes.Small}></HorizontalDivider>
			Image Resolution

			<div class="resolutionContainer">
				<ResolutionInput bind:_value={thumb_width} max_size={THUMBNAIL_MAX}></ResolutionInput>

				<VerticalDivider width={DividerSizes.Small}></VerticalDivider>

				<ResolutionInput bind:_value={thumb_height} max_size={THUMBNAIL_MAX}></ResolutionInput>
			</div>

			<HorizontalDivider height={DividerSizes.Normal}></HorizontalDivider>
			<div class="bigImageExceptionCheckboxContainer">
				<input type="checkbox" id="bigImageExceptionCheckbox" class="bigImageExceptionCheckbox" />
				<label for="bigImageExceptionCheckbox">
					Generate larger resolution thumbnails for images with high aspect ratio (recommended)
				</label>
			</div>
		</BorderedBox>
	</div>
{/await}

<style>
	.resolutionInputContainerInner {
		display: flex;
		height: min-content;
	}

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

	.dbPathInput {
		flex-grow: 100;
	}

	.textInput {
		background-color: var(--background);
		border: 1px solid var(--secondary-alt);
		padding-right: 6px;
		padding-left: 6px;
	}

	.textInput:focus {
		outline: 1px solid var(--accent);
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
