<script lang="ts">
	import '../../../colors.css';
	import Checkbox from '../../Shared/Checkbox.svelte';
	import { InfiniteMediaStore } from '../../InfiniteMedia/InfiniteMediaStore.svelte';
	import { emit } from '@tauri-apps/api/event';
	import { commands } from '$lib/tauri_bindings';

	async function toggleShowFileNamesOption() {
		InfiniteMediaStore.showNames = !InfiniteMediaStore.showNames;
		await commands.setConfigValueBool('Layout', 'show_filenames', InfiniteMediaStore.showNames);
	}

	let thumbnailScaleDisplay = $state(InfiniteMediaStore.thumbnailScale);

	let thumbnailScaleTimer: number = 0;
	function onChangeThumbnailScale(scale: number) {
		thumbnailScaleDisplay = scale;
		clearTimeout(thumbnailScaleTimer);
		thumbnailScaleTimer = setTimeout(async () => {
			InfiniteMediaStore.thumbnailScale = scale;
			commands.setConfigValueF64('Layout', 'thumbnail_scale', scale);
		}, 200);
	}
</script>

<dialog class="layoutMenu" open>
	<ul>
		<li class="layoutMenuItem">
			<Checkbox
				onCheck={async () => await toggleShowFileNamesOption()}
				state={InfiniteMediaStore.showNames ?? false}
			></Checkbox>
			<button
				class="layoutMenuItemDescription"
				onclick={async () => {
					await toggleShowFileNamesOption();
				}}>Show file names under media</button
			>
		</li>

		<li class="layoutMenuItem">
			<div class="propertyText">Thumbnail Scale</div>

			<div class="thumbnailScaleSliderContainer">
				<input
					class="thumbnailScaleSlider"
					type="range"
					min="0.5"
					max="3"
					step="0.2"
					bind:value={() => thumbnailScaleDisplay, (v) => onChangeThumbnailScale(v ?? 1.0)}
				/>
			</div>

			<div class="thumbnailScaleSliderValue">
				<span class="thumbnailScaleText">
					{thumbnailScaleDisplay}
				</span>
			</div>
		</li>
	</ul>
</dialog>

<style>
	.layoutMenu {
		width: 300px;
		height: 200px;
		background-color: var(--background);
		color: var(--text);
		border: 1px solid var(--secondary-alt);
		position: fixed;
		left: calc(100vw - 350px);
		top: calc(32px + 8px);
		border-radius: 8px;
		padding: 8px;
	}

	.layoutMenuItem {
		display: flex;
		text-align: center;
		align-items: center;
		padding: 2px;
	}

	.layoutMenuItemDescription {
		margin: 4px;
		font-size: small;
	}

	.propertyText {
		font-size: 12px;
		background-color: var(--secondary-alt);
		padding-top: 8px;
		padding-bottom: 8px;
		padding-left: 2px;
		padding-right: 2px;
		text-align: center;
		border: 1px solid var(--border);
		border-right: none;
		border-radius: 4px 0px 0px 4px;
	}

	.thumbnailScaleSliderValue {
		background-color: var(--accent);
		color: var(--text-opposite);
		padding: 2px;
		font-weight: bold;
		outline: 1px solid var(--secondary-alt);
		width: 36px;
		height: 36px;
		text-align: center;
		border-radius: 0px 4px 4px 0px;
	}

	.thumbnailScaleSlider {
		padding: 4px;
		position: relative;
		top: 2px;
	}

	.thumbnailScaleSliderContainer {
		border: 1px solid var(--secondary-alt);
		padding-left: 4px;
		padding-right: 4px;
	}

	.thumbnailScaleText {
		position: relative;
		top: 3px;
	}
</style>
