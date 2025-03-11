import { getCurrentWindow, type Window } from '@tauri-apps/api/window';
import { writable, type Writable, get } from 'svelte/store';

export let appWindow: Writable<Window | undefined> = writable(undefined);

export const initializeAppWindow = async () => {
	import('@tauri-apps/api/window').then((mod) => {
		appWindow.set(mod.getCurrentWindow());
	});
};

export const minimizeWindow = async () => {
	await getCurrentWindow().minimize();
};

export const maximizeWindow = async () => {
	await getCurrentWindow().toggleMaximize();
};

export const closeWindow = async () => {
	await getCurrentWindow().close();
};

export const fullscreenWindow = async () => {
	const fullscreen = await get(appWindow)?.isFullscreen();

	if (fullscreen) {
		await get(appWindow)?.setFullscreen(false);
	} else {
		await get(appWindow)?.setFullscreen(true);
	}
};
