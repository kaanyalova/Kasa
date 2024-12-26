<script lang="ts">
	import { onDestroy } from 'svelte';
	import BorderedBox from '../../../Shared/BorderedBox.svelte';
	import { DividerSizes } from '../../../Shared/Dividers/DividerSizes';
	import VerticalDivider from '../../../Shared/Dividers/VerticalDivider.svelte';
	import LoupePlus from '../../../Vector/LoupePlus.svelte';
	import SearchHardrive from '../../../Vector/SearchHardrive.svelte';
	import Button from '../../Shared/Button.svelte';
	import ConfirmationDialog from '../../Shared/ConfirmationDialog.svelte';
	import { ConfirmationScreenStore } from '../../Shared/ConfirmationDialogStore.svelte';
	import Database from '../Database.svelte';
	import IndexerButton from './IndexerButton.svelte';
	import IndexerButtonDestructive from './IndexerButtonDestructive.svelte';
	import SearchFolder from '../../../Vector/SearchFolder.svelte';
	import Trash from '../../../Vector/Trash.svelte';
	import UserTrash from '../../../Vector/UserTrash.svelte';
	import { open } from '@tauri-apps/plugin-dialog';
	import { commands } from '$lib/tauri_bindings';
	import { error, info, trace } from '@tauri-apps/plugin-log';
	import { openFilePickerWithMultipleFolderSelection } from '$lib/openFilePicker';
	import { clickOutside, clickOutsideModal } from '$lib/clickOutside';
	import Heart from '../../../Vector/Heart.svelte';

	let entriesPromise = $state();
	let selectedEntries: Array<number> = $state([]);
	let entries: Array<string> = $state([]);
	let isCTRLBeingHeld = $state(false);

	onDestroy(() => {
		ConfirmationScreenStore.close();
	});

	async function onAddIndex() {
		const paths = await openFilePickerWithMultipleFolderSelection();

		paths.forEach((path) => {
			console.log(`indexing path: ${path}`);
			commands.addIndexSource(path);
			commands.indexPath(path);
		});

		entries = await commands.getIndexPaths();
	}

	async function onRescanAll() {
		await commands.indexAll();
	}

	async function onRescanSelected() {
		selectedEntries.forEach(async (entryIdx) => {
			await commands.indexPath(entries[entryIdx]);
		});
	}

	async function onRemoveSelected() {
		selectedEntries.forEach(async (entryIdx) => {
			await commands.removeIndexSource(entries[entryIdx]);
		});

		entries = await commands.getIndexPaths();
	}

	async function onNukeSelected() {
		console.log(selectedEntries);
		selectedEntries.forEach(async (entryIdx) => {
			console.log(`${entries[entryIdx]}`);
			await commands.nukeSelectedIndex(entries[entryIdx]);
		});

		entries = await commands.getIndexPaths();
	}

	async function onRemoveAll() {
		entries.forEach(async (entry) => {
			await commands.removeIndexSource(entry);
		});

		entries = await commands.getIndexPaths();
	}

	async function onNukeAll() {
		await commands.nukeAllIndexes();
		entries = await commands.getIndexPaths();
	}

	async function updateSelectedIndexes() {
		entries = await commands.getIndexPaths();
	}

	function onEntryClicked(event: MouseEvent, index: number) {
		if (event.ctrlKey) {
			if (!selectedEntries.includes(index)) {
				selectedEntries.push(index);
			}
		} else {
			selectedEntries = [index];
		}
	}
</script>

