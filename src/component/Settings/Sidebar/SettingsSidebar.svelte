<script lang="ts">
	import Heart from '../../Vector/Heart.svelte';
	import Image from '../../Vector/Image.svelte';
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
					category={SettingsCategories.Thumbnails}
					onClick={() => {
						updateSelectedCategory(SettingsCategories.Thumbnails);
						console.log('update');
					}}
					name="Thumbnails"
				>
					<Image height={20} width={20}></Image>
				</SidebarEntry>
			</li>
			<li class="sidebarEntry">
				<SidebarEntry
					{selectedCategory}
					category={SettingsCategories.Database}
					name="Database"
					onClick={() => {
						updateSelectedCategory(SettingsCategories.Database);
						console.log('update2');
					}}
				>
					<Heart height={20} width={20}></Heart>
				</SidebarEntry>
			</li>
			<li class="sidebarEntry">Third</li>
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
		height: 100vh;
		display: flex;
	}

	.sidebarEntryList {
		background-color: var(--background);
		flex-grow: 100;
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
	}
</style>
