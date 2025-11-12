import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import wasm from 'vite-plugin-wasm';
import { copyFileSync } from 'fs';
import { resolve } from 'path';
import path from 'path';
import tsconfigPaths from 'vite-tsconfig-paths';

export default defineConfig({
  plugins: [
    react(),
    wasm(),
    tsconfigPaths(),
  ],
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
    },
  },
  build: {
    target: 'esnext',
    rollupOptions: {
      input: {
        main: resolve(__dirname, 'index.html'),
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
