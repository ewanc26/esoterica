// Vite config for the Esoterica web UI (Svelte 5 + WASM)
import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';

export default defineConfig({
  plugins: [svelte()],
  server: { port: 5173 },
  build: { target: 'esnext' },
});
