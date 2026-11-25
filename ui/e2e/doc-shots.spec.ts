import { expect, test } from '@playwright/test'

// Saves review screenshots for the owner. Run: DOC_SHOTS=1 npx playwright test doc-shots
test.skip(!process.env.DOC_SHOTS, 'only with DOC_SHOTS=1')

test('save starter deck screenshot', async ({ page }, info) => {
  await page.clock.setFixedTime(new Date('2026-11-25T14:07:00Z'))
  await page.goto('/')
  await expect(page.locator('[data-widget-id]').first()).toBeVisible()
  const size = info.project.name.split('-').pop()
  await page.screenshot({ path: `../docs/screenshots/G-1-clock-${size}.png` })
})

test('save gauge screenshot', async ({ page }, info) => {
  await page.clock.setFixedTime(new Date('2026-11-25T14:07:00Z'))
  await page.goto('/')
  await expect(page.locator('[data-widget-id]').first()).toBeVisible()
  await expect
    .poll(async () => {
      await page.evaluate(() =>
        (window as unknown as { __emitSource: (i: string, p: unknown) => Promise<void> }).__emitSource('stats', {
          'cpu.load': 62,
        }),
      )
      return page.locator('[data-widget-id="cpu"]').getByTestId('gauge-value').innerText()
    })
    .toContain('62')
  const size = info.project.name.split('-').pop()
  await page.screenshot({ path: `../docs/screenshots/R2-2-gauge-${size}.png` })
})
