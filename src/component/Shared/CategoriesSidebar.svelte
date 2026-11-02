<script lang="ts">
	let { children }: CategoriesSidebarProps = $props();
</script>

<div class="categoriesSidebar">
	<ul class="sidebarEntryList">
		{@render children?.()}
	</ul>
</div>

<style>
	.categoriesSidebar {
		background-color: var(--background);
		border-right: 1px solid var(--border);
		width: 200px;
		height: var(--page-content-height);
		display: flex;
		z-index: 3;
	}

	.sidebarEntryList {
		background-color: var(--background);
		flex-grow: 1;
		display: flex;
		flex-direction: column;
		border: var(--secondary-alt) solid 1px;
		margin: 5px;
		overflow-y: auto;
		list-style: none;
		padding: 0;
	}
</style>
