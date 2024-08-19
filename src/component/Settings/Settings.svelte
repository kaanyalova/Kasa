<script lang="ts">
	import SettingsSidebar from './SettingsSidebar.svelte';

	let selectedCategory = $state('');

	function updateSelectedCategory(category: string) {
		selectedCategory = category;
	}
</script>

<SettingsSidebar {updateSelectedCategory} {selectedCategory}>Hello</SettingsSidebar>

<div class="settings"></div>

<style>
	.settings {
		color: var(--text);
	}
</style>
