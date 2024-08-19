<script>
	import { disableMenu } from '$lib/disableRightClickMenu';
	import { onMount } from 'svelte';
	import '../app.css';
	import '../colors.css';
	import { invoke } from '@tauri-apps/api/core';

	// Reloading *will* break the MediaServer on rust side, this should disable it on release builds
	// https://github.com/tauri-apps/wry/issues/30
	//disableMenu();
</script>

<slot />
