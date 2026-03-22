import { error } from '@sveltejs/kit';
import type { PageLoad } from './$types';
import { commands } from '$lib/tauri_bindings';

export const prerender = false;

export const load: PageLoad = async ({ params }) => {
	const sources = await commands.getMediaSources(params.hash);

	return {
		sources: sources
	};

	if (sources.length === 0) {
		error(404, 'Not found');
	}
};
