import { expect, test, type Page } from '@playwright/test'

type Call = { cmd: string; args: unknown }
const calls = (page: Page) =>
  page.evaluate(() => (window as unknown as { __ipcCalls: Call[] }).__ipcCalls)
const sideEffects = async (page: Page) =>
  (await calls(page)).filter((c) => c.cmd === 'run_action' || /^volume_(set|mute)$/.test(c.cmd))

const tile = (page: Page, id: string) => page.locator(`[data-widget-id="${id}"]`)

async function openEditor(page: Page, deck = 'edit') {
  await page.goto(`/?deck=${deck}`)
  await expect(page.locator('[data-widget-id]').first()).toBeVisible()
  await page.keyboard.press('Control+e')
  await expect(page.getByTestId('editor-bar')).toBeVisible()
}

test('Ctrl+E enters edit mode and shows the palette; Done leaves it with no changes', async ({
  page,
}) => {
  await page.goto('/')
  await expect(page.locator('[data-widget-id]').first()).toBeVisible()
  await expect(page.getByRole('region', { name: 'Widget palette' })).toHaveCount(0)
  await page.keyboard.press('Control+e')
  await expect(page.getByRole('region', { name: 'Widget palette' })).toBeVisible()
  await expect(page.getByTestId('editor-state')).toContainText('No changes')
  await page.locator('[data-action="done"]').click()
  await expect(page.getByTestId('editor-bar')).toHaveCount(0)
})

test('a long press on the deck background enters edit mode; on a tile it does not', async ({
  page,
}) => {
  await page.clock.install({ time: new Date('2026-11-25T14:00:00Z') })
  await page.goto('/?deck=edit')
  await expect(page.locator('[data-widget-id]').first()).toBeVisible()
  await page.clock.pauseAt(new Date('2026-11-25T14:01:00Z'))

  const box = (await tile(page, 'game-one').boundingBox())!
  await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2)
  await page.mouse.down()
  await page.clock.runFor(1000)
  await page.mouse.up()
  await expect(page.getByTestId('editor-bar')).toHaveCount(0)

  await page.mouse.move(5, 5)
  await page.mouse.down()
  await page.clock.runFor(1000)
  await page.mouse.up()
  await expect(page.getByTestId('editor-bar')).toBeVisible()
})

test('a tap in edit mode selects the tile and runs nothing', async ({ page }) => {
  await openEditor(page)
  const before = await sideEffects(page)
  const counterText = await tile(page, 'water').innerText()
  const timerText = await tile(page, 'brew').innerText()
  const micPressed = await tile(page, 'mic').getAttribute('aria-pressed')

  for (const id of ['water', 'brew', 'mic', 'vol', 'game-one']) {
    await tile(page, id).click()
    await expect(page.getByTestId('inspector-details')).toContainText(id)
    await expect(tile(page, id)).toHaveClass(/selected/)
  }
  // A long press on a counter resets it in run mode; in edit mode it does nothing.
  await tile(page, 'water').click({ delay: 800 })

  expect(await sideEffects(page)).toEqual(before)
  expect(await tile(page, 'water').innerText()).toBe(counterText)
  expect(await tile(page, 'brew').innerText()).toBe(timerText)
  expect(await tile(page, 'mic').getAttribute('aria-pressed')).toBe(micPressed)
})

test('tiles run again after leaving edit mode', async ({ page }) => {
  await openEditor(page)
  await page.locator('[data-action="done"]').click()
  await tile(page, 'game-one').click()
  await expect.poll(async () => (await sideEffects(page)).length).toBe(1)
})

test('adding a widget places it in the first free cell and selects it', async ({ page }) => {
  await openEditor(page)
  await page.locator('[data-palette-type="text"]').click()
  const added = tile(page, 'text-1')
  await expect(added).toBeVisible()
  await expect(added).toHaveClass(/selected/)
  await expect(page.getByTestId('inspector-details')).toContainText('column 3, row 2')
  await expect(page.getByTestId('editor-state')).toContainText('Unsaved changes')

  // The page is now full: a second add is refused.
  await page.locator('[data-palette-type="counter"]').click()
  await expect(page.getByTestId('editor-state')).toContainText('full')
  await expect(page.locator('[data-widget-id]')).toHaveCount(6)
})

test('an invalid draft shows its errors inline and cannot be saved', async ({ page }) => {
  await openEditor(page)
  await page.locator('[data-palette-type="launch"]').click()
  await expect(page.getByTestId('inspector-errors')).toContainText('action is required')
  await expect(page.getByTestId('editor-state')).toContainText('1 problem')
  await expect(page.locator('[data-action="save"]')).toBeDisabled()
  expect((await calls(page)).filter((c) => c.cmd === 'validate_deck').length).toBeGreaterThanOrEqual(2)
})

test('leaving with unsaved changes asks; keep editing stays, discard leaves', async ({ page }) => {
  await openEditor(page)
  await page.locator('[data-palette-type="text"]').click()
  await page.locator('[data-action="done"]').click()
  const dialog = page.getByRole('dialog')
  await expect(dialog).toContainText('Unsaved changes')
  await expect(dialog.locator('[data-action="confirm-save"]')).toBeDisabled()

  await dialog.locator('[data-action="keep-editing"]').click()
  await expect(dialog).toHaveCount(0)
  await expect(tile(page, 'text-1')).toBeVisible()

  await page.locator('[data-action="done"]').click()
  await page.getByRole('dialog').locator('[data-action="confirm-discard"]').click()
  await expect(page.getByTestId('editor-bar')).toHaveCount(0)
  await expect(tile(page, 'text-1')).toHaveCount(0)
})

test('the Discard button reverts the draft', async ({ page }) => {
  await openEditor(page)
  await page.locator('[data-palette-type="text"]').click()
  await page.locator('[data-action="discard"]').click()
  await expect(tile(page, 'text-1')).toHaveCount(0)
  await expect(page.getByTestId('editor-state')).toContainText('No changes')
})

test('edit mode keeps the page choice and the running deck state', async ({ page }) => {
  await page.goto('/?deck=edit')
  await tile(page, 'water').click()
  const text = await tile(page, 'water').innerText()
  await page.getByRole('button', { name: 'Full', exact: true }).click()
  await page.keyboard.press('Control+e')
  await expect(page.locator('[data-page-id="full"]')).toBeVisible()
  // The Full page has one cell and it is taken, so adding is refused there.
  await page.locator('[data-palette-type="text"]').click()
  await expect(page.getByTestId('editor-state')).toContainText('full')
  await page.getByRole('button', { name: 'Home', exact: true }).click()
  expect(await tile(page, 'water').innerText()).toBe(text)
})

test('edit controls are at least 48 px', async ({ page }) => {
  await openEditor(page)
  for (const loc of [
    page.locator('[data-palette-type]'),
    page.locator('[data-action="done"]'),
    page.locator('[data-action="save"]'),
  ]) {
    for (const b of await loc.all()) {
      const box = (await b.boundingBox())!
      expect(box.height).toBeGreaterThanOrEqual(48)
      expect(box.width).toBeGreaterThanOrEqual(48)
    }
  }
})
