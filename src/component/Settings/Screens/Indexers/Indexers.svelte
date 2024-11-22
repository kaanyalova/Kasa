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
	import { error, trace } from '@tauri-apps/plugin-log';
	onDestroy(() => {
		ConfirmationScreenStore.close();
	});

	async function onAddIndex() {
		// Is this always a directory?
		const path = await open({
			multiple: false,
			directory: true
		});

		if (typeof path !== undefined) {
			await commands.addIndexSource(path!!);
			await commands.indexPath(path!!);
		} else {
			error('Invalid directory');
		}
	}

	async function onRescanAll() {
		await commands.indexAll();
	}

	async function rescanSelected() {
		// TODO selection
	}
</script>

<ConfirmationDialog></ConfirmationDialog>
<div class="indexers">
	<div class="leftPanel">
		<BorderedBox padding={4}>
			<ul>
				<li class="entry">Entry</li>
				<li class="entry">Entry</li>
				<li class="entry">Entry</li>
				<li class="entry">Entry</li>
				<li class="entry">Entry</li>
				<li class="entry">Entry</li>
			</ul>
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
				<IndexerButton onClick={() => {}}>
					{#snippet text()}
						<div>Add index</div>
					{/snippet}
					<!--Replace with search-folder gnome icon-->
					<LoupePlus width={20} height={20}></LoupePlus>
				</IndexerButton>
			</li>

			<li class="flex"></li>
			<li class="flex">
				<IndexerButton onClick={() => {}}>
					{#snippet text()}
						Re-scan all
					{/snippet}
					<SearchHardrive width={20} height={20}></SearchHardrive>
				</IndexerButton>
			</li>

			<li class="flex">
				<IndexerButton onClick={() => {}}>
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
				<IndexerButton onClick={() => {}}>
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
							() => {
								console.log('Delete all indexes');
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
							() => {
								console.log('Exiting');
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
							() => {
								console.log('Nuke data');
							}
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

			<li></li>
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
	}

	.leftPanel {
		display: flex;
		flex-grow: 1;
	}

	.entry {
		display: flex;
		flex-grow: 1;
		background: var(--background);
		padding: 4px;
	}

	.entry:nth-child(2n) {
		background: var(--secondary-alt);
	}
</style>
