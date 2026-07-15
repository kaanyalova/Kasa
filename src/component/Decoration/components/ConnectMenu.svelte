<script lang="ts">
	import { onMount } from 'svelte';
	import '../../../fonts.css';
	import { onConnectToDb } from '$lib/dbSelection';

	let url = $state('');

	let inputElement: HTMLInputElement;

	onMount(() => {
		inputElement.focus();
	});

	let { isActive = $bindable() }: ConnectMenuProps = $props();

	async function onClickConnect() {
		console.log('onClickConnect', url);
		await onConnectToDb(url);
		isActive = false;
	}
</script>

<dialog class="connectMenu" open>
	<div class="dialogContainer">
		<div class="remoteConnectText">Connect to a remote server</div>
		<div class="inputContainer">
			<div class="boxThing">
				<span class="icon-[material-symbols--link-2] w-5 h-"></span>
			</div>
			<!-- svelte-ignore a11y_autofocus  why is this a thing?, not that it works-->
			<input
				type="text"
				autofocus
				placeholder="Server Url"
				class="serverInput"
				bind:this={inputElement}
				bind:value={url}
			/>
			<button
				class="connectButton"
				onclick={async () => {
					await onClickConnect();
				}}>Connect</button
			>
		</div>
	</div>
</dialog>

<style>
	.connectMenu {
		width: 400px;
		height: 100px;
		background-color: var(--background);
		color: var(--text);
		border: 1px solid var(--secondary-alt);
		position: fixed;
		left: calc(420px);
		top: calc(32px + 8px);
		border-radius: 8px;
		padding: 8px;
		display: flex;
		justify-content: center;
		align-items: center;
	}

	.boxThing {
		height: 30px;
		width: 30px;
		background-color: var(--accent);
		border: 1px solid var(--accent-border);
		color: var(--text-opposite);
		display: flex;
		justify-content: center;
		align-items: center;
	}

	.serverInput {
		outline: none;
		width: 250px;
		height: 30px;
		border: 1px solid var(--secondary-alt);
		border-left: none !important;
		padding-left: 4px;
	}

	.serverInput:focus {
		border: solid 1px var(--accent);
	}

	.connectButton {
		background-color: var(--accent);
		color: var(--text-opposite);
		padding-left: 4px;
		padding-right: 4px;
		margin-left: 8px;
		border-radius: 4px;
		border: 1px solid var(--accent-border);
	}

	.connectButton:hover {
		background-color: var(--accent-hover);
	}

	.inputContainer {
		display: flex;
	}

	.dialogContainer {
		display: flex;
		flex-direction: column;
	}

	.remoteConnectText {
		font-size: smaller;
		position: absolute;
		top: 12px;
	}
</style>
