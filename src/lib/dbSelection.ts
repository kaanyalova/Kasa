import { error } from '@tauri-apps/plugin-log';
import { openFilePickerWithSaveDialog, openFilePickerWithSelectDialog } from './openFilePicker';
import { commands, events } from './tauri_bindings';

export async function onNewDb(): Promise<string> {
	const paths = await openFilePickerWithSaveDialog('Kasa Database', '*.kasa', 'default.kasa');
	const path = paths[0];

	console.log('Selected path:', path);

	if (!path) {
		error('File picker failed to select file');
		return '';
	}

	await commands.setDbPath(path);
	const dbName = path.split('/').pop() || '';

	events.dbsUpdatedEvent.emit({ new_db: true });
	return dbName;
}

export async function onOpenDb(): Promise<string> {
	const paths = await openFilePickerWithSelectDialog('Kasa Database', '*.kasa');
	const path = paths[0];

	console.log('Selected path:', path);

	if (!path) {
		error('File picker failed to select file');
		return '';
	}

	const dbName = path.split('/').pop() || '';
	await commands.setDbPath(path);

	events.dbsUpdatedEvent.emit({ new_db: false });
	return dbName;
}

export async function onConnectToDb(url: string): Promise<string> {
	await commands.setDbPath(url);
	await events.dbsUpdatedEvent.emit({ new_db: false });
	return url;
}
