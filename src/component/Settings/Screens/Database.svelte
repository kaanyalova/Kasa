<script lang="ts">
	import { commands } from '$lib/tauri_bindings';
	import Button from '../Shared/Button.svelte';
	import ConfirmationDialog from '../Shared/ConfirmationDialog.svelte';
	import { ConfirmationScreenStore } from '../Shared/ConfirmationDialogStore.svelte';
</script>

<ConfirmationDialog></ConfirmationDialog>
<div class="">
	<button
		onclick={() => {
			ConfirmationScreenStore.newDialog(
				'Are you sure',
				'This will potentially completely break your db',
				() => {
					commands.nukeDbVersioning();
				}
			);
		}}
	>
		Nuke DB Versioning (Only press if you know what you are doing)
	</button>
</div>

<style>
	button {
		background-color: var(--destructive);
		padding: 6px;
		border-radius: 8px;
	}
</style>
