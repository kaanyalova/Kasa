<script lang="ts">
	import { FileServer } from '$lib/serve';
	import { exists } from '@tauri-apps/plugin-fs';
	import { info } from '@tauri-apps/plugin-log';
	import Sidebar from '../component/Sidebar/Sidebar.svelte';
	import '../component/Sidebar/SideBarGlobals.scss';
	import WindowTitlebar from '../component/Decoration/WindowTitlebar.svelte';
	import InfiniteMediaGp from '../component/InfiniteMedia/InfiniteMediaGP.svelte';
	import MediaModal from '../component/MediaModal/MediaModal.svelte';
	import { MediaModalStatusStore } from '../component/MediaModal/MediaModalStatusStore.svelte';
	import MainPageTitlebarInsides from '../component/Decoration/MainPageTitlebarInsides.svelte';
</script>

<Sidebar>
	<div class="background">
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
			var(--page-content-width) - var(--sidebar-width)
		); /* doesn't work with vanilla css*/
	}

	.contentWrapper {
		padding: 10px;
	}

	.sideBarIcon {
		color: var(--text);
	}

	.content_ {
		height: calc(var(--page-content-height) - 32px);
		width: calc(var(--page-content-width) -var(--sidebar-width));

		overflow-y: scroll;
	}

	.modalWrapper {
		position: relative;
		top: 10vh;
	}
</style>
