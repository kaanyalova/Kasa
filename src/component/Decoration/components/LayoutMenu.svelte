<script lang="ts">
	import '../../../colors.css';
	import Checkbox from '../../Shared/Checkbox.svelte';
	import { InfiniteMediaStore } from '../../InfiniteMedia/InfiniteMediaStore.svelte';
	import { emit } from '@tauri-apps/api/event';
	import { commands } from '$lib/tauri_bindings';

	async function toggleShowFileNamesOption() {
		InfiniteMediaStore.setShowNames(!InfiniteMediaStore.showNames);
		await commands.setConfigValueBool('Layout', 'show_filenames', InfiniteMediaStore.showNames);
	}

	let thumbnailScaleDisplay = $state(InfiniteMediaStore.thumbnailScale);

	let thumbnailScaleTimer: number = 0;
	function onChangeThumbnailScale(scale: number) {
		thumbnailScaleDisplay = scale;
		clearTimeout(thumbnailScaleTimer);
		thumbnailScaleTimer = setTimeout(async () => {
			InfiniteMediaStore.setThumbnailScale(scale);
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
			<div class="scaleContainer">
				<div class="scaleLabel">Thumbnail Scale</div>

				<div class="scaleContent">
					<div class="scaleIcon">
						<span class="icon-[material-symbols--photo-size-select-small] w-5 h-5"></span>
					</div>

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
				</div>
			</div>
		</li>

		<li class="layoutMenuItem"></li>
	</ul>
</dialog>

<style lang="scss">
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

	.scaleIcon {
		background-color: var(--accent);
		color: var(--text-opposite);
		border: 1px solid var(--accent-border);
		width: 36px;
		height: 36px;
		display: flex;
		align-items: center;
		justify-content: center;
		flex-shrink: 0;
	}

	.thumbnailScaleSliderValue {
		background-color: var(--secondary-alt);
		color: var(--text);
		padding: 2px;
		font-weight: bold;
		width: 36px;
		height: 36px;
		text-align: center;
		border-radius: 0px 4px 4px 0px;
	}

	.thumbnailScaleSlider {
		padding: 4px;
		position: relative;
		top: 2px;
		width: 100%;
	}

	.thumbnailScaleSliderContainer {
		border: 1px solid var(--secondary-alt);
		padding-left: 4px;
		padding-right: 4px;
		width: 100%;
	}

	.thumbnailScaleText {
		position: relative;
		top: 3px;
	}

	.scaleContent {
		display: flex;
		width: 100%;
	}

	.scaleLabel {
		text-align: left;
		font-size: smaller;
	}

	.scaleContainer {
		width: 100%;
	}
</style>
