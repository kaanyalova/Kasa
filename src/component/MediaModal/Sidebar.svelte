<script lang="ts">
	import { info } from '@tauri-apps/plugin-log';
	import X from '../Vector/X.svelte';
	import { MediaModalStatusStore } from './MediaModalStatusStore.svelte';
	import { onMount } from 'svelte';
	import TagDisplay from './Tags/TagDisplay.svelte';
	import SidebarFooter from './SidebarFooter.svelte';
	import { TauriEvent } from '@tauri-apps/api/event';
	import { writeText } from '@tauri-apps/plugin-clipboard-manager';
	import Clipboard from '../Vector/Clipboard.svelte';
	import VerticalDivider from '../Shared/Dividers/VerticalDivider.svelte';
	import Tick from '../Vector/Tick.svelte';
	import HorizontalDivider from '../Shared/Dividers/HorizontalDivider.svelte';
	import { DividerSizes } from '../Shared/Dividers/DividerSizes';

	let { data, updateTagsTextBoxContents }: SidebarProps = $props();

	let showCopySuccessButton = $state(-1);

	function onClickPath(path: string, i: number) {
		showCopySuccessButton = i;
		writeText(path);
		setTimeout(() => {
			showCopySuccessButton = -1;
		}, 1000);
	}
</script>

<div class="sidebar">
	<div class="windowControls">
		<button onclick={MediaModalStatusStore.close} class="">
			<div class="closeButtonWrapper">
				<X height={15} width={15}></X>
			</div>
		</button>
	</div>

	<div class="sideBarScrollableContents">
		<div class="header">Known Paths</div>
		<ul class=" details multicolorRows monoFont">
			{#each data.paths as path, i}
				<li>
					<button
						class="pathCopyButton"
						aria-label="Copy the Path"
						onclick={() => onClickPath(path, i)}
					>
						<div class="pathCopyButtonInsides">
							<div class="oneLine pathContainer expandOnHover">
								{path}
							</div>
						</div>
						<div
							class="copyIconContainer"
							class:copyIconContainer2N={(i + 1) % 2 === 0}
							onclick={() => onClickPath(path, i)}
							aria-hidden="true"
						>
							{#if showCopySuccessButton === i}
								<Tick height={15} width={15}></Tick>
							{:else}
								<Clipboard height={15} width={15}></Clipboard>
							{/if}
						</div>
					</button>
				</li>
			{/each}
		</ul>

		<div class="separator"></div>

		<div class="header">Meta</div>
		<ul class="details multicolorRows">
			{#each data.meta as meta_entry}
				<li class="metaEntry paddedLi">
					<span class="oneLine">
						<span class="metaEntryTitle">{meta_entry.name}:</span>
						<span
							class:monoFont={meta_entry.isValueMonospaced}
							class:oneLine={meta_entry.isOneLine}
						>
							{meta_entry.value}</span
						>
					</span>
				</li>
			{/each}
		</ul>

		<div class="separator"></div>

		<div class="header">Import Info</div>
		<ul class="details">
			<li class="paddedLi">Imported From <span class="redditMoveThisLater">Placeholder</span></li>
			<li class="paddedLi">
				Link : <a href="https://old.reddit.com/" class="">
					https://www.youtube.com/watch?v=dQw4w9WgXcQ</a
				>

				<!-- 	
		This should open in users browser not in tauri There should also be the reddit's alien thing
		next to the "badge", The Link should not be the full, just cut off after first line Not done
		-->
			</li>
		</ul>

		<div class="separator"></div>

		<TagDisplay
			isInEditMode={MediaModalStatusStore.tagsEditModeActive}
			initialEditBoxContents={data.rawTagsField}
			{updateTagsTextBoxContents}
			{data}
		></TagDisplay>
	</div>

	<SidebarFooter {data}></SidebarFooter>
</div>

<style>
	.sidebar {
		background: var(--background);
		width: 300px;
		color: var(--text);
		font-family: 'Ubuntu';
		border: var(--secondary-alt) 1px solid;
		border-left: 0px;
		border-radius: 0px 5px 5px 0px;
	}

	.sideBarScrollableContents {
		overflow-y: auto;
		height: calc(
			80vh - 25px - 50px - 2px
		); /* Container height - Window Controls height - Sidebar Footer height - Borders */
	}

	.windowControls {
		height: 25px;
		background-color: var(--accent);
		flex-grow: 1;
		display: flex;
		flex-direction: row-reverse;
		color: black;
		padding-right: 5px;
		border-radius: 0px 5px 0px 0px;
	}

	.closeButtonWrapper {
		height: 100%;
	}

	.header {
		padding-left: 4px;
		padding-right: 4px;
	}

	.separator {
		height: 10px;
	}
	.tags {
		background-color: var(--background);
		border: var(--secondary-alt) 1px solid;
		border-radius: 2px;
		font-size: small;
		resize: none;
		margin: 4px;
	}
	.tags:focus {
		outline: var(--accent) 1px solid;
	}

	.metaEntry {
		display: flex;
		align-items: center;
	}
	.metaEntryTitle {
		font-weight: bold;
		padding-right: 4px;
	}

	.oneLine {
		text-overflow: ellipsis;
		overflow: hidden;
		white-space: nowrap;
	}

	/*Somewhat broken in webkit, does not always update when changing stuff on the dev server*/
	.expandOnHover:hover {
		white-space: normal;
		text-overflow: unset;
		word-break: break-all;
		text-align: left;
	}

	.pathContainer {
		color: var(--text);
		padding: 4px;
	}
	.pathCopyButton {
		display: flex;
		flex-grow: 1;
		fill: var(--text);
	}
	.pathCopyButtonInsides {
		display: flex;
		align-items: center;
		/* Does not work */
		flex-grow: 1;
		width: calc(300px - 15px - 16px - 8px); /* Sidebar width - Icon width - Padding */
	}

	.copyIconContainer {
		background-color: var(--secondary-alt);
		align-self: stretch;
		overflow: hidden;
		display: flex;
		align-items: center;
		border-radius: 0px 2px 2px 0px;
		padding-left: 4px;
		padding-right: 4px;
	}

	.copyIconContainer2N {
		background-color: var(--background);
	}

	.details {
		font-size: small;
		border: var(--secondary-alt) 1px solid;
		padding: 2px;
		border-radius: 2px;
		margin: 4px;
	}
	.multicolorRows > li:nth-child(2n) {
		background-color: var(--secondary-alt);
	}

	.monoFont {
		font-family: 'UbuntuMono';
	}

	.paddedLi {
		padding: 2px;
	}
	.redditMoveThisLater {
		background-color: #e64f17;
		padding: 2px;
		color: var(--text);
		border-radius: 999px;
	}

	a {
		text-decoration: underline;
		font-size: small;
	}
</style>
