<script lang="ts">
	import { commands } from '$lib/tauri_bindings';
	import { onMount } from 'svelte';

	onMount(() => {
		const allTags = commands.getTags;
	});
</script>

<div class="newTagPicker newTagPickerElement">
	<input class="newTagPickerElement tagSearch" type="text" />
</div>

<style>
	.newTagPicker {
		height: 200px;
		width: 300px;
		background-color: var(--background);
		position: relative;
		z-index: 10;
		padding: 8px;
		border: 1px solid var(--secondary-alt);
	}

	.tagSearch {
		width: 100%;
		background-color: var(--secondary-alt);
		color: var(--text);
		outline: none;
		padding-left: 8px;
		padding-right: 8px;
		padding: 4px;
		border-radius: 8px;
	}

	.tagSearch:focus {
		border: 1px solid var(--accent);
	}
</style>
