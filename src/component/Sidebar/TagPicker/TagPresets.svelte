<script lang="ts">
	import { commands } from '$lib/tauri_bindings';
	import { onMount } from 'svelte';

	let presets = $state([]);

	onMount(() => {});
</script>

<!--
 - Search on input


-->

<div class="tagPresetPicker">
	<input type="text" class="tagPresetPickerInput" placeholder="WIP" />
	<div class="expandButton">▼</div>

	<div class="items"></div>
</div>

<style>
	.tagPresetPicker {
		display: flex;
		flex-direction: row;
	}
	.tagPresetPickerInput {
		width: 232px;
		height: 32px;
		background-color: var(--secondary-alt);
		color: var(--text-color);
		outline: none;
		margin: 8px;
		border-radius: 8px 0px 0px 8px;
		padding: 4px;
		padding-left: 8px;
		flex-grow: 1;
		margin-right: 0;
	}
	.tagPresetPickerInput:focus {
		outline: 1px solid var(--accent);
	}

	.tagPresetPickerInput::placeholder {
		color: var(--text-color);
		opacity: 0.8;
	}

	.expandButton {
		background-color: var(--accent);
		width: 24px;
		height: 32px;
		align-self: center;
		border-radius: 0px 8px 8px 0px;
		display: flex;
		align-items: center;
		justify-content: center;
		color: black;
		cursor: pointer;
	}
</style>
