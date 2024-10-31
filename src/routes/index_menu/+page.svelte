<script lang="ts">
	import WindowControls from '../../component/Decoration/WindowControls.svelte';
	import WindowTitlebar from '../../component/Decoration/WindowTitlebar.svelte';
	import IndexMenu from '../../component/IndexMenu/IndexMenu.svelte';
	import { SettingsCategories } from '../../component/Settings/Settings';
	import Settings from '../../component/Settings/Settings.svelte';
	import SettingsSidebar from '../../component/Settings/Sidebar/SettingsSidebar.svelte';

	let selectedCategory: string = $state(SettingsCategories.Thumbnails);

	function updateSelectedCategory(category: string) {
		selectedCategory = category;
	}
</script>

<div class="background">
	<WindowTitlebar platform="gnome"></WindowTitlebar>
	<IndexMenu></IndexMenu>
</div>

<style>
	.background {
		width: calc(100vw - 200px);
		background-color: var(--background);
		display: flex;
		flex-direction: column;
		height: 100vh;
		width: 100vw;
	}

	.titlebarInsides {
		color: var(--text);
	}
</style>
