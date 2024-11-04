<script lang="ts">
	import VerticalDivider from '../../Shared/Dividers/VerticalDivider.svelte';

	let { onClick, name, category, selectedCategory }: SidebarEntryProps = $props();
</script>

<button onclick={() => onClick()} class:isSelected={category === selectedCategory}>
	<div class="buttonInner">
		<VerticalDivider width={4}></VerticalDivider>

		<slot></slot>
		<div class="buttonInnerText">{name}</div>
	</div>
</button>

<style>
	button {
		display: flex;
		flex-grow: 1;
		justify-content: start;
		flex-direction: row;
		padding: 4px;
	}

	button:hover {
		background-color: var(--secondary-alt);
	}

	.isSelected {
		background-color: var(--secondary-alt);
		font-weight: bold;
	}

	.buttonInner {
		fill: var(--text);
		color: var(--text);
		display: flex;
		flex-direction: row;
		align-items: center;
	}

	.buttonInnerText {
		padding-left: 8px;
		padding-right: 8px;
	}
</style>
