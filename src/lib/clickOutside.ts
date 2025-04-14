// https://github.com/rster2002/svelte-outside-click

import { info, trace } from '@tauri-apps/plugin-log';
import type { ActionReturn } from 'svelte/action';

// License: MIT
export function clickOutsideTagName(node: Node, onEventFunction: any, tagName: string) {
	const handleClick = (event: Event) => {
		var path = event.composedPath();

		// Needed so it doesn't exits from video players settings menu
		if ((event.target as HTMLElement).tagName !== tagName) {
			return;
		}
		//trace(`Clicked outside of element with id ${(event.target as HTMLElement).tagName}`)

		//console.log((event.target as HTMLElement).tagName);

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

export function clickOutsideExcludingTagName(node: Node, onEventFunction: any, tagName: string) {
	const handleClick = (event: Event) => {
		var path = event.composedPath();

		// Needed so it doesn't exits from video players settings menu
		if ((event.target as HTMLElement).tagName === tagName) {
			return;
		}
		trace(`Clicked outside of element with id ${(event.target as HTMLElement).tagName}`);

		console.log((event.target as HTMLElement).tagName);

		if (!path.includes(node)) {
			onEventFunction();
		}
	};

	document.addEventListener('click', handleClick);

	return {
		destroy() {
			document.removeEventListener('click', handleClick);
			console.log('Destory called... Somewhere');
		}
	};
}

export function clickOutsideClass(node: Node, onEventFunction: any, _class: string) {
	const handleClick = (event: Event) => {
		var path = event.composedPath();

		// Needed so it doesn't exits from video players settings menu

		//(event.target as HTMLElement).classList.forEach((v) => console.log(`Class: ${v}`))
		if (!(event.target as HTMLElement).classList.contains(_class)) {
			return;
		}
		//trace(`Clicked outside of element with id ${(event.target as HTMLElement).tagName}`)

		//console.log((event.target as HTMLElement).tagName);

		if (!path.includes(node)) {
			onEventFunction();
		}
	};

	document.addEventListener('click', handleClick);

	return {
		destroy() {
			console.log('Destory called... Somewhere');

			document.removeEventListener('click', handleClick);
		}
	};
}

//export function clickOutsideModal(node: Node, onEventFunction: any) {
//	const destory = clickOutsideTagName(node, onEventFunction, 'DIALOG');
//}

export function clickOutside(node: Node, onEventFunction: any) {
	const handleClick = (event: Event) => {
		var path = event.composedPath();

		if (!path.includes(node)) {
			onEventFunction();
		}
	};

	document.addEventListener('click', handleClick);

	return () => {
		trace('destory click outside listener');
		document.removeEventListener('click', handleClick);
	};
}

export function clickOutsideModal(node: Node, callback: () => void): ActionReturn {
	const handleClick = (event: MouseEvent) => {
		const path = event.composedPath();

		if (node && !path.includes(node)) {
			callback();
		}
	};

	document.addEventListener('click', handleClick, true);

	return {
		destroy() {
			trace('Removing click outside listener');
			document.removeEventListener('click', handleClick, true);
		}
	};
}
