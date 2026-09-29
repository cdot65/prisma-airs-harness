import {defineConfig} from '@playwright/test';

export default defineConfig({
  testDir: './tests',
  use: {
    baseURL: process.env.DOCS_URL || 'http://127.0.0.1:4173/prisma-airs-harness/',
    launchOptions: {executablePath: process.env.CHROMIUM_PATH || undefined},
  },
  webServer: process.env.DOCS_URL ? undefined : {
    command: 'npm run serve',
    url: 'http://127.0.0.1:4173/prisma-airs-harness/',
    reuseExistingServer: !process.env.CI,
  },
});
