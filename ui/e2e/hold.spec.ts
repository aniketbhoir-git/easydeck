import { expect, test, type Page } from '@playwright/test'

// Long-press tests run on a paused fake clock: the hold timer (600 ms) fires only when the
// test advances time with `runFor`, so CPU load cannot turn a hold into a tap.

const runCalls = (page: Page) =>
  page.evaluate(() =>
    (window as unknown as { __ipcCalls: { cmd: string; args: unknown }[] }).__ipcCalls.filter(
      (c) => c.cmd === 'run_action',
    ),
  )

const FIXED_TIME = new Date('2026-11-25T14:07:00Z')

test.beforeEach(async ({ page }) => {
  await page.clock.install({ time: new Date(FIXED_TIME.getTime() - 60_000) })
  await page.goto('/')
  await expect(page.locator('[data-widget-id]').first()).toBeVisible()
  await page.clock.pauseAt(FIXED_TIME)
})

async function holdFor(page: Page, widgetId: string, ms: number) {
  const box = (await page.locator(`[data-widget-id="${widgetId}"]`).boundingBox())!
  await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2)
  await page.mouse.down()
  await page.clock.runFor(ms)
  await page.mouse.up()
}

test('a long press calls run_action once for the holdAction and no tap', async ({ page }) => {
  await holdFor(page, 'notes', 700)
  await expect.poll(() => runCalls(page)).toEqual([
    { cmd: 'run_action', args: { widgetId: 'notes', hold: true } },
  ])
  await page.clock.runFor(1000)
  expect(await runCalls(page)).toHaveLength(1)
})

test('a press shorter than the hold time is a tap', async ({ page }) => {
  await holdFor(page, 'notes', 500)
  await expect.poll(() => runCalls(page)).toEqual([{ cmd: 'run_action', args: { widgetId: 'notes' } }])
  await page.clock.runFor(1000)
  expect(await runCalls(page)).toHaveLength(1)
})

test('a long press on a tile without holdAction is a tap', async ({ page }) => {
  await holdFor(page, 'game-one', 700)
  await expect.poll(() => runCalls(page)).toEqual([{ cmd: 'run_action', args: { widgetId: 'game-one' } }])
})
