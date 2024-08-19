<script lang="ts">
	import { osType } from './utils/os';
	import { cn } from './utils/utils';
	import Gnome from './controls/linux/Gnome.svelte';
	import MacOs from './controls/MacOs.svelte';
	import Windows from './controls/Windows.svelte';
	import { info } from '@tauri-apps/plugin-log';

	export let platform = 'auto';
	export let hide = false;
	export let hideMethod = 'display';
	export let justify = false;

	const customClass = cn(
		'flex',
		$$props.class,
		hide && (hideMethod === 'display' ? 'hidden' : 'invisible')
	);

	// Determine the default platform based on the operating system if platform not specified
	if (platform === 'auto') {
		switch (osType) {
			case 'macos':
				platform = 'macos';
				break;
			case 'linux':
				platform = 'gnome';
				break;
			default:
				platform = 'windows';
		}
	}
</script>

{#if platform === 'windows'}
	<Windows {...$$props} class={cn(customClass, justify && 'ml-auto')} />

	<!-- 
	Macos is disabled here, will draw the titlebar on top of the sidebar instead
-->
{:else if platform === 'macos'}
	<MacOs {...$$props} class={cn(customClass, justify && 'ml-0 ')} />
{:else if platform === 'gnome'}
	<Gnome {...$$props} class={cn(customClass, justify && 'ml-auto')} />
{:else}
	<Windows {...$$props} class={cn(customClass, justify && 'ml-auto')} />
{/if}
