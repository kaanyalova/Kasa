<script lang="ts">
	import Bars from '../Vector/Bars.svelte';
	import Moon from '../Vector/Moon.svelte';
	import SidebarIcon from '../Vector/SidebarIcon.svelte';
	import { sidebarStore } from '../Sidebar/SidebarStore.svelte';
	import { InfiniteMediaStore } from '../InfiniteMedia/InfiniteMediaStore.svelte';
	import '../../fonts.css';

	function handleSidebarButton() {
		sidebarStore.toggle();
	}
</script>

<div class="insides" data-tauri-drag-region>
	{#if InfiniteMediaStore.onSelectMode}
		<div class="selectionText">
			Selected <strong> {InfiniteMediaStore.selectedHashes.length} </strong> Items
		</div>

		<div class="insides"></div>
	{:else}
		<div class="iconContainer">
			<button onclick={handleSidebarButton}>
				<div class="iconContainer">
					<SidebarIcon height={20} width={20}></SidebarIcon>
				</div>
			</button>
			<div class="iconPadding"></div>

			<!--
         <Moon height={20} width={20}></Moon>
		<div class="iconPadding"></div>-->

			<div class="open">Open DB ▼</div>
			<div class="iconPadding" data-tauri-drag-region></div>

			<div class="open">Select ▼</div>
		</div>

		<div class="insidesFiller"></div>
		<div class="title" data-tauri-drag-region>Kasa</div>
		<div class="iconPadding" data-tauri-drag-region></div>

		<div class="dbInfo" data-tauri-drag-region>memes.db</div>
		<div class="insidesFiller"></div>

		<div class="jobs">Running Job: Indexing Files</div>
		<div class="iconPadding" data-tauri-drag-region></div>
		<Bars width={15} height={15}></Bars>
	{/if}
</div>

<style>
	.insides {
		display: flex;
		flex-grow: 1;
		justify-content: center;
		align-items: center;
	}
	.insidesFiller {
		flex-grow: 1;
	}

	.iconContainer {
		display: flex;
		align-items: center;
		justify-content: center;
		margin-left: 4px;
		margin-right: 4px;
	}

	svg {
		fill: var(--text);
	}
	.iconPadding {
		width: 10px;
	}
	.open {
		color: black;
		background: var(--accent);
		padding-left: 2px;
		padding-right: 2px;
		border-radius: 4px;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.selectionText {
		background-color: var(--primary);
		padding-left: 4px;
		padding-right: 4px;
		margin: 4px;
		border-radius: 4px;
	}

	.jobs {
		color: black;
		background: var(--accent);
		padding-left: 2px;
		padding-right: 2px;
		border-radius: 4px;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.title {
		color: var(--text);
	}
	.dbInfo {
		color: var(--text-opposite);
		background: var(--primary);
		border-radius: 4px;
		padding-left: 2px;
		padding-right: 2px;
	}
</style>
