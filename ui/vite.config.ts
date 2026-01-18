import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';

export default defineConfig({
  plugins: [svelte()],
  server: {
    proxy: {
      '/api': 'http://127.0.0.1:3001',
      '/events': {
        target: 'http://127.0.0.1:3001',
        ws: false,
      },
    },
  },
});
