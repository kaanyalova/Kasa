<script lang="ts">
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
	import { onNewDb, onOpenDb } from '$lib/dbSelection';
	import LayoutMenu from './components/LayoutMenu.svelte';
	import ConnectMenu from './components/ConnectMenu.svelte';

	let dbName = $state('');
	let isLayoutMenuActive = $state(false);
	let isConnectMenuActive = $state(false);

	onMount(async () => {
		const config = await commands.getConfig();
		const dbPath = config.Database.db_path;

		dbName = dbPath.split('/').pop() || '';
	});

	function handleSidebarButton() {
		sidebarStore.toggle();
	}

	function onOpenLayoutSettings() {
		isLayoutMenuActive = !isLayoutMenuActive;
	}

	// todo actually check if the connection is secure
	async function isConnectionSecure(): Promise<boolean> {
		const isRemote = await commands.isRemoteDb();
		if (isRemote) {
			const url_ = await commands.getRemoteServerUrl();

			if (url_.startsWith('http://')) {
				return false;
			}
		}

		return true;
	}
</script>

<div class="insides" data-tauri-drag-region>
	{#if InfiniteMediaStore.onSelectMode}
		<div class="selectionText">
			Selected <strong> {InfiniteMediaStore.selectedHashes.length} </strong> Items
		</div>

		<div class="insides"></div>
	{:else}
		<div class="iconContainer" data-tauri-drag-region>
			<button onclick={handleSidebarButton} title="Toggle Sidebar">
				<div class="iconContainer">
					<span class="icon-[material-symbols--side-navigation] w-5 h-5"></span>
				</div>
			</button>
			<div class="iconPadding"></div>

			<!--
         <Moon height={20} width={20}></Moon>
			<div class="iconPadding"></div>-->

			<div class="dbButtons" data-tauri-drag-region>
				<button
					class="option accent"
					onclick={async () => {
						dbName = await onNewDb();
					}}
				>
					New DB
				</button>

				<button
					class="option accent"
					onclick={async () => {
						dbName = await onOpenDb();
					}}>Open DB</button
				>

				<button class="option accent" onclick={() => (isConnectMenuActive = !isConnectMenuActive)}>
					Connect to DB ▼
				</button>

				{#if isConnectMenuActive}
					<ConnectMenu bind:isActive={isConnectMenuActive}></ConnectMenu>
				{/if}
			</div>
		</div>

		<div class="insidesFiller"></div>
		<div class="title" data-tauri-drag-region>Kasa</div>
		<div class="iconPadding" data-tauri-drag-region></div>

		<div class="dbInfo" data-tauri-drag-region>
			{dbName}

			<div class="dbSecure">
				{#await isConnectionSecure() then secure}
					{#if !secure}
						<div class="iconContainer warnIconContainer">
							<span class="icon-[material-symbols--warning] w-4 h-4"></span>
						</div>
						(Unsecure)
					{/if}
				{/await}
			</div>
		</div>

		<div class="insidesFiller"></div>

		<button class="layoutSettings" onclick={() => (isLayoutMenuActive = !isLayoutMenuActive)}
			>Layout ▼</button
		>
		{#if isLayoutMenuActive}
			<LayoutMenu></LayoutMenu>
		{/if}

		<div class="iconPadding" data-tauri-drag-region></div>
	{/if}
</div>

<style>
	.dbButtons {
		display: flex;
		gap: 8px;
	}

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
		color: var(--text);
		fill: color;
	}

	svg {
		fill: var(--text);
	}
	.iconPadding {
		width: 10px;
	}
	.option {
		color: var(--text);
		padding-left: 2px;
		padding-right: 2px;
		border-radius: 4px;
		text-overflow: ellipsis;
		white-space: nowrap;
		padding-right: 4px;
		padding-left: 4px;
	}

	.option.accent {
		background-color: var(--accent);
		border: 1px solid var(--accent-hover);
		color: var(--text-opposite);
	}

	.option.accent:hover {
		background-color: var(--accent-hover);
	}

	.option.bordered {
		border: 1px solid var(--primary);
	}

	.option.bordered:hover {
		background-color: color-mix(in srgb, var(--secondary) 20%, var(--background));
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
		padding-left: 4px;
		padding-right: 4px;
		display: flex;
		align-items: center;
		text-align: center;
	}

	.layoutSettings {
		background-color: var(--accent);
		margin-left: 10px;
		margin-right: 10px;
		padding-left: 4px;
		padding-right: 4px;
		border-radius: 4px;
		border: 1px solid var(--accent-hover);
	}

	.layoutSettings:hover {
		background-color: var(--accent-hover);
	}

	.dbSecure {
		margin-left: 2px;
		color: var(--text-opposite);
		text-align: center;
		display: flex;
		align-items: center;
	}

	.warnIconContainer {
		color: #9c302c;
	}
</style>
