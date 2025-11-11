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
      name: 'copy-landing-page',
      closeBundle() {
        copyFileSync(
          resolve(__dirname, 'landing.html'),
          resolve(__dirname, 'dist/landing.html')
        );
      }
    }
  ],
  build: {
    target: 'esnext',
    commonjsOptions: {
      include: [/recharts/, /node_modules/],
    },
  },
  optimizeDeps: {
    exclude: ['continuum-golf-simulator'],
    include: ['recharts'],
  },
});
