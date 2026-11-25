import { expect, test, type Page } from '@playwright/test'

// Counter fixture: start 2, step 3, max 20 (`?deck=counter` in src/lib/mock-ipc.ts).
// Hold timing runs on a paused fake clock, never on wall-clock waits.

const FIXED_TIME = new Date('2026-11-25T14:07:00Z')
const tile = (page: Page) => page.locator('[data-widget-id="water"]')
const value = (page: Page) => tile(page).getByTestId('counter-value')
const runCalls = (page: Page) =>
  page.evaluate(() =>
    (window as unknown as { __ipcCalls: { cmd: string }[] }).__ipcCalls.filter((c) => c.cmd === 'run_action'),
  )

test.beforeEach(async ({ page }) => {
  await page.clock.install({ time: new Date(FIXED_TIME.getTime() - 60_000) })
  await page.goto('/?deck=counter')
  await expect(value(page)).toBeVisible()
  await page.clock.pauseAt(FIXED_TIME)
})

async function holdFor(page: Page, ms: number) {
  const box = (await tile(page).boundingBox())!
  await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2)
  await page.mouse.down()
  await page.clock.runFor(ms)
  await page.mouse.up()
}

test('three taps show start + 3 * step, with no IPC call', async ({ page }) => {
  await expect(value(page)).toHaveText('2')
  for (let i = 0; i < 3; i++) await holdFor(page, 50)
  await expect(value(page)).toHaveText('11')
  expect(await runCalls(page)).toHaveLength(0)
})

test('a long press resets to start', async ({ page }) => {
  await holdFor(page, 50)
  await holdFor(page, 50)
  await expect(value(page)).toHaveText('8')
  await holdFor(page, 700)
  await expect(value(page)).toHaveText('2')
  await page.clock.runFor(1000)
  await expect(value(page)).toHaveText('2')
  expect(await runCalls(page)).toHaveLength(0)
})

test('the count stops at max', async ({ page }) => {
  for (let i = 0; i < 8; i++) await holdFor(page, 50)
  await expect(value(page)).toHaveText('20')
})

test('a reload keeps the value', async ({ page }) => {
  await holdFor(page, 50)
  await holdFor(page, 50)
  await expect(value(page)).toHaveText('8')
  await page.reload()
  await expect(value(page)).toHaveText('8')
})
