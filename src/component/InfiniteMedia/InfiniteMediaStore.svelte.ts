import { commands } from '$lib/tauri_bindings';

export class InfiniteMediaStoreInner {
	constructor() {}

	selectedHashes: Array<string> = $state([]);
	onSelectMode = $derived(this.selectedHashes.length > 0);

	private showNames: undefined | boolean = $state(undefined);
	private thumbnailScale: undefined | number = $state(undefined);
	private isLoaded: boolean = $state(false);
	private layoutChangeListeners: Set<() => void> = new Set();

	private notifyLayoutChangeListeners() {
		this.layoutChangeListeners.forEach((l) => l());
	}

	getIsLoaded(): boolean {
		return this.isLoaded;
	}

	subscribeForLayoutChanges(callback: () => void): () => void {
		this.layoutChangeListeners.add(callback);
		return () => {
			this.layoutChangeListeners.delete(callback);
		};
	}

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
		this.notifyLayoutChangeListeners();
	}

	getShowNames(): boolean | undefined {
		return this.showNames;
	}

	getThumbnailScale(): number | undefined {
		return this.thumbnailScale;
	}

	setThumbnailScale(scale: number) {
		this.thumbnailScale = scale;
		this.notifyLayoutChangeListeners();
	}

	async loadSettings() {
		const config = await commands.getConfig();
		this.showNames = config.Layout.show_filenames;
		this.thumbnailScale = config.Layout.thumbnail_scale;
		this.isLoaded = true;
		this.notifyLayoutChangeListeners();
	}
}

export const InfiniteMediaStore = new InfiniteMediaStoreInner();
