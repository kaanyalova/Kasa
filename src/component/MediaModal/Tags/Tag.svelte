<script lang="ts">
	import Cross from '../../Vector/Cross.svelte';
	import CrossFilled from '../../Vector/CrossFilled.svelte';
	import Trash from '../../Vector/Trash.svelte';
	import TrashGnome from '../../Vector/TrashGnome.svelte';
	import TrashWithQuestionMark from '../../Vector/TrashWithQuestionMark.svelte';

	let { name, onDelete }: TagProps = $props();

	let clickCount = $state(0);
	let clickColdown: number = $state(0);

	let _delete = $state(false);

	function onClick() {
		clearTimeout(clickColdown);
		if (clickCount === 1) {
			onDelete(name);
			clickCount = 0;
		} else {
			clickCount += 1;
		}

		clickColdown = setTimeout(() => {
			clickCount = 0;
		}, 2000);
	}
</script>

{#if !_delete}
	<button
		class="tag"
		onclick={() => {
			onClick();
		}}
	>
		<div class="name">{name}</div>

		<div class="tagButton">
			{#if clickCount === 1}
				<!--
				On second click, show the delete icon
				-->
				<div class="tagButton trashButton">
					<TrashGnome height={16} width={16}></TrashGnome>
				</div>
			{:else}
				<div class="tagButton xButton">
					<Cross height={16} width={16}></Cross>
				</div>
			{/if}
		</div>
	</button>
{/if}

<style>
	.tag {
		padding: 4px;
		padding-right: 6px;
		padding-left: 6px;
		border: 1px solid var(--border);
		display: inline-flex;
		flex-direction: row;
		margin: 4px;
		transform: 2s;
		vertical-align: middle;
	}

	.tag:hover {
		background-color: var(--secondary-alt);
	}

	.tagButton {
		fill: var(--text);
		align-items: center;
		justify-content: center;
		display: flex;
		height: 24px;
		padding-left: 4px;
	}

	.xButton {
		position: relative;
		top: 1px; /* Text alignment???? */
	}

	.trashButton {
		display: flex;
	}
</style>
