// See https://kit.svelte.dev/docs/types#app
// for information about these interfaces
declare global {
	namespace App {
		// interface Error {}
		// interface Locals {}
		// interface PageData {}
		// interface PageState {}
		// interface Platform {}
	}

	interface Window { __TAURI__: any; }
	declare function $effect(fn: () => void | (() => void) | Promise<void>): void;
}

export { };
