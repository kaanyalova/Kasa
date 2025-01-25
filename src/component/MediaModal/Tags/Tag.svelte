<script lang="ts">
	import CrossFilled from '../../Vector/CrossFilled.svelte';
	import Trash from '../../Vector/Trash.svelte';
	import TrashWithQuestionMark from '../../Vector/TrashWithQuestionMark.svelte';

	let { name, onDelete }: TagProps = $props();

	let isHovered = $state(false);
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
		onmouseenter={() => (isHovered = true)}
		onmouseleave={() => (isHovered = false)}
		class:tagHover={isHovered}
		onclick={() => {
			onClick();
		}}
	>
		<div class="name">{name}</div>
		<div class="xButton" class:xButtonHover={isHovered}>
			{#if isHovered}
				{#if clickCount === 0}
					<CrossFilled height={16} width={16}></CrossFilled>
				{:else if clickCount === 1}
					<Trash height={16} width={16}></Trash>
				{/if}
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
	}

	.xButtonHover {
		fill: var(--text);
		align-items: center;
		justify-content: center;
		display: flex;
		height: 24px;
		padding-left: 6px;
	}

	.tagHover {
		background-color: var(--secondary-alt);
	}
</style>
