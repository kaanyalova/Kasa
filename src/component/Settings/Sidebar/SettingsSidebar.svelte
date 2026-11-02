<script lang="ts">
	import CategoriesSidebar from '../../Shared/CategoriesSidebar.svelte';
	import CategoriesSidebarEntry from '../../Shared/CategoriesSidebarEntry.svelte';
	import { SettingsCategories } from '../Settings';

	let { selectedCategory, updateSelectedCategory, children }: SettingsSidebarProps = $props();
</script>

<div class="sidebarParent">
	<CategoriesSidebar>
		<li class="sidebarEntry">
			<CategoriesSidebarEntry
				{selectedCategory}
				category={SettingsCategories.Database}
				name="Database"
				onClick={() => updateSelectedCategory(SettingsCategories.Database)}
			>
				<span class="icon-[material-symbols--database] w-5 h-5"></span>
			</CategoriesSidebarEntry>
		</li>

		<li class="sidebarEntry">
			<CategoriesSidebarEntry
				{selectedCategory}
				category={SettingsCategories.Thumbnails}
				name="Thumbnails"
				onClick={() => updateSelectedCategory(SettingsCategories.Thumbnails)}
			>
				<span class="icon-[material-symbols--image] w-5 h-5"></span>
			</CategoriesSidebarEntry>
		</li>

		<li class="sidebarEntry">
			<CategoriesSidebarEntry
				{selectedCategory}
				category={SettingsCategories.Downloaders}
				name="Gallery DL"
				onClick={() => updateSelectedCategory(SettingsCategories.Downloaders)}
			>
				<span class="icon-[material-symbols--download] w-5 h-5"></span>
			</CategoriesSidebarEntry>
		</li>

		<li class="sidebarEntry">
			<CategoriesSidebarEntry
				{selectedCategory}
				category={SettingsCategories.UnreferencedItems}
				name="Unreferenced"
				onClick={() => updateSelectedCategory(SettingsCategories.UnreferencedItems)}
			>
				<span class="icon-[material-symbols--delete] w-5 h-5"></span>
			</CategoriesSidebarEntry>
		</li>

		<li class="sidebarEntry">
			<CategoriesSidebarEntry
				{selectedCategory}
				category={SettingsCategories.Indexers}
				name="Indexers"
				onClick={() => updateSelectedCategory(SettingsCategories.Indexers)}
			>
				<span class="icon-[material-symbols--database-search] w-5 h-5"></span>
			</CategoriesSidebarEntry>
		</li>
	</CategoriesSidebar>

	{@render children?.()}
</div>

<style>
	li {
		border-bottom: solid 1px var(--secondary-alt);
		color: var(--text);
		display: flex;
		flex-direction: row;
	}

	.sidebarEntry {
		display: flex;
		background-color: var(--background);
	}

	.sidebarParent {
		display: flex;
		flex-direction: row;
		z-index: 3;
	}
</style>
