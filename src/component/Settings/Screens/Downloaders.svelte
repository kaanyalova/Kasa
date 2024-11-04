<script lang="ts">
	import BorderedBox from '../../Shared/BorderedBox.svelte';
	import { DividerSizes } from '../../Shared/Dividers/DividerSizes';
	import HorizontalDivider from '../../Shared/Dividers/HorizontalDivider.svelte';
	import VerticalDivider from '../../Shared/Dividers/VerticalDivider.svelte';
	import PathInput from '../Shared/PathInput.svelte';
	import Title from '../Shared/Title.svelte';

	type GalleryDlConfigOption = {
		keys: object;
		value: string;
	};

	function onPathConfirm() {}

	let path: string = $state('');
</script>

<div class="downloaders">
	<Title>gallery-dl Paths</Title>
	<BorderedBox padding={4}>
		Output Path
		<PathInput details="" onConfirm={onPathConfirm} bind:path></PathInput>

		<HorizontalDivider height={DividerSizes.Normal}></HorizontalDivider>

		Config Path (optional)
		<PathInput details="" onConfirm={onPathConfirm} bind:path></PathInput>
	</BorderedBox>

	<HorizontalDivider height={DividerSizes.Normal}></HorizontalDivider>

	<Title>Indexer Options</Title>
	<BorderedBox padding={4}>
		<div class="checkBoxContainer">
			<input type="checkbox" id="gdlSaveAllMeta" class="checkBox" />
			<label for="gdlSaveAllMeta"> Save all the data from media imported using gallery-dl </label>
		</div>
	</BorderedBox>
</div>

<style>
	.downloaders {
		display: flex;
		flex-grow: 100;
		flex-direction: column;
	}

	.checkBoxContainer {
		display: flex;
		align-items: center;
		line-height: 15px;
	}
	.checkBox {
		width: 15px;
		height: 15px;
		margin-right: 4px;
	}
</style>
