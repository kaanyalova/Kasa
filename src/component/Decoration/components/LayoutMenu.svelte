<script lang="ts">
	import { isLayoutMenuActive } from '../DecorationStore.svelte';
	import '../../../colors.css';
	import Checkbox from '../../Shared/Checkbox.svelte';
	import { InfiniteMediaStore } from '../../InfiniteMedia/InfiniteMediaStore.svelte';
	import { emit } from '@tauri-apps/api/event';

	function onChangeShowFileNamesOption(state: boolean) {
		InfiniteMediaStore.setShowNames(state);
	}

	function onThumbnailSizeChange() {}
</script>

<dialog class="layoutMenu" open>
	<ul>
		<li class="layoutMenuItem">
			<Checkbox onCheck={onChangeShowFileNamesOption} state={InfiniteMediaStore.showNames}
			></Checkbox>
			<button
				class="layoutMenuItemDescription"
				onclick={() => InfiniteMediaStore.setShowNames(!InfiniteMediaStore.showNames)}
				>Show file names under media</button
			>
		</li>

		<li class="layoutMenuItem">
			<div class="layoutMenuItemDescription">Thumbnail Scale</div>

			<div class="thumbnailScaleSliderContainer">
				<input
					class="thumbnailScaleSlider"
					type="range"
					min="0.5"
					max="3"
					step="0.2"
					bind:value={InfiniteMediaStore.thumbnailScale}
				/>
			</div>

			<div class="thumbnailScaleSliderValue">
				{InfiniteMediaStore.thumbnailScale}
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
		left: calc(100vw - 300px - 475px);
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
	}

	.thumbnailScaleSlider {
		padding: 4px;
	}

	.thumbnailScaleSliderContainer {
		border: 1px solid var(--secondary-alt);
		padding-left: 4px;
		padding-right: 4px;
	}
</style>
