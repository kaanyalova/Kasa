<script lang="ts">
	import { onMount, untrack } from 'svelte';
	import SearchBarTestEntries from './SearchBarTestEntries';
	import { stat } from '@tauri-apps/plugin-fs';
	import type { SearchDropDownProps } from './SearchDropDown';
	import { info } from '@tauri-apps/plugin-log';
	import { handleSelect } from './HandleSelect';
	import SearchDropDownEntry from './SearchDropDownEntry.svelte';
	// TODO adding tags

	// keyboardSelectedIndex is undefined if it is not selected
	let { entriesToShow, keyboardSelectedIndex, searchContents }: SearchDropDownProps = $props();

	let entriesList: HTMLUListElement;

	onMount(async () => {});
</script>

<!--
onclick doesn't work, see
https://stackoverflow.com/questions/17769005/onclick-and-onblur-ordering-issue-->

<!-- We might want meta entries here for sorting and special operations-->
<div class="dropDown">
	<ul bind:this={entriesList}>
		{#each entriesToShow as entry, index}
			<li id={index.toString()}>
				<button
					onmousedown={() => {
						console.log(`selected with mouse, index:  ${index}`);
						searchContents = handleSelect(entry.name, searchContents);
					}}
					class:selected={index === keyboardSelectedIndex}
				>
					<SearchDropDownEntry
						name={entry.name}
						color={entry.tag_details.color}
						count={entry.count}
					/>
				</button>
			</li>
		{/each}
	</ul>
</div>

<style lang="scss">
	.entry {
		display: flex;
		flex-direction: row;
		flex-grow: 100;
		align-items: flex-end;
	}

	.count {
		align-self: flex-end;
		margin-left: auto;
	}

	.dropDown {
		position: absolute;
		top: 34px;
		background-color: var(--background);
		border-radius: 0px 0px 5px 5px;
		border: solid 1px;
		border-color: #31353f;
		width: calc(var(--searchbar-width) + 26px); /* -32on webkit, -21 on firefox wtf*/
		box-shadow: rgba(100, 100, 111, 0.2) 0px 7px 29px 0px;
		font-family: 'Ubuntu';
		border: 1px var(--border-color) solid;
		z-index: 2;
		font-size: smaller;
	}
	li {
		border-color: white;
		color: var(--text);
		display: flex;
	}

	/*
	li:hover {
		background-color: var(--secondary-alt);
		border-radius: 5px;
	}
	*/
	button {
		flex-grow: 100;
		display: flex;
		justify-content: start;
		text-align: left;
		padding: 0px;
		margin: 2px;
	}
	button.selected {
		background-color: #6b819b;
		border-radius: 5px;
	}
</style>
