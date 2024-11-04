<script lang="ts">
	import Database from './Screens/Database.svelte';
	import Downloaders from './Screens/Downloaders.svelte';
	import Indexers from './Screens/Indexers/Indexers.svelte';
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

	{#if selectedCategory === SettingsCategories.Downloaders}
		<Downloaders></Downloaders>
	{/if}
	{#if selectedCategory === SettingsCategories.Indexers}
		<Indexers></Indexers>
	{/if}
</div>

<style>
	.settings {
		color: var(--text);
		display: flex;
		flex-grow: 1;
		padding: 6px;
		font-size: small;
	}
</style>
