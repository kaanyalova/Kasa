<script lang="ts">
	import { onDestroy } from 'svelte';
	import BorderedBox from '../../../Shared/BorderedBox.svelte';
	import ConfirmationDialog from '../../Shared/ConfirmationDialog.svelte';
	import { ConfirmationScreenStore } from '../../Shared/ConfirmationDialogStore.svelte';
	import IndexerButton from './IndexerButton.svelte';
	import IndexerButtonDestructive from './IndexerButtonDestructive.svelte';
	import { commands } from '$lib/tauri_bindings';
	import { openFilePickerWithMultipleFolderSelection } from '$lib/openFilePicker';
	import { clickOutside, clickOutsideClassExcluding, clickOutsideModal } from '$lib/clickOutside';

	let entriesPromise = $state();
	let selectedEntries: Array<number> = $state([]);
	let entries: Array<string> = $state([]);
	let isCTRLBeingHeld = $state(false);

	function clickOutsideRightPanel(node: Node, callback: () => void) {
		return clickOutsideClassExcluding(node, callback, 'rightPanel');
	}

	onDestroy(() => {
		ConfirmationScreenStore.close();
	});

	async function onAddIndex() {
		const paths = await openFilePickerWithMultipleFolderSelection();

		paths.forEach((path) => {
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
		selectedEntries.forEach(async (entryIdx) => {
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
			<div use:clickOutsideRightPanel={() => (selectedEntries = [])}>
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
					<span class="icon-[material-symbols--screen-search-desktop] w-5 h-5"></span>
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
					<span class="icon-[material-symbols--hard-drive] w-5 h-5"></span>
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

					<span class="icon-[material-symbols--image-search] w-5 h-5"></span>
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
					<span class="icon-[material-symbols--delete] w-5 h-5"></span>
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

					<span class="icon-[material-symbols--hard-drive] w-5 h-5"></span>
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

					<span class="icon-[material-symbols--hard-drive] w-5 h-5"></span>
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
					<span class="icon-[material-symbols--hard-drive] w-5 h-5"></span>
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

					<span class="icon-[material-symbols--favorite] w-5 h-5"></span>
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
		width: calc(
			100vw - 435px
		); /* TODO The list moves slightly when it goes form empty -> non-empty */
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
