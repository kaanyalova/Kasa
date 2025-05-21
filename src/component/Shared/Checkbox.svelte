<script lang="ts">
	import { tick } from 'svelte';
	import Tick from '../Vector/Tick.svelte';

	let { onCheck, state }: CheckboxProps = $props();

	function onClick() {
		if (state === true) {
			state = false;
			onCheck(state);
		} else {
			state = true;
			onCheck(state);
		}
	}
</script>

<div>
	<button class="checkBox" onclick={() => onClick()} oncontextmenu={(e) => e.preventDefault()}>
		{#if state === true}
			<Tick height={16} width={16}></Tick>
		{/if}
	</button>
</div>

<style>
	.checkBox {
		height: 20px;
		width: 20px;
		display: flex;
		align-items: center;
		justify-content: center;
		fill: var(--text);
		border: 1px solid var(--border-on-secondary-alt);
		background-color: var(--secondary-alt);
	}
</style>
