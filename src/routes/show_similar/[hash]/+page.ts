import { error } from '@sveltejs/kit';
import type { PageLoad } from './$types';
import { commands } from '$lib/tauri_bindings';

export const prerender = false;

export const load: PageLoad = async ({ params }) => {
    const closest = await commands.getTopNClosestForMedia(params.hash, 100);

    return {
        closest: closest
    };

    if (closest.length === 0) {
        error(404, 'Not found');
    }
};
