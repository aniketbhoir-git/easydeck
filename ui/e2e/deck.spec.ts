import { expect, test, type Page } from '@playwright/test'

const calls = (page: Page) => page.evaluate(() => (window as unknown as { __ipcCalls: { cmd: string; args: unknown }[] }).__ipcCalls)

test.beforeEach(async ({ page }) => {
  await page.goto('/')
  await expect(page.locator('[data-widget-id]').first()).toBeVisible()
})

test('the fixture deck (ui/src/lib/mock-ipc.ts) renders its home page', async ({ page }) => {
  await expect(page.locator('[data-widget-id]')).toHaveCount(4)
  for (const label of ['Clock', 'CPU', 'Game', 'Notes']) {
    await expect(page.getByRole('button', { name: label, exact: true })).toBeVisible()
  }
  await expect(page.getByRole('button', { name: 'In game', exact: true })).toBeVisible()
})

test('a tap calls run_action exactly once', async ({ page }) => {
  await page.locator('[data-widget-id="notes"]').tap()
  await expect.poll(async () => (await calls(page)).filter((c) => c.cmd === 'run_action')).toEqual([
    { cmd: 'run_action', args: { widgetId: 'notes' } },
  ])
  // Give a stray second call time to arrive.
  await page.waitForTimeout(300)
  expect((await calls(page)).filter((c) => c.cmd === 'run_action')).toHaveLength(1)
})

test('text cannot be selected', async ({ page }) => {
  const tile = page.locator('[data-widget-id="game-one"]')
  expect(await tile.evaluate((el) => getComputedStyle(el).userSelect)).toBe('none')
  await page.locator('.label', { hasText: 'Game' }).dblclick()
  await page.keyboard.press('Control+A')
  expect(await page.evaluate(() => window.getSelection()?.toString() ?? '')).toBe('')
})

test('a pressed tile gets the press class', async ({ page }) => {
  // dispatchEvent, not a real touch: Playwright cannot hold a tap, so this checks the press wiring.
  const tile = page.locator('[data-widget-id="cpu"]')
  await expect(tile).not.toHaveClass(/pressed/)
  await tile.dispatchEvent('pointerdown')
  await expect(tile).toHaveClass(/pressed/)
  await tile.dispatchEvent('pointerup')
  await expect(tile).not.toHaveClass(/pressed/)
})

test('screenshot: home page', async ({ page }) => {
  await expect(page).toHaveScreenshot('home.png', { animations: 'disabled' })
})

test('screenshot: pressed tile', async ({ page }) => {
  await page.locator('[data-widget-id="cpu"]').dispatchEvent('pointerdown')
  await page.waitForTimeout(200)
  await expect(page).toHaveScreenshot('home-pressed.png', { animations: 'disabled' })
})

test('screenshot: second page', async ({ page }) => {
  await page.getByRole('button', { name: 'In game', exact: true }).tap()
  await expect(page.locator('[data-widget-id="heal"]')).toBeVisible()
  await expect(page).toHaveScreenshot('in-game.png', { animations: 'disabled' })
})

test('a deck without positions renders in reading order', async ({ page }) => {
  // The mock returns what load_deck returns after easydeck-core places the widgets.
  await page.goto('/?deck=no-positions')
  await expect(page.locator('[data-widget-id]')).toHaveCount(5)
  const boxes = await page.locator('[data-widget-id]').evaluateAll((els) =>
    els.map((el) => {
      const r = el.getBoundingClientRect()
      return { id: el.getAttribute('data-widget-id'), x: r.x, y: r.y }
    }),
  )
  expect(boxes.map((b) => b.id)).toEqual(['a', 'b', 'c', 'd', 'e'])
  // Row by row, left to right: x grows within a row, y grows between rows.
  expect(boxes[0].x).toBeLessThan(boxes[1].x)
  expect(boxes[1].x).toBeLessThan(boxes[2].x)
  expect(boxes[3].x).toBeCloseTo(boxes[0].x, 0)
  expect(boxes[4].x).toBeCloseTo(boxes[1].x, 0)
  expect(boxes[3].y).toBeGreaterThan(boxes[0].y)
  expect(boxes[2].y).toBeCloseTo(boxes[0].y, 0)
})
