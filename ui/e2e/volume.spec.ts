import { expect, test, type Page } from '@playwright/test'

// Volume fixture (?deck=volume in src/lib/mock-ipc.ts): a wide tile and a tall tile, with an
// in-memory system volume that starts at 40. Throttle timing runs on a paused fake clock.

const FIXED_TIME = new Date('2026-11-25T14:07:00Z')
const wide = (page: Page) => page.locator('[data-widget-id="vol-wide"]')
const tall = (page: Page) => page.locator('[data-widget-id="vol-tall"]')

type Win = { __ipcCalls: { cmd: string; args: unknown }[] }
const calls = (page: Page, cmd: string) =>
  page.evaluate(
    (c) => (window as unknown as Win).__ipcCalls.filter((x) => x.cmd === c).map((x) => x.args),
    cmd,
  )
const sets = async (page: Page) => (await calls(page, 'volume_set')) as { level: number }[]

test.beforeEach(async ({ page }) => {
  await page.clock.install({ time: new Date(FIXED_TIME.getTime() - 60_000) })
  await page.goto('/?deck=volume')
  await expect(wide(page).getByTestId('volume-level')).toHaveText('40%')
  await page.clock.pauseAt(FIXED_TIME)
})

async function dragAcross(page: Page, tile: ReturnType<typeof wide>, steps: number, stepMs: number) {
  const box = (await tile.getByTestId('volume-track').boundingBox())!
  const y = box.y + box.height / 2
  await page.mouse.move(box.x + 2, y)
  await page.mouse.down()
  for (let i = 1; i <= steps; i++) {
    await page.mouse.move(box.x + 2 + ((box.width - 4) * i) / steps, y)
    await page.clock.runFor(stepMs)
  }
  await page.mouse.up()
}

test('dragging sends at most one set per 50 ms and ends on the final value', async ({ page }) => {
  // 40 moves, 5 ms apart: 200 ms of drag. Unthrottled that is 40 calls; throttled, about 5.
  await dragAcross(page, wide(page), 40, 5)
  await page.clock.runFor(100)
  const all = await sets(page)
  expect(all.length).toBeGreaterThanOrEqual(2)
  expect(all.length).toBeLessThanOrEqual(6)
  const last = all[all.length - 1].level
  expect(last).toBeGreaterThanOrEqual(95)
  await expect(wide(page).getByTestId('volume-level')).toHaveText(`${last}%`)
  expect(await calls(page, 'run_action')).toHaveLength(0)
})

test('the final value of a drag reaches the system, a quick drag inside one window still ends on it', async ({ page }) => {
  await dragAcross(page, wide(page), 10, 1)
  await page.clock.runFor(100)
  const all = await sets(page)
  const last = all[all.length - 1].level
  expect(all.length).toBeLessThanOrEqual(3)
  await expect(wide(page).getByTestId('volume-level')).toHaveText(`${last}%`)
  expect(last).toBeGreaterThanOrEqual(95)
})

test('a vertical tile drags bottom to top', async ({ page }) => {
  const box = (await tall(page).getByTestId('volume-track').boundingBox())!
  await page.mouse.move(box.x + box.width / 2, box.y + box.height - 1)
  await page.mouse.down()
  await page.mouse.move(box.x + box.width / 2, box.y + box.height / 4)
  await page.clock.runFor(100)
  await page.mouse.up()
  await page.clock.runFor(100)
  const all = await sets(page)
  const last = all[all.length - 1].level
  expect(last).toBeGreaterThanOrEqual(73)
  expect(last).toBeLessThanOrEqual(77)
  await expect(tall(page).getByTestId('volume')).toHaveClass(/vertical/)
  await expect(wide(page).getByTestId('volume')).not.toHaveClass(/vertical/)
})

test('mute shows a clear off state and a Windows-side change updates the tile', async ({ page }) => {
  const mute = wide(page).getByTestId('volume-mute')
  await mute.click()
  await expect(wide(page).getByTestId('volume')).toHaveAttribute('data-state', 'muted')
  await expect(wide(page).getByTestId('volume-level')).toHaveText('muted')
  expect(await calls(page, 'volume_mute')).toEqual([{ muted: true }])
  await mute.click()
  await expect(wide(page).getByTestId('volume')).toHaveAttribute('data-state', 'on')
  // Muted from outside: the app emits the volume event.
  await page.evaluate(() =>
    (
      window as unknown as { __emitVolume: (p: { level: number; muted: boolean }) => Promise<void> }
    ).__emitVolume({ level: 12, muted: true }),
  )
  await expect(wide(page).getByTestId('volume-level')).toHaveText('muted')
  await expect(tall(page).getByTestId('volume')).toHaveAttribute('data-state', 'muted')
})

test('shows no data when the volume cannot be read', async ({ page }) => {
  await page.goto('/?deck=volume&volume=off')
  await expect(wide(page).getByTestId('volume')).toHaveAttribute('data-state', 'nodata')
  await expect(wide(page).getByTestId('volume-mute')).toBeDisabled()
})
