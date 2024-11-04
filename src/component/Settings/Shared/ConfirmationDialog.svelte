<script lang="ts">
	import { onMount } from 'svelte';
	import { ConfirmationScreenStore } from './ConfirmationDialogStore.svelte';
	import Title from './Title.svelte';
</script>

{#if ConfirmationScreenStore.isOpen}
	<dialog class="confirmationDialog">
		<div class="insides">
			<div class="title">{ConfirmationScreenStore.title}</div>

			<div class="text">
				{#if ConfirmationScreenStore.passUnsafeHTML}
					{@html ConfirmationScreenStore.text}
					Unsafe
				{:else}
					{ConfirmationScreenStore.text}
					Safe
				{/if}
			</div>

			<div class="buttons">
				<button class="button confirmButton" onclick={() => ConfirmationScreenStore.onConfirm()}>
					{ConfirmationScreenStore.confirmText}
				</button>
				<button class="button exitButton" onclick={() => ConfirmationScreenStore.close()}
					>{ConfirmationScreenStore.exitText}</button
				>
			</div>
		</div>
	</dialog>
{/if}

<style>
	.confirmationDialog {
		width: calc(100%);
		height: 100%;
		background: rgba(
			0,
			0,
			0,
			0.5
		); /*slightly lower than the MediaModal as the background is darker*/
		z-index: 5;
		position: fixed;
		left: 0;
		top: 0;
		bottom: 0;
		display: flex;
		justify-content: center;
		align-items: center;
	}

	.insides {
		min-width: 40%;
		min-height: 25%;
		background: var(--background);
		border: 1px solid var(--secondary-alt);
		z-index: 100;
		display: flex;
		flex-direction: column;
	}

	.title {
		color: var(--text);
		display: flex;
		align-items: center;
		justify-content: center;
		font-size: large;
		font-weight: bold;
		padding: 4px;
	}

	.text {
		color: var(--text);
		text-align: center;
		padding: 8px;
	}

	.buttons {
		display: flex;
		flex-grow: 1;
		align-items: center;
	}

	.button {
		padding: 4px;
		border-radius: 4px;
		flex-grow: 1;
		margin-left: 12px;
		margin-right: 12px;
		width: 50%;
		height: 8vh;
		font-size: large;
		font-weight: bold;
	}

	.confirmButton {
		background: #ac4245;
		color: white;
	}

	.exitButton {
		background: var(--accent);
	}
</style>
