# UI end-to-end tests

Playwright runs the built UI (`npm run build:e2e`, mode `e2e`) with Tauri `mockIPC` and a
fixture deck (`src/lib/mock-ipc.ts`). The mock is imported only in that mode, so the
production bundle never contains it. Two touch viewports: 1920x480 and 1024x600.

- Run: `npm run e2e` (needs `npx playwright install chromium` once).
- Screenshot baselines live in `e2e/__screenshots__/` and are made in the CI container only,
  because fonts differ per OS. On Windows or macOS the screenshot tests will not match; run
  the behavior tests with `npx playwright test --grep-invert screenshot`.
- Update baselines: run the `UI snapshots` workflow (Actions tab), download the `ui-baselines`
  artifact, copy it into `e2e/__screenshots__/`, commit.
- On a failure, the `ui-playwright` artifact holds the expected, actual and diff images.
