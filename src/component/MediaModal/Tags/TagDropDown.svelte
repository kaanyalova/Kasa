<script lang="ts">
	import type { TagDropDownProps } from './TagDropDown';

	let { top, left, selectedIndex, tags, onTagClick }: TagDropDownProps = $props();
</script>

<div class="tagDropDown" style="top:{top}px; left:{left}px">
	<ul>
		{#each tags as tag, i}
			<li class="tagEntry" class:keyBoadSelected={i === selectedIndex}>
				<button onmousedown={() => onTagClick(i)}> {tag.name}</button>
			</li>
		{/each}
	</ul>
</div>

<style>
	.tagDropDown {
		position: fixed;
		background-color: var(--background);
		font-size: small;
		border: var(--secondary-alt) 1px solid;
		padding: 4px;
	}

	.tagEntry:hover {
		background-color: var(--secondary-alt);
	}

	.tagEntry.keyBoadSelected {
		background-color: #6b819b;
	}

	button {
		display: flex;
		flex-grow: 1;
	}
	li {
		display: flex;
	}
</style>
