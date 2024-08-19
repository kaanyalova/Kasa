<script lang="ts">
	import { onMount, untrack } from 'svelte';
	import SearchBarTestEntries from './SearchBarTestEntries';
	import { stat } from '@tauri-apps/plugin-fs';
	import type { SearchDropDownProps, SearchTag } from './SearchDropDown';
	import { info } from '@tauri-apps/plugin-log';
	// TODO adding tags

	// keyboardSelectedIndex is undefined if it is not selected
	let { entriesToShow, keyboardSelectedIndex, searchContents }: SearchDropDownProps = $props();

	let entriesList: HTMLUListElement;

	onMount(async () => {});
</script>

<!--
onclick doesn't work, see
https://stackoverflow.com/questions/17769005/onclick-and-onblur-ordering-issue-->
<div class="dropDown">
	<ul bind:this={entriesList}>
		{#each entriesToShow as entry, index}
			<li id={index.toString()}>
				<button
					onmousedown={() => {
						console.log(`selected with mouse, index:  ${index}`);
					}}
					class:selected={index === keyboardSelectedIndex}
				>
					{entry.name}
				</button>
			</li>
		{/each}
	</ul>
</div>

<style lang="scss">
	.dropDown {
		position: absolute;
		top: 34px;
		background-color: var(--background);
		border-radius: 0px 0px 10px 10px;
		border: solid 1px;
		border-color: #31353f;
		width: calc(var(--searchbar-width) + 26px); /* -32on webkit, -21 on firefox wtf*/
		box-shadow: rgba(100, 100, 111, 0.2) 0px 7px 29px 0px;
		font-family: 'Ubuntu';
		border: 1px var(--border-color) solid;
		z-index: 2;
	}
	li {
		border-color: white;
		padding: 2px;
		color: var(--text);
		display: flex;
		margin: 4px;
	}

	li:hover {
		background-color: var(--secondary-alt);
		border-radius: 10px;
	}
	button {
		flex-grow: 100;
		display: flex;
		justify-content: start;
		text-align: left;
		padding: 2px;
		padding-left: 4px;
		padding-right: 4px;
	}
	button.selected {
		background-color: #6b819b;
		border-radius: 10px;
	}
</style>
