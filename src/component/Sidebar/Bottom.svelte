<script>
	import { WebviewWindow } from '@tauri-apps/api/webviewWindow';
	import SettingsButton from './SettingsButton.svelte';
	import SidebarButton from './SidebarButton.svelte';
	import './SideBarGlobals.scss';
	import { info } from '@tauri-apps/plugin-log';

	function onSettingsButtonClick() {
		const webView = new WebviewWindow('settings', {
			title: 'Settings',
			url: 'settings',
			decorations: false
		});
	}
</script>

<!-- 
TODO
When a process is running the settings button should get smaller and a pie (like gnome) should be shown as progress indicator-->
<div class="bottom">
	<ul class="sidebarContent">
		<li class="settingsContainer">
			<SettingsButton
				text="Settings"
				onClick={() => {
					onSettingsButtonClick();
				}}
			>
				<span class="icon-[material-symbols--settings] w-5 h-5"></span>
			</SettingsButton>
		</li>
	</ul>
</div>

<style>
	li {
		display: flex;
	}
	ul {
		width: var(--sidebar-content-width);
	}

	.bottom {
		margin: 10px;
	}
</style>
