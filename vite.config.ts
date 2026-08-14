import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';
import { vite as vidstack } from 'vidstack/plugins';
import tailwindcss from '@tailwindcss/vite';

export default defineConfig({
	plugins: [sveltekit(), vidstack(), tailwindcss()],
	server: {
		watch: {
			ignored: ["**/target/**", "**/kasa_tauri/gen/**"]
		}
	},
	build: {
		sourcemap:true
	}
});


//vidstack({ include: /player\// });