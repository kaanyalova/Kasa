<script lang="ts">
	import { clickOutsideClass } from '$lib/clickOutside';
	import { onDestroy } from 'svelte';
	import NewTagPicker from './NewTagPicker.svelte';

	let open = $state(false);
	let outsideListenerDestory: any;

	function onClickOutside(node: Node, onEventFunction: any) {
		outsideListenerDestory = clickOutsideClass(
			node,
			onEventFunction,
			'newTagPickerElement'
		).destroy;
	}

	onDestroy(() => {
		outsideListenerDestory();
	});
</script>

<button
	class="newTag newTagPickerElement"
	use:onClickOutside={() => {
		open = false;
		console.log('Clicked outside NewTagButton');
	}}
	onclick={() => (open = !open)}>+</button
>

{#if open}
	<NewTagPicker />
{/if}

<style>
	.newTag {
		border: 1px solid var(--border);
		display: inline-flex;
		height: 25px;
		margin: 4px;
		padding-left: 4px;
		padding-right: 4px;
		background-color: var(--accent);
		color: black;
		font-size: 24px;
		font-weight: bold;
		align-items: center;
	}

	.newTag:hover {
		background-color: color-mix(in srgb, var(--accent) 80%, black 20%);
		cursor: pointer;
	}
</style>
