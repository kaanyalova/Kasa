<script lang="ts">
	import WindowControls from '../../component/Decoration/WindowControls.svelte';
	import WindowTitlebar from '../../component/Decoration/WindowTitlebar.svelte';
	import { SettingsCategories } from '../../component/Settings/Settings';
	import Settings from '../../component/Settings/Settings.svelte';
	import SettingsSidebar from '../../component/Settings/Sidebar/SettingsSidebar.svelte';

	let selectedCategory: string = $state(SettingsCategories.Database);

	function updateSelectedCategory(category: string) {
		selectedCategory = category;
	}
</script>

<SettingsSidebar {selectedCategory} {updateSelectedCategory}>
	<div class="background">
		<WindowTitlebar platform="gnome">
			<div class="titlebarInsides" data-tauri-drag-region></div>
		</WindowTitlebar>
		<Settings category={selectedCategory}></Settings>
	</div>
</SettingsSidebar>

<style>
	.background {
		width: calc(100vw - 200px);
		background-color: var(--background);
		display: flex;
		flex-direction: column;
	}

	.titlebarInsides {
		color: var(--text);
	}
</style>
