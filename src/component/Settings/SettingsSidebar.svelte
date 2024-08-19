<script lang="ts">
	import Heart from '../Vector/Heart.svelte';
	import Image from '../Vector/Image.svelte';

	let { selectedCategory, updateSelectedCategory }: SettingsProps = $props();
</script>

<div class="sidebarParent">
	<div class="settingsSidebar">
		<ul class="sidebarEntryList">
			<li class="sidebarEntry">
				<button>
					<div class="buttonInner">
						<Image width={20} height={20}></Image>
						<div class="buttonInnerText">Thumbnails</div>
					</div>
				</button>
			</li>
			<li class="sidebarEntry">Second</li>
			<li class="sidebarEntry">Third</li>
			<li class="sidebarEntry">Forth</li>
		</ul>
	</div>
	<slot></slot>
</div>

<style>
	button {
		display: flex;
		flex-grow: 100;
		justify-content: start;
		flex-direction: row;
	}
	li:hover {
		background-color: var(--secondary-alt);
	}
	li {
		border-bottom: solid 1px var(--secondary-alt);
		color: var(--text);
		padding: 4px;
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
