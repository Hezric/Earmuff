import { defineConfig } from '@playwright/test';
export default defineConfig({
  testDir: './tests/browser',
  outputDir: 'playwright-results',
  workers: 1,
  use: { baseURL: 'http://127.0.0.1:1438', headless: true },
  webServer: {
    command: 'vp dev --host 127.0.0.1 --port 1438 --strictPort',
    url: 'http://127.0.0.1:1438',
    reuseExistingServer: false,
  },
});
