// Tauri doesn't have a Node.js server to do proper SSR
// so we use adapter-static with a fallback to index.html to put the site in SPA mode
// See: https://svelte.dev/docs/kit/single-page-apps
// See: https://v2.tauri.app/start/frontend/sveltekit/ for more info
import path from 'path'
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte'
import adapter from "svelte-adapter-bun"; 

/** @type {import('@sveltejs/kit').Config} */
const config = {
    preprocess: vitePreprocess(),

    kit: {
        alias: {
            $lib: path.resolve('./src/lib'),
        },

        adapter: adapter({
            fallback: 'index.html',
        }),

        paths: {
            relative: true,
        },
    },
}

export default config
