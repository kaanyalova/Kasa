<script>
	import { onMount } from 'svelte';
	import '../app.css';
	import '../colors.css';
	import { invoke } from '@tauri-apps/api/core';

	// Reloading *will* break the MediaServer on rust side, this should disable it on release builds
	// https://github.com/tauri-apps/wry/issues/30
	//disableMenu();
</script>
<!--
Needed for dragging to be possible if the app is run on fullscreen, At least on gnome it seems like
it is not possible to drag the window if the top bar doesn't exist (if the app is on a non-primary monitor) 
-->
<div class="topDragRegion" data-tauri-drag-region></div>

<div class="page-border">
	<slot />
</div>

<style>
	/* 
	Placing box shadows around is possible with putting an transparent margin around
	and adding that to --window-border-size but the window resize regions are outside of the actual 
	window borders it is possible to manually trigger resizes with startResizeDragging()
	and as far as I know there is no easy way of adding event listeners to borders and adding an element
	for each window border sounds like a terrible solution. 

	https://v2.tauri.app/reference/javascript/api/namespacewebviewwindow/#startresizedragging
	
	Adding rounded corners is also possible here but it requires manually listening for "full-screen" resizes
	and removing all the rounded corners when it happens. Resizing also does not work as intended just like the 
	shadows.
	*/
	.page-border {
		position: fixed;
		top: 0;
		left: 0;
		right: 0;
		bottom: 0;
		border: var(--window-border-size) solid var(--window-border-color);
		box-sizing: border-box;
	}

	.topDragRegion {
		position: absolute;
		top: 0;
		left: 0;
		height:var(--window-border-size);
		width: 100vw;
		background-color: transparent;
		z-index: 9999;

	}

	:global(:root) {
		--window-border-color: var(--border);
		--window-border-size: 2px;
		--page-content-height: calc(100vh - var(--window-border-size) * 2);
		--page-content-width: calc(100vw - var(--window-border-size) * 2);
	}
</style>
