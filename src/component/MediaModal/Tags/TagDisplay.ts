import type { MediaInfo } from '$lib/tauri_bindings';

export type TagDisplayProps = {
	isInEditMode: boolean;
	initialEditBoxContents: string;
	updateTagsTextBoxContents: Function;
	data: MediaInfo;
};

export type CursorPosition = {
	top: number | null;
	left: number | null;
};
