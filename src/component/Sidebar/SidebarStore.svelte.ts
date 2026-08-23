import { info } from '@tauri-apps/plugin-log';
import { onMount } from 'svelte';
import { writable } from 'svelte/store';

// TODO hide sidebar if size is zero?

function updateSidebarSize() {
	const root: any = document.querySelector(':root');
	root.style.setProperty('--main-val', sidebarStore.size + 'px');
}

function createSidebarStore() {
	let isActive = $state(true);
	let sidebarSize = $state(100);
	let savedSidebarSize = $state(100);

	let listeners: Set<() => void> = new Set();

	function notifyResizeSubscribers() {
		listeners.forEach((l) => l());
	}

	return {
		subscribeForResizes: (callback: () => void): (() => void) => {
			listeners.add(callback);
			return () => {
				listeners.delete(callback);
			};
		},

		toggle: () => {
			// close
			if (isActive) {
				savedSidebarSize = sidebarSize;
				sidebarSize = 0;
				updateSidebarSize();
				isActive = false;
				notifyResizeSubscribers();
			}

			// open
			else if (!isActive) {
				sidebarSize = savedSidebarSize;
				updateSidebarSize();
				isActive = true;
				notifyResizeSubscribers();
			}
		},

		setSize: (size: number) => {
			sidebarSize = size;
			updateSidebarSize();
			notifyResizeSubscribers();
		},
		get isActive() {
			return isActive;
		},
		get size() {
			return sidebarSize;
		}
	};
}

export const sidebarStore = createSidebarStore();
