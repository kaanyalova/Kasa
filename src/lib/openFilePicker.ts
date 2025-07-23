import { commands } from './tauri_bindings';
import { platform } from '@tauri-apps/plugin-os';
import type { TauriEvent } from '@tauri-apps/api/event';
import { comma } from 'postcss/lib/list';
import { open, save } from '@tauri-apps/plugin-dialog';
import { error } from '@tauri-apps/plugin-log';

/**
 *  Open a file picker dialog to select multiple files.
 *  Uses the ashpd file picker on linux, falls back to the tauri one on other platforms.
 */
export async function openFilePickerWithMultipleFolderSelection(): Promise<Array<string>> {
	const os = platform();
	if (os === 'linux') {
		const files = await commands.newLinuxFilePickerDialogMultipleFolderSelect();
		return files.map((it) => decodeURI(it));
	} else {
		// The ts definition is just wrong
		const path: string | Array<string> | null = await open({
			multiple: false,
			directory: true
		});

		if (Array.isArray(path)) {
			return path.map((it) => {
				return decodeURI(it);
			});
		}

		if (typeof path == undefined) {
			return [];
		}
		return [decodeURI(path!!)];
	}
}

export async function openFilePickerWithSaveDialog(
	filterName: string,
	filterGlob: string,
	currentName: string
): Promise<Array<string>> {
	const os = platform();

	if (os === 'linux') {
		const files = await commands.newLinuxFilePickerDialogSaveFile(
			filterName,
			filterGlob,
			currentName
		);
		return files.map((it) => decodeURI(it));
	} else {
		const path = await save({
			defaultPath: currentName,
			filters: [
				{
					name: filterName,
					extensions: [filterGlob]
				}
			]
		});

		if (path === undefined || path === null) {
			return [];
		}

		return [decodeURI(path)];
	}
}

/**
 *
 * @param filterName The name to be displayed in the file picker dialog
 * @param filterGlob The file extension filter to be applied
 * @returns
 */
export async function openFilePickerWithSelectDialog(
	filterName: string,
	filterGlob: string
): Promise<Array<string>> {
	const os = platform();

	if (os === 'linux') {
		const files = await commands.newLinuxFilePickerDialogFileSelect(filterName, filterGlob);
		return files.map((it) => decodeURI(it));
	} else {
		const path = await open({
			multiple: false,
			filters: [
				{
					name: filterName,
					extensions: [filterGlob]
				}
			]
		});

		if (path === undefined || path === null) {
			return [];
		}

		if (Array.isArray(path)) {
			return path.map((it) => decodeURI(it));
		}

		return [decodeURI(path)];
	}
}
