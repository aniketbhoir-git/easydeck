import { expect, test, type Page } from '@playwright/test'

// Toggle fixture: `?deck=toggle` in src/lib/mock-ipc.ts. `mic` has a look per state;
// `lamp` has only an onAction and no looks.

const tile = (page: Page, id: string) => page.locator(`[data-widget-id="${id}"]`)
const runCalls = (page: Page) =>
  page.evaluate(() =>
    (window as unknown as { __ipcCalls: { cmd: string; args: unknown }[] }).__ipcCalls
      .filter((c) => c.cmd === 'run_action')
      .map((c) => c.args),
  )
const bg = (page: Page, id: string) =>
  tile(page, id).evaluate((el) => getComputedStyle(el).backgroundColor)

test.beforeEach(async ({ page }) => {
  await page.goto('/?deck=toggle')
  await expect(tile(page, 'mic')).toBeVisible()
})

test('shows the off look and makes no call', async ({ page }) => {
  await expect(tile(page, 'mic')).toContainText('Muted')
  await expect(tile(page, 'mic')).toHaveAttribute('aria-pressed', 'false')
  await expect.poll(() => bg(page, 'mic')).toBe('rgb(45, 74, 124)')
  expect(await runCalls(page)).toHaveLength(0)
})

test('tap runs the on switch once and shows the on look; tap again switches off', async ({
  page,
}) => {
  await tile(page, 'mic').click()
  await expect(tile(page, 'mic')).toContainText('Live')
  await expect(tile(page, 'mic')).toHaveAttribute('aria-pressed', 'true')
  await expect.poll(() => bg(page, 'mic')).toBe('rgb(124, 45, 45)')
  expect(await runCalls(page)).toEqual([{ widgetId: 'mic', toggle: 'on' }])
  await tile(page, 'mic').click()
  await expect(tile(page, 'mic')).toContainText('Muted')
  expect(await runCalls(page)).toEqual([
    { widgetId: 'mic', toggle: 'on' },
    { widgetId: 'mic', toggle: 'off' },
  ])
})

test('a toggle without looks keeps the widget label and the default colour', async ({ page }) => {
  await tile(page, 'lamp').click()
  await expect(tile(page, 'lamp')).toContainText('Lamp')
  await expect(tile(page, 'lamp')).toHaveAttribute('aria-pressed', 'true')
  await expect.poll(() => bg(page, 'lamp')).toBe('rgb(35, 37, 43)')
})

test('a reload keeps the state', async ({ page }) => {
  await tile(page, 'mic').click()
  await expect(tile(page, 'mic')).toContainText('Live')
  await page.reload()
  await expect(tile(page, 'mic')).toContainText('Live')
})

test('a rejected run_action reverts the state', async ({ page }) => {
  await page.goto('/?deck=toggle&fail=mic')
  await expect(tile(page, 'mic')).toContainText('Muted')
  await tile(page, 'mic').click()
  await expect.poll(async () => (await runCalls(page)).length).toBe(1)
  await expect(tile(page, 'mic')).toContainText('Muted')
  await expect(tile(page, 'mic')).toHaveAttribute('aria-pressed', 'false')
  await page.reload()
  await expect(tile(page, 'mic')).toContainText('Muted')
})

test('screenshot: toggle off', async ({ page }) => {
  await expect(page).toHaveScreenshot('toggle-off.png', { animations: 'disabled' })
})

test('screenshot: toggle on', async ({ page }) => {
  await tile(page, 'mic').click()
  await expect(tile(page, 'mic')).toContainText('Live')
  await page.mouse.move(0, 0)
  await expect(page).toHaveScreenshot('toggle-on.png', { animations: 'disabled' })
})
