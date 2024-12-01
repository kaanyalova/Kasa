<script lang="ts">
	import { FileServer } from '$lib/serve';
	import { exists } from '@tauri-apps/plugin-fs';
	import {} from '@tauri-apps/plugin-fs';
	import { info } from '@tauri-apps/plugin-log';
	import { onDestroy, onMount, tick } from 'svelte';
	import Sidebar from '../component/Sidebar/Sidebar.svelte';
	import '../component/Sidebar/SideBarGlobals.scss';
	import { invoke } from '@tauri-apps/api/core';
	import WindowTitlebar from '../component/Decoration/WindowTitlebar.svelte';
	import SidebarIcon from '../component/Vector/SidebarIcon.svelte';
	import InfiniteMediaGp from '../component/InfiniteMedia/InfiniteMediaGP.svelte';
	import MediaModal from '../component/MediaModal/MediaModal.svelte';
	import { MediaModalStatusStore } from '../component/MediaModal/MediaModalStatusStore.svelte';
	import { sidebarStore } from '../component/Sidebar/SidebarStore.svelte';
	import MainPageTitlebarInsides from '../component/Decoration/MainPageTitlebarInsides.svelte';
	import { commands } from '$lib/tauri_bindings';

	let fileServer: FileServer;

	onMount(async () => {
		info('conneting  to db');
		/*

		const db_env_var: string = await invoke('get_env_var', {
			var: 'KASA_DB_PATH'
		});

		info(`db path from env: ${db_env_var}`);

		await invoke('connect_to_db', {
			dbPath: db_env_var
		});
		*/

		await commands.connectDbs();
	});

	function toLight() {}
</script>

<Sidebar>
	<div class="background" class:backgroundSidebarInactive={!sidebarStore.isActive}>
		<WindowTitlebar platform="gnome">
			<MainPageTitlebarInsides></MainPageTitlebarInsides>
		</WindowTitlebar>
		<div class="content_">
			{#if MediaModalStatusStore.isOpen}
				<div class="modalWrapper">
					<MediaModal imageHash={MediaModalStatusStore.hash}></MediaModal>
				</div>
			{/if}
			<InfiniteMediaGp></InfiniteMediaGp>
		</div>
	</div>
</Sidebar>

<style lang="scss">
	.background {
		background-color: var(--background);
		width: calc(
			100vw - var(--sidebar-width) - var(--resize-bar-padding) - var(--window-border-size) * 2
		); /* doesn't work with vanilla css*/
		min-width: 100vh;
	}

	.backgroundSidebarInactive {
		/* The sidebar resizeBarPadding would not be pixel perfect otherwise */
		width: calc(100vw - var(--sidebar-width) - var(--window-border-size) * 2) !important;
	}

	.contentWrapper {
		padding: 10px;
	}

	.sideBarIcon {
		color: var(--text);
	}

	.content_ {
		height: calc(100vh - 32px - var(--window-border-size) * 2);
		width: calc(
			100vw - var(--sidebar-width) - var(--resize-bar-padding) - var(--window-border-size) * 2
		);

		overflow-y: scroll;
	}

	.modalWrapper {
		position: relative;
		top: 10vh;
	}
</style>
