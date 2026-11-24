import { defineConfig } from '@playwright/test'

const port = 4173

// Two touch viewports, the sizes of the target monitors.
const touch = { hasTouch: true, deviceScaleFactor: 1 }

export default defineConfig({
  testDir: 'e2e',
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: 0,
  reporter: [['list'], ['html', { open: 'never' }]],
  outputDir: 'test-results',
  // Baselines are made in the CI container only (fonts differ per OS).
  snapshotPathTemplate: '{testDir}/__screenshots__/{projectName}/{arg}{ext}',
  expect: { toHaveScreenshot: { threshold: 0.01, maxDiffPixels: 0 } },
  use: { baseURL: `http://127.0.0.1:${port}`, trace: 'retain-on-failure' },
  projects: [
    { name: 'wide-1920x480', use: { ...touch, viewport: { width: 1920, height: 480 } } },
    { name: 'small-1024x600', use: { ...touch, viewport: { width: 1024, height: 600 } } },
  ],
  webServer: {
    command: `npm run build:e2e && npx vite preview --outDir dist-e2e --host 127.0.0.1 --port ${port} --strictPort`,
    url: `http://127.0.0.1:${port}`,
    reuseExistingServer: !process.env.CI,
    timeout: 120_000,
  },
})
