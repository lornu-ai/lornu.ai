import { defineConfig } from '@playwright/test';

export default defineConfig({
  testDir: './verification',
  use: {
    headless: true,
    baseURL: 'http://localhost:5175',
  },
});
