import { info } from '@tauri-apps/plugin-log';

export function handleSelect(selection: string, searchContents: string): string {
	const queries = searchContents.split(',');
	const lastTag = queries[queries.length - 1];

	const isPrefixedWithTagExcludeCharacter = lastTag.trim().startsWith('-');

	if (isPrefixedWithTagExcludeCharacter) {
		queries[queries.length - 1] = '-' + selection;
	} else {
		queries[queries.length - 1] = selection;
	}
	return queries.join(', ');
}
