import { commands } from '$lib/tauri_bindings';
import { stat } from '@tauri-apps/plugin-fs';

export class InfiniteMediaStoreInner {
	constructor() {
	}

	selectedHashes: Array<string> = $state([]);
	onSelectMode = $derived(this.selectedHashes.length > 0);
	showNames = $state(true);
	thumbnailScale = $state(3.0);

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
	}

}

export const InfiniteMediaStore = new InfiniteMediaStoreInner();
