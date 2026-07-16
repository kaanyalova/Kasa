<script lang="ts">
	import { tick } from 'svelte';

	let { onCheck, state }: CheckboxProps = $props();

	function onClick() {
		state = !state;
		onCheck(state);
	}
</script>

<div>
	<button class="checkBox" onclick={() => onClick()} oncontextmenu={(e) => e.preventDefault()}>
		{#if state === true}
			<span class="icon-[material-symbols--check] w-4 h-4"></span>
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
