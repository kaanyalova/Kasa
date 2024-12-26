// @ts-nocheck
// https://github.com/rster2002/svelte-outside-click

import { info } from '@tauri-apps/plugin-log';

// License: MIT
export function clickOutsideModal(node: Node, onEventFunction) {
	const handleClick = (event: Event) => {
		var path = event.composedPath();

		// Needed so it doesn't exits from video players settings menu
		if (event.target.tagName !== 'DIALOG') {
			return;
		}
		console.log(event.target.tagName);

		if (!path.includes(node)) {
			onEventFunction();
		}
	};

	document.addEventListener('click', handleClick);

	return {
		destroy() {
			document.removeEventListener('click', handleClick);
		}
	};
}

export function clickOutside(node, onEventFunction) {
	const handleClick = (event) => {
		var path = event.composedPath();

		if (!path.includes(node)) {
			onEventFunction();
		}
	};

	document.addEventListener('click', handleClick);

	return {
		destroy() {
			document.removeEventListener('click', handleClick);
		}
	};
}
