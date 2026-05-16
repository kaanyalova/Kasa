import { error } from "@tauri-apps/plugin-log";
import { openFilePickerWithSaveDialog, openFilePickerWithSelectDialog } from "./openFilePicker";
import { commands } from "./tauri_bindings";
import { emit } from "@tauri-apps/api/event";

export async function onNewDb(): Promise<string> {
		const paths = await openFilePickerWithSaveDialog('Kasa Database', '*.kasa', 'default.kasa');
		const path = paths[0];

		console.log('Selected path:', path);

		if (!path) {
			error('File picker failed to select file');
			return "";
		}

		await commands.setDbPath(path);
		const dbName = path.split('/').pop() || '';

		await emit('dbs_updated', { newDb: true });
        return dbName;
    }

	export async function onOpenDb(): Promise<string> {
		const paths = await openFilePickerWithSelectDialog('Kasa Database', '*.kasa');
		const path = paths[0];

		console.log('Selected path:', path);

		if (!path) {
			error('File picker failed to select file');
			return "";
		}

		const dbName = path.split('/').pop() || '';
		await commands.setDbPath(path);

		await emit('dbs_updated');
        return dbName;
    }