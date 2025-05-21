<script lang="ts">
	import Bars from '../Vector/Bars.svelte';
	import Moon from '../Vector/Moon.svelte';
	import SidebarIcon from '../Vector/SidebarIcon.svelte';
	import { sidebarStore } from '../Sidebar/SidebarStore.svelte';
	import { InfiniteMediaStore } from '../InfiniteMedia/InfiniteMediaStore.svelte';
	import '../../fonts.css';
	import { save } from '@tauri-apps/plugin-dialog';
	import { error } from '@tauri-apps/plugin-log';
	import { commands } from '$lib/tauri_bindings';
	import { emit } from '@tauri-apps/api/event';
	import {
		openFilePickerWithSaveDialog,
		openFilePickerWithSelectDialog
	} from '$lib/openFilePicker';
	import { onMount } from 'svelte';
	import TagPickerCheckBox from '../Sidebar/TagPicker/TagPickerCheckBox.svelte';
	import { isLayoutMenuActive } from './DecorationStore.svelte';
	import LayoutMenu from './components/LayoutMenu.svelte';

	let dbName = $state('');

	onMount(async () => {
		const config = await commands.getConfig();
		const dbPath = config.Database.db_path;

		dbName = dbPath.split('/').pop() || '';
	});

	function handleSidebarButton() {
		sidebarStore.toggle();
	}

	async function onNewDb() {
		const paths = await openFilePickerWithSaveDialog('Kasa Database', '*.kasa', 'default.kasa');
		const path = paths[0];

		console.log('Selected path:', path);

		if (!path) {
			error('File picker failed to select file');
			return;
		}

		await commands.setDbPath(path);
		dbName = path.split('/').pop() || '';

		await emit('dbs_updated');
	}

	async function onOpenDb() {
		const paths = await openFilePickerWithSelectDialog('Kasa Database', '*.kasa');
		const path = paths[0];

		console.log('Selected path:', path);

		if (!path) {
			error('File picker failed to select file');
			return;
		}

		dbName = path.split('/').pop() || '';
		await commands.setDbPath(path);

		await emit('dbs_updated');
	}

	function onOpenLayoutSettings() {
		isLayoutMenuActive.value = !isLayoutMenuActive.value;
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

			<button class="option newDb" onclick={async () => onNewDb()}> New DB </button>
			<div class="iconPadding" data-tauri-drag-region></div>

			<button class="option" onclick={async () => await onOpenDb()}>Open DB ▼</button>
			<div class="iconPadding" data-tauri-drag-region></div>
		</div>

		<div class="insidesFiller"></div>
		<div class="title" data-tauri-drag-region>Kasa</div>
		<div class="iconPadding" data-tauri-drag-region></div>

		<div class="dbInfo" data-tauri-drag-region>{dbName}</div>
		<div class="insidesFiller"></div>

		<button class="layoutSettings" onclick={() => onOpenLayoutSettings()}>Layout ▼</button>
		{#if isLayoutMenuActive.value}
			<LayoutMenu></LayoutMenu>
		{/if}

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
	.option {
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

	.newDb {
		padding-right: 4px;
		padding-left: 4px;
	}

	.layoutSettings {
		background-color: var(--accent);
		margin-left: 10px;
		margin-right: 10px;
		padding-left: 4px;
		padding-right: 4px;
		border-radius: 4px;
	}
</style>