<ConfirmationDialog></ConfirmationDialog>
<div class="indexers">
	<div class="leftPanel">
		<BorderedBox padding={4}>
			<div use:clickOutsideModal={() => (selectedEntries = [])}>
				{#await updateSelectedIndexes() then}
					<ul>
						{#each entries as entry, i}
							<li class="entry" class:selectedEntry={selectedEntries.includes(i)}>
								<button onclick={(e) => onEntryClicked(e, i)}>
									<div class="entryText">
										{entry}
									</div>
								</button>
							</li>
						{/each}
					</ul>
				{/await}
			</div>
		</BorderedBox>
	</div>

	<div class="rightPanel">
		<ul>
			<li class="flex">
				<!--
				Does it make sense to make this directly open up the system file picker? Rest of the path pickers support both
				directly entering the path and the file picker.

				But the other ones write to the config file instead of the DB so it should be fine?
				-->
				<IndexerButton onClick={async () => await onAddIndex()}>
					{#snippet text()}
						<div>Add index</div>
					{/snippet}
					<!--Replace with search-folder gnome icon-->
					<LoupePlus width={20} height={20}></LoupePlus>
				</IndexerButton>
			</li>

			<li class="flex">
				<IndexerButton
					onClick={async () => {
						await onRescanAll();
					}}
				>
					{#snippet text()}
						Re-scan all
					{/snippet}
					<SearchHardrive width={20} height={20}></SearchHardrive>
				</IndexerButton>
			</li>

			<li class="flex">
				<IndexerButton
					onClick={async () => {
						await onRescanSelected();
					}}
				>
					{#snippet text()}
						Re-scan selected
					{/snippet}

					<SearchFolder width={20} height={20}></SearchFolder>
				</IndexerButton>
			</li>

			<!--
			Does not have any confirmations, as user will need to select the indexes in the first place
			and re-indexing is not that hard
			-->
			<li class="flex">
				<IndexerButton
					onClick={async () => {
						await onRemoveSelected();
					}}
				>
					{#snippet text()}
						Remove selected
					{/snippet}
					<UserTrash width={20} height={20}></UserTrash>
				</IndexerButton>
			</li>

			<!--
			Not red, though not as destructive as ones bellow it as the user can still re-index their files, there is still a confirmation though
			-->
			<li class="flex">
				<IndexerButton
					onClick={() => {
						ConfirmationScreenStore.newDialog(
							'Are you sure?',
							'This will delete <strong>all the indexers</strong> but the metadata will stay',
							async () => {
								await onRemoveAll();
							},
							undefined,
							undefined,
							true
						);
					}}
				>
					{#snippet text()}
						Remove all indexers
					{/snippet}

					<SearchHardrive width={20} height={20}></SearchHardrive>
				</IndexerButton>
			</li>

			<li class="flex">
				<!--Make this one red, and add a confirmation box on top-->
				<IndexerButtonDestructive
					onClick={() => {
						ConfirmationScreenStore.newDialog(
							'Are you sure?',
							'This will both delete <strong>the references to files</strong> and <strong>all the stored metadata</strong>',
							async () => {
								await onNukeSelected();
							},
							undefined,
							undefined,
							true
						);
					}}
				>
					{#snippet text()}
						<div class="">
							<ul>
								<li>Remove selected</li>
								<li><strong> along with all data </strong></li>
							</ul>
						</div>
					{/snippet}

					<SearchHardrive width={20} height={20}></SearchHardrive>
				</IndexerButtonDestructive>
			</li>

			<li class="flex">
				<IndexerButtonDestructive
					onClick={() => {
						ConfirmationScreenStore.newDialog(
							'Are you sure?',
							'This will both delete <strong>all references to files</strong> and <strong>all the stored metadata</strong>',
							async () => {
								onNukeAll();
							},
							undefined,
							undefined,
							true
						);
					}}
				>
					{#snippet text()}
						<div class="">
							<ul>
								<li>Remove <strong>ALL</strong> indexes</li>
								<li><strong>along with all data</strong></li>
							</ul>
						</div>
					{/snippet}
					<SearchHardrive width={20} height={20}></SearchHardrive>
				</IndexerButtonDestructive>
			</li>

			<li class="flex">
				<IndexerButtonDestructive
					onClick={() => {
						commands.cleanupUnreferencedFiles();
					}}
				>
					{#snippet text()}
						<div class="">
							<ul>
								<li>
									Remove <strong>ALL</strong> unindexed
								</li>

								<li>data</li>
							</ul>
						</div>
					{/snippet}

					<Heart width={20} height={20}></Heart>
				</IndexerButtonDestructive>
			</li>
		</ul>
	</div>
</div>

<style>
	.indexers {
		display: flex;
		flex-grow: 1;
	}

	.rightPanel {
		display: flex;
		flex-grow: 0.01;
		flex-direction: column;
		width: 200px;
	}

	.leftPanel {
		display: flex;
		flex-grow: 1;
	}

	.entry {
		flex-shrink: 1;
		background: var(--background);
		padding: 4px;
		min-width: 0px;
		margin-top: 1px;
		margin-left: 1px;
	}

	.selectedEntry {
		outline: 1px solid var(--accent);
		/* https://stackoverflow.com/a/12693151 */
	}

	.entryText {
		width: calc(100vw - 435px);
		text-overflow: ellipsis;
		overflow: hidden;
		white-space: nowrap;
		text-align: start;
	}

	.entry:nth-child(2n) {
		background: var(--secondary-alt);
	}
	ul {
		display: flex;
		flex-direction: column;
	}
</style>
