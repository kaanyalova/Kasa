<script lang="ts">
	import Database from './Screens/Database.svelte';
	import Thumbnails from './Screens/Thumbnails/Thumbnails.svelte';
	import { SettingsCategories } from './Settings';

	let { category: selectedCategory } = $props();
</script>

<div class="settings">
	{#if selectedCategory === SettingsCategories.Thumbnails}
		<Thumbnails></Thumbnails>
	{/if}
	{#if selectedCategory === SettingsCategories.Database}
		<Database></Database>
	{/if}
</div>

<style>
	.settings {
		color: var(--text);
		display: flex;
		flex-grow: 100;
		padding: 6px;
	}
</style>
