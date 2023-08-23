import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';
// import preprocess from 'svelte-preprocess';

export default defineConfig({
	build: {
		outDir: 'build'
	},
	server: {
		port: 5173
	},
	resolve: {
		alias: {
			$lib: "./src/lib",
			types: "./src/rs_bindings.ts",
		},
	},
	plugins: [
		svelte({
			// preprocess: [
			// 	preprocess({}),
			// ],
		})
	]
});