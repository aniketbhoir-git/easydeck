import { expect, test } from '@playwright/test'

// Saves review screenshots for the owner. Run: DOC_SHOTS=1 npx playwright test doc-shots
test.skip(!process.env.DOC_SHOTS, 'only with DOC_SHOTS=1')

test('save starter deck screenshot', async ({ page }, info) => {
  await page.goto('/')
  await expect(page.locator('[data-widget-id]').first()).toBeVisible()
  const size = info.project.name.split('-').pop()
  await page.screenshot({ path: `../docs/screenshots/A-7-starter-${size}.png` })
})
