import type { SearchType } from '$lib/tauri_bindings';

function createSearchStore() {
	let searchContents = $state('');
	let searchType: SearchType = $state('TagSearch');

	return {
		get searchContents() {
			return searchContents;
		},
		set searchContents(value: string) {
			searchContents = value;
		},
		get searchType() {
			return searchType;
		},
		set searchType(type: SearchType) {
			searchType = type;
		}
	};
}

export const SearchStore = createSearchStore();
