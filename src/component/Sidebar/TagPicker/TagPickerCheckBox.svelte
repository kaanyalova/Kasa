<script lang="ts">
	import { tick } from 'svelte';
	import Tick from '../../Vector/Tick.svelte';
	import Cross from '../../Vector/Cross.svelte';
	import type { MouseEventHandler } from 'svelte/elements';
	import { stat } from '@tauri-apps/plugin-fs';

	let { tagName, onCheck, state }: TagPickerCheckBoxProps = $props();

	$effect(() => {
		onCheck(state, tagName);
	});

	function onClick() {
		if (state === 'selected') {
			state = 'unselected';
		} else {
			state = 'selected';
		}
	}
	function onContextMenu(e: Event) {
		e.preventDefault();
		if (state === 'exclude') {
			state = 'unselected';
		} else {
			state = 'exclude';
		}
	}
</script>

<div class="tagPickerCheckBox">
	<button class="checkBox" onclick={() => onClick()} oncontextmenu={(e) => onContextMenu(e)}>
		{#if state === 'selected'}
			<Tick height={16} width={16}></Tick>
		{:else if state === 'exclude'}
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
