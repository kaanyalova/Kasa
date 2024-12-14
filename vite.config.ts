import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';
import { vite as vidstack } from 'vidstack/plugins';

export default defineConfig({
	plugins: [sveltekit(), vidstack()],
	server: {
		watch: {
			ignored: ["**/target/**", "**/kasa_tauri/gen/**"]
		}
	}
});


//vidstack({ include: /player\// });