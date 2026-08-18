<script lang="ts">
	import { onNewDb, onOpenDb } from '$lib/dbSelection';

	let { problemText }: DbFileMissingProps = $props();
</script>

<div class="dbFileMissingWrapper">
	<div class="dbFileMissingContainer">
		<div class="dbFileMissingText">
			{problemText}
		</div>

		<div class="dbMissingButtonRow">
			<button
				class="dbMissingButton"
				onclick={async () => {
					await onNewDb();
				}}>New DB</button
			>
			<button
				class="dbMissingButton"
				onclick={async () => {
					await onOpenDb();
				}}>Open DB</button
			>
		</div>
	</div>
</div>

<style>
	.dbFileMissingText {
		color: var(--text);

		font-size: 20px;
	}

	.filePath {
		font-family: 'UbuntuMono';
		border: 1px solid var(--secondary-alt);
		padding: 2px;
		margin-left: 4px;
		margin-right: 4px;
	}

	.dbFileMissingWrapper {
		display: flex;
		flex-grow: 1;
		align-items: center;
		justify-content: center;
		height: calc(100% - 32px);
		flex-direction: column;
	}

	.dbMissingButton {
		background-color: var(--accent);
		border: 1px solid var(--accent-border);
		color: var(--text-opposite);
		padding: 8px;
		margin-top: 16px;
		margin-left: 4px;
		margin-right: 4px;
		font-weight: bold;
		border-radius: 4px;
	}
	.dbMissingButton:hover {
		background-color: var(--accent-hover);
	}

	.dbFileMissingContainer {
		display: flex;
		align-items: center;
		justify-self: center;
		flex-direction: column;
		border: 1px solid var(--secondary-alt);
		padding: 32px;
	}
</style>
