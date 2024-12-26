import { commands } from './tauri_bindings';
import { platform } from '@tauri-apps/plugin-os';
import type { TauriEvent } from '@tauri-apps/api/event';
import { comma } from 'postcss/lib/list';
import { open } from '@tauri-apps/plugin-dialog';
import { error } from '@tauri-apps/plugin-log';
export async function openFilePickerWithMultipleFolderSelection(): Promise<Array<string>> {
	const os = platform();
	if (os === 'linux') {
		const files = await commands.newLinuxFilePickerDialog();
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
