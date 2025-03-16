<script>
	import Search from './Search/Search.svelte';
	import '../../colors.css';
	import Categories from './Categories.svelte';
	import Bottom from './Bottom.svelte';
	import Operations from './Operations.svelte';
	import Collections from './Collections.svelte';
	import './SideBarGlobals.scss';
	import { sidebarStore } from './SidebarStore.svelte';
	import TagPicker from './TagPicker/TagPicker.svelte';
</script>

<div class="sidebarParent">
	<!---->
	{#if sidebarStore.isActive}
		<div class="sidebar" id="sidebar">
			<ul>
				<li>
					<div>
						<div class="searchContainer">
							<Search></Search>
						</div>
						<div class="invisDraggable" data-tauri-drag-region></div>
					</div>
				</li>

				<!--
						
					<li>
						<Categories></Categories>
					</li>
					
					<li>
						<Collections></Collections>
					</li>
					-->
				<li>
					<TagPicker></TagPicker>
				</li>

				<li>
					<Operations></Operations>
				</li>
			</ul>
			<div class="filler"></div>
			<ul>
				<li>
					<Bottom></Bottom>
				</li>
			</ul>
		</div>
	{/if}

	<!---->

	<div class=""><slot></slot></div>
</div>

<style>
	.sidebar {
		height: calc(100vh - var(--window-border-size) * 2);
		width: var(--sidebar-width);
		background-color: var(--background);
		display: flex;
		flex-direction: column;
		float: left;
		user-select: none;
		border-right: 1px solid var(--border);
	}

	.content {
		position: absolute;
		left: var(--sidebar-width);
	}

	ul {
		flex-direction: column;
		justify-content: flex-start;
		overflow: hidden;
	}

	li {
		display: flex;
		justify-content: center;
	}

	.filler {
		flex-grow: 1;
	}

	.sidebarParent {
		display: flex;
		flex-direction: row;
	}

	.csd {
		height: 32px;
		background-color: color-mix(in srgb, var(--secondary) 30%, transparent);
		flex-shrink: 0;
	}

	.invisDraggable {
		position: absolute;
		background-color: var(--background);
		height: 32px;
		width: calc(var(--sidebar-width) - 1px); /* Sidebar width - resizeBar Width*/
		top: 0px;
		left: 1px;
		z-index: 2;
	}

	.searchContainer {
		z-index: 3;
		position: relative;
		border-right: 1px solid var(--border-on-secondary-alt);
	}
</style>
