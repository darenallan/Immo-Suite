/// <reference types="vitest/config" />
import react from '@vitejs/plugin-react';
import { defineConfig } from 'vite';
import pkg from './package.json' with { type: 'json' };

// La version affichée est celle du binaire : APP_VERSION (commit) fournie par la CI, sinon celle du paquet.
const version = process.env['APP_VERSION'] ?? pkg.version;
const environment = process.env['APP_ENV'] ?? 'development';

export default defineConfig({
  plugins: [react()],
  define: {
    __APP_VERSION__: JSON.stringify(version),
    __APP_ENV__: JSON.stringify(environment),
  },
  build: {
    target: 'es2022',
    sourcemap: false,
    cssCodeSplit: false,
  },
  test: {
    environment: 'jsdom',
    globals: true,
    setupFiles: ['./src/test-setup.ts'],
    include: ['src/**/*.test.{ts,tsx}'],
  },
});
