import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import wasm from 'vite-plugin-wasm';
import { copyFileSync } from 'fs';
import { resolve } from 'path';

export default defineConfig({
  plugins: [
    react(),
    wasm(),
    {
      name: 'copy-index-page',
      closeBundle() {
        copyFileSync(
          resolve(__dirname, 'index.html'),
          resolve(__dirname, 'dist/index.html')
        );
      }
    }
  ],
  build: {
    target: 'esnext',
    rollupOptions: {
      input: {
        app: resolve(__dirname, 'app.html')
      }
    },
    commonjsOptions: {
      include: [/recharts/, /node_modules/],
    },
  },
  optimizeDeps: {
    exclude: ['continuum-golf-simulator'],
    include: ['recharts'],
  },
});
