<script lang="ts">
	import { isLayoutMenuActive } from '../DecorationStore.svelte';
	import '../../../colors.css';
	import Checkbox from '../../Shared/Checkbox.svelte';
	import { InfiniteMediaStore } from '../../InfiniteMedia/InfiniteMediaStore.svelte';

	function onChangeShowFileNamesOption(state: boolean) {
		InfiniteMediaStore.setShowNames(state);
	}
</script>

<dialog class="layoutMenu" open>
	<ul>
		<li class="layoutMenuItem">
			<Checkbox onCheck={onChangeShowFileNamesOption} state={InfiniteMediaStore.showNames}
			></Checkbox>
			<div class="layoutMenuItemDescription">Show file names under media</div>
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
	}

	.layoutMenuItemDescription {
		margin-left: 8px;
	}
</style>
