<script lang="ts">
	import type { SearchOptionButtonProps } from './SearchOptionButtonProps';

	let { title, children, isSelected, onClick }: SearchOptionButtonProps = $props();
</script>

<button {title} class:selected={isSelected} onclick={() => onClick()}>
	{@render children()}
</button>

<style>
	button {
		border: 1px solid var(--secondary-alt);
		color: var(--text);
		padding: 4px;
		display: flex;
		align-items: center;
		justify-content: center;
		margin-left: 4px;
		margin-right: 4px;
	}

	button:hover {
		background-color: var(--secondary-alt);
	}

	button.selected {
		background-color: var(--secondary-alt);
		border: 1px solid var(--accent);
	}
</style>
