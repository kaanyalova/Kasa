<script lang="ts">
	import { commands } from '$lib/tauri_bindings';
	import { SearchStore } from '../../Sidebar/Search/SearchStore.svelte';
	import Cross from '../../Vector/Cross.svelte';
	import CrossFilled from '../../Vector/CrossFilled.svelte';
	import Trash from '../../Vector/Trash.svelte';
	import TrashGnome from '../../Vector/TrashGnome.svelte';
	import TrashWithQuestionMark from '../../Vector/TrashWithQuestionMark.svelte';
	import { MediaModalStatusStore } from '../MediaModalStatusStore.svelte';

	let { name, color, onDelete }: TagProps = $props();

	let clickCount = $state(0);
	let clickColdown: number = $state(0);

	let _delete = $state(false);

	function onClick() {
		clearTimeout(clickColdown);
		if (clickCount === 1) {
			onDelete(name);
			clickCount = 0;
		} else {
			clickCount += 1;
		}

		clickColdown = setTimeout(() => {
			clickCount = 0;
		}, 2000);
	}

	// TODO: show the user when they are Ctrl+hovering,
	function onCtrlClick() {
		SearchStore.searchContents = name;
		commands.search(name);
		MediaModalStatusStore.close();
	}
</script>

{#if !_delete}
	<button
		class="tag"
		onclick={(e) => {
			if (e.ctrlKey) {
				onCtrlClick();
			} else {
				onClick();
			}
		}}
	>
		<div class="tagButton">
			{#if clickCount === 1}
				<!--
				On second click, show the delete icon
				-->
				<div class="tagButton trashButton">
					<TrashGnome height={12} width={12}></TrashGnome>
				</div>
			{:else}
				<div class="tagButton xButton">
					<Cross height={16} width={16}></Cross>
				</div>
			{/if}
		</div>

		<div class="name">{name}</div>

		<div class="coloredPart" style="background-color:{color};"></div>
	</button>
{/if}

<style>
	.tag {
		border: 1px solid var(--border);
		display: flex;
		align-items: center;
		justify-content: center;
		margin: 4px;
		border-radius: 0px 2px 2px 0px;
		cursor: pointer;
	}

	.tag:hover {
		background-color: var(--secondary-alt);
	}

	.tagButton {
		fill: var(--text);
		align-items: center;
		justify-content: center;
		display: flex;
		width: 16px;
		position: relative;
		height: 19px;
		top: 1px;
	}

	.name {
		font-size: small;
		word-wrap: break-word;
		max-width: 238px;
		margin-right: 4px;
		text-align: center;
	}

	.xButton {
		position: relative;
	}

	.coloredPart {
		width: 4px;
		height: 25px;
		border-radius: 0px 2px 2px 0px;
	}
</style>
