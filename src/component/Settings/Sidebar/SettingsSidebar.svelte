<script lang="ts">
	import { SettingsCategories } from '../Settings';
	import SidebarEntry from './SidebarEntry.svelte';

	let { selectedCategory, updateSelectedCategory }: SettingsSidebarProps = $props();
</script>

<div class="sidebarParent">
	<div class="settingsSidebar">
		<ul class="sidebarEntryList">
			<li class="sidebarEntry">
				<SidebarEntry
					{selectedCategory}
					category={SettingsCategories.Database}
					name="Database"
					onClick={() => updateSelectedCategory(SettingsCategories.Database)}
				>
					<span class="icon-[material-symbols--database] w-5 h-5"></span>
				</SidebarEntry>
			</li>

			<li class="sidebarEntry">
				<SidebarEntry
					{selectedCategory}
					category={SettingsCategories.Thumbnails}
					onClick={() => {
						updateSelectedCategory(SettingsCategories.Thumbnails);
						console.log('update');
					}}
					name="Thumbnails"
				>
					<span class="icon-[material-symbols--image] w-5 h-5"></span>
				</SidebarEntry>
			</li>

			<li class="sidebarEntry">
				<SidebarEntry
					{selectedCategory}
					category={SettingsCategories.Downloaders}
					name="Gallery DL"
					onClick={() => updateSelectedCategory(SettingsCategories.Downloaders)}
				>
					<span class="icon-[material-symbols--download] w-5 h-5"></span>
				</SidebarEntry>
			</li>

			<li class="sidebarEntry">
				<SidebarEntry
					{selectedCategory}
					category={SettingsCategories.UnreferencedItems}
					name="Unreferenced"
					onClick={() => updateSelectedCategory(SettingsCategories.UnreferencedItems)}
				>
					<span class="icon-[material-symbols--delete] w-5 h-5"></span>
				</SidebarEntry>
			</li>
			<li class="sidebarEntry">
				<SidebarEntry
					{selectedCategory}
					category={SettingsCategories.Indexers}
					name="Indexers"
					onClick={() => updateSelectedCategory(SettingsCategories.Indexers)}
				>
					<span class="icon-[material-symbols--database-search] w-5 h-5"></span>
				</SidebarEntry>
			</li>
			<li class="sidebarEntry">Forth</li>
		</ul>
	</div>
	<slot></slot>
</div>

<style>
	li {
		border-bottom: solid 1px var(--secondary-alt);
		color: var(--text);
		display: flex;
		flex-direction: row;
	}

	.settingsSidebar {
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
