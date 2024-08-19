<script>
	import WindowControls from '../../component/Decoration/WindowControls.svelte';
	import WindowTitlebar from '../../component/Decoration/WindowTitlebar.svelte';
	import Settings from '../../component/Settings/Settings.svelte';
	import SettingsSidebar from '../../component/Settings/SettingsSidebar.svelte';
</script>

<SettingsSidebar>
	<div class="background">
		<WindowTitlebar platform="gnome">
			<div class="titlebarInsides" data-tauri-drag-region></div>
		</WindowTitlebar>
	</div>
</SettingsSidebar>

<style>
	.background {
		width: calc(100vw - 200px);
		background-color: var(--background);
	}

	.titlebarInsides {
		color: var(--text);
	}
</style>
