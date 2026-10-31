import { defineConfig } from '@playwright/test';

// Deux largeurs d'écran (TESTING_QA.md) : 390 px (téléphone) et 1440 px (ordinateur). Chrome installé sur le poste.
const channel = process.env['PW_CHANNEL'] ?? 'chrome';

export default defineConfig({
  testDir: './e2e',
  retries: 0,
  reporter: [['list']],
  use: { baseURL: 'http://127.0.0.1:4173', channel },
  projects: [
    { name: 'telephone-390', use: { viewport: { width: 390, height: 844 } } },
    { name: 'ordinateur-1440', use: { viewport: { width: 1440, height: 900 } } },
  ],
  webServer: {
    command: 'npm run build && npx vite preview --host 127.0.0.1 --port 4173 --strictPort',
    url: 'http://127.0.0.1:4173',
    reuseExistingServer: false,
    timeout: 120_000,
  },
});
