<script lang="ts">
	import { tick } from 'svelte';
	import Tick from '../../Vector/Tick.svelte';
	import Cross from '../../Vector/Cross.svelte';
	import type { MouseEventHandler } from 'svelte/elements';
	import { stat } from '@tauri-apps/plugin-fs';
	import { trace } from '@tauri-apps/plugin-log';

	let { tagName, onCheck, checkboxState }: TagPickerCheckBoxProps = $props();

	function onClick() {
		if (checkboxState === 'selected') {
			checkboxState = 'unselected';
			onCheck(checkboxState, tagName);
		} else {
			checkboxState = 'selected';
			onCheck(checkboxState, tagName);
		}
	}
	function onContextMenu(e: Event) {
		e.preventDefault();
		if (checkboxState === 'exclude') {
			checkboxState = 'unselected';
			onCheck(checkboxState, tagName);
		} else {
			checkboxState = 'exclude';
			onCheck(checkboxState, tagName);
		}
	}
</script>

<div class="tagPickerCheckBox">
	<button class="checkBox" onclick={() => onClick()} oncontextmenu={(e) => onContextMenu(e)}>
		{#if checkboxState === 'selected'}
			<Tick height={16} width={16}></Tick>
		{:else if checkboxState === 'exclude'}
			<Cross height={20} width={20}></Cross>
		{:else}{/if}
	</button>
</div>

<style>
	.tagPickerCheckBox {
		display: flex;
		align-items: center;
		justify-content: center;
		margin-right: 8px;
	}
	.checkBox {
		height: 20px;
		width: 20px;
		display: flex;
		align-items: center;
		justify-content: center;
		fill: var(--text);
		border: 1px solid var(--border-on-secondary-alt);
		background-color: var(--secondary-alt);
	}
</style>
