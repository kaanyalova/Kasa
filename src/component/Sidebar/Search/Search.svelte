<!-- svelte-ignore non_reactive_update -->
<script lang="ts">
	import SearchDropDown from './SearchDropDown.svelte';
	import { onMount, type SvelteComponent } from 'svelte';
	import { event } from '@tauri-apps/api';
	import type { Optional } from '$lib/Option';
	import '../../../fonts.css';
	import MagnifyingGlass from '../../Vector/MagnifyingGlass.svelte';
	import { info } from '@tauri-apps/plugin-log';
	import '../SideBarGlobals.scss';
	import { invoke } from '@tauri-apps/api/core';
	import type { SearchTag } from './SearchDropDown';
	import { stat } from '@tauri-apps/plugin-fs';
	import { commands } from '$lib/tauri_bindings';

	let entriesToShow: Array<SearchTag> = $state([]);
	let searchContents = $state('');
	let shouldShow = $state(false);
	let shouldActuallyShow = $derived(
		shouldShow && entriesToShow.length > 0 && searchContents.length > 0
	);
	// TODO derive from (dropdownLen > 0 || shouldShow) so it doesn't look "selected" when there is no entries
	//  -1 means user hasn't selected any entry with keyboard yet
	let cooldown = $state(0);
	let keyboardSelectedIndex = $state(-1);

	$effect(() => {
		// Reset the keyboard index to -1 when sthe search entry array changes
		// This is what major search engines does
		entriesToShow;
		keyboardSelectedIndex = -1;
	});

	function handleSearch() {
		if (searchContents.length > 0) {
			clearTimeout(cooldown);
			cooldown = setTimeout(getDropdownEntriesFromDB, 200);
		} else {
			entriesToShow = [];
		}
	}

	async function getDropdownEntriesFromDB() {
		const entries: Array<SearchTag> = await invoke('query_tags', {
			tagName: searchContents,
			count: 10
		}).then((e) => {
			return e as Array<SearchTag>;
		});

		if (entries.length > 0) {
			shouldShow = true;
		}

		entriesToShow = entries;
	}

	function onSearchButtonClicked() {
		commands.search(searchContents, 0, 0);
	}
</script>

<div class="parentDiv">
	<button class="searchIcon" class:selected={shouldActuallyShow} onclick={onSearchButtonClicked}>
		<div class="searchIconInner">
			<MagnifyingGlass height={15} width={15}></MagnifyingGlass>
		</div>
	</button>

	<input
		type="search"
		id="search"
		class="searchBar"
		placeholder="Search"
		class:selected={shouldActuallyShow}
		bind:value={searchContents}
		oninput={() => handleSearch()}
		onfocus={() => {
			shouldShow = true;
		}}
		onkeydown={(event) => {
			const dropDownLenght = entriesToShow.length;
			// Keyboard handling for dropdown
			switch (event.key) {
				case 'ArrowUp':
					if (keyboardSelectedIndex >= 0) {
						keyboardSelectedIndex -= 1;

						console.log(`index minus one, current index ${keyboardSelectedIndex}`);
					}
					break;

				case 'ArrowDown':
					if (keyboardSelectedIndex === -1) {
						keyboardSelectedIndex = 0;
						console.log(`initial current index ${keyboardSelectedIndex}`);
					} else if (keyboardSelectedIndex === dropDownLenght - 1) {
						return;
					} else if (keyboardSelectedIndex < dropDownLenght - 1) {
						keyboardSelectedIndex += 1;
						console.log(`index plus one, current index ${keyboardSelectedIndex}`);
					}
					break;

				case 'Enter':
					if (keyboardSelectedIndex >= 0) {
						console.log(`selected with keyboard ,index: ${keyboardSelectedIndex}`);
						shouldShow = false;
					}
					break;
			}
		}}
		onfocusout={() => (shouldShow = false)}
		autocomplete="off"
	/>

	{#if shouldActuallyShow}
		<SearchDropDown {entriesToShow} {keyboardSelectedIndex} {searchContents}></SearchDropDown>
	{/if}
</div>

<style>
	.searchBar {
		padding: 5px;
		border-radius: 0px 10px 10px 0px;
		background-color: var(--secondary-alt);
		color: var(--text);
		font-size: large;
		width: var(--searchbar-width);
		height: 35px;
		min-width: none;
		border: solid color-mix(in srgb, var(--secondary) 30%, transparent) 1px;
		border-left: none;
		box-shadow: rgba(100, 100, 111, 0.2) 0px 7px 29px 0px;
	}
	.searchBar::placeholder {
		color: var(--text);
	}
	.parentDiv {
		margin: 15px;
		justify-content: center;
		display: flex;
		flex-grow: 100;
	}

	.searchBar.selected {
		border-radius: 0px 10px 0px 0px;
	}

	input:focus {
		outline: none;
	}

	.searchIcon {
		display: flex;
		justify-content: center;
		align-items: center;
		background-color: var(--accent);
		border-radius: 10px 0px 0px 10px;
		border: solid color-mix(in srgb, var(--secondary) 30%, transparent) 1px;
		box-shadow: rgba(100, 100, 111, 0.2) 0px 7px 29px 0px;
		height: 35px;
		/* width: 50px*/
	}

	.searchIconInner {
		display: flex;
		margin: 5px;
		justify-content: center;
	}

	.searchIcon.selected {
		border-radius: 10px 0px 0px 0px;
	}
</style>
