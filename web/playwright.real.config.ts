import { defineConfig } from '@playwright/test';

// Parcours sur la PILE RÉELLE (serveur Rust, PostgreSQL, front servi par `vite preview` qui relaie /api). La pile est montée et
// arrêtée par `ops/dev/real-stack.ps1` ; ce fichier ne démarre rien. Largeurs : 390 px et 1440 px.
const channel = process.env['PW_CHANNEL'] ?? 'chrome';

export default defineConfig({
  testDir: './e2e-real',
  retries: 0,
  workers: 1,
  timeout: 120_000,
  reporter: [['list']],
  use: { baseURL: process.env['E2E_BASE_URL'] ?? 'http://127.0.0.1:4173', channel },
  projects: [
    { name: 'telephone-390', use: { viewport: { width: 390, height: 844 } } },
    { name: 'ordinateur-1440', use: { viewport: { width: 1440, height: 900 } } },
  ],
});
