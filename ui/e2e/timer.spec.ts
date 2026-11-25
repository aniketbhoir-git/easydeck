import { expect, test, type Page } from '@playwright/test'

// Timer fixture: `?deck=timer` in src/lib/mock-ipc.ts has a 5 s countdown ("brew") and a
// stopwatch ("lap"). Time runs on a paused fake clock, never on wall-clock waits.

const FIXED_TIME = new Date('2026-11-25T14:07:00Z')
const tile = (page: Page, id: string) => page.locator(`[data-widget-id="${id}"]`)
const value = (page: Page, id: string) => tile(page, id).getByTestId('timer-value')
const runCalls = (page: Page) =>
  page.evaluate(() =>
    (window as unknown as { __ipcCalls: { cmd: string }[] }).__ipcCalls.filter((c) => c.cmd === 'run_action'),
  )

test.beforeEach(async ({ page }) => {
  await page.clock.install({ time: new Date(FIXED_TIME.getTime() - 60_000) })
  await page.goto('/?deck=timer')
  await expect(value(page, 'brew')).toBeVisible()
  await page.clock.pauseAt(FIXED_TIME)
})

async function press(page: Page, id: string, ms: number) {
  const box = (await tile(page, id).boundingBox())!
  await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2)
  await page.mouse.down()
  await page.clock.runFor(ms)
  await page.mouse.up()
}
const tapTile = (page: Page, id: string) => press(page, id, 50)
const holdTile = (page: Page, id: string) => press(page, id, 700)

test('a 5 s countdown ends, pulses until the next tap, and a tap clears it', async ({ page }) => {
  await expect(value(page, 'brew')).toHaveText('0:05')
  await tapTile(page, 'brew')
  await page.clock.runFor(2000)
  await expect(value(page, 'brew')).toHaveText('0:03')
  await expect(tile(page, 'brew')).not.toHaveClass(/ended/)
  await page.clock.runFor(3000)
  await expect(value(page, 'brew')).toHaveText('0:00')
  await expect(tile(page, 'brew')).toHaveClass(/ended/)
  await page.clock.runFor(5000)
  await expect(tile(page, 'brew')).toHaveClass(/ended/)
  await tapTile(page, 'brew')
  await expect(tile(page, 'brew')).not.toHaveClass(/ended/)
  await expect(value(page, 'brew')).toHaveText('0:05')
  expect(await runCalls(page)).toHaveLength(0)
})

test('stopwatch pause and resume keep the elapsed time', async ({ page }) => {
  await tapTile(page, 'lap')
  await page.clock.runFor(3000)
  await expect(value(page, 'lap')).toHaveText('0:03')
  await tapTile(page, 'lap')
  const frozen = await value(page, 'lap').textContent()
  await page.clock.runFor(10_000)
  await expect(value(page, 'lap')).toHaveText(frozen!)
  await tapTile(page, 'lap')
  await page.clock.runFor(2000)
  await expect(value(page, 'lap')).toHaveText('0:05')
})

test('a long press resets a running timer', async ({ page }) => {
  await tapTile(page, 'lap')
  await page.clock.runFor(4000)
  await holdTile(page, 'lap')
  await expect(value(page, 'lap')).toHaveText('0:00')
  await page.clock.runFor(3000)
  await expect(value(page, 'lap')).toHaveText('0:00')
})

test('screenshot: ended countdown', async ({ page }) => {
  await tapTile(page, 'brew')
  await page.clock.runFor(5000)
  await expect(tile(page, 'brew')).toHaveClass(/ended/)
  await expect(page).toHaveScreenshot('timer-ended.png', { animations: 'disabled' })
})
