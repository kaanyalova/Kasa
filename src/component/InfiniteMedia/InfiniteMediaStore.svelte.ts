import { commands } from '$lib/tauri_bindings';

export class InfiniteMediaStoreInner {
	constructor() {}

	selectedHashes: Array<string> = $state([]);
	onSelectMode = $derived(this.selectedHashes.length > 0);

	showNames: undefined | boolean = $state(undefined);
	thumbnailScale: undefined | number = $state(undefined);
	isLoaded: boolean = $state(false);

	addMedia(hash: string) {
		if (this.selectedHashes.includes(hash as never)) {
			this.selectedHashes = this.selectedHashes.filter((h) => h !== hash);
		} else {
			this.selectedHashes = [...this.selectedHashes, hash];
		}
	}

	cleanAllMedia() {
		this.selectedHashes = [];
	}

	setShowNames(state: boolean) {
		this.showNames = state;
	}

	async loadSettings() {
		const config = await commands.getConfig();
		this.showNames = config.Layout.show_filenames;
		this.thumbnailScale = config.Layout.thumbnail_scale;
		this.isLoaded = true;
	}
}

export const InfiniteMediaStore = new InfiniteMediaStoreInner();
