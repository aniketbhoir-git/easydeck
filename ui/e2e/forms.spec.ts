import { expect, test, type Page } from '@playwright/test'

type Call = { cmd: string; args: unknown }
const calls = (page: Page) =>
  page.evaluate(() => (window as unknown as { __ipcCalls: Call[] }).__ipcCalls)
const tile = (page: Page, id: string) => page.locator(`[data-widget-id="${id}"]`)

async function openEditor(page: Page) {
  await page.goto('/?deck=edit')
  await expect(page.locator('[data-widget-id]').first()).toBeVisible()
  await page.keyboard.press('Control+e')
  await expect(page.getByTestId('editor-bar')).toBeVisible()
}

test('launch form: change kind, see the missing-field error, fix it', async ({ page }) => {
  await openEditor(page)
  await tile(page, 'game-one').click()
  const form = page.getByTestId('widget-form')
  await expect(form.locator('[data-field="appId"]')).toHaveValue('440')
  await expect(page.getByTestId('editor-state')).toContainText('No changes')

  await form.locator('[data-field="kind"]').selectOption('exe')
  await expect(form.locator('[data-field="appId"]')).toHaveCount(0)
  await expect(page.getByTestId('field-error-action-path')).toContainText('needs the field "path"')
  await expect(page.getByTestId('editor-state')).toContainText('1 problem')
  await expect(page.locator('[data-action="save"]')).toBeDisabled()

  await form.locator('[data-field="path"]').fill(String.raw`C:\Games\game.exe`)
  await expect(page.getByTestId('field-error-action-path')).toHaveCount(0)
  await expect(page.getByTestId('editor-state')).toContainText('Unsaved changes')
  await expect(page.getByTestId('editor-state')).not.toContainText('problem')
  await expect(page.getByTestId('risk-note').first()).toContainText('run a program')

  // Back to a steam kind: the path is gone, the app id is asked for again.
  await form.locator('[data-field="kind"]').selectOption('steam')
  await expect(form.locator('[data-field="path"]')).toHaveCount(0)
  await expect(page.getByTestId('field-error-action-appId')).toBeVisible()

  expect((await calls(page)).filter((c) => c.cmd === 'run_action')).toEqual([])
})

test('a new Steam tile is built with the form and the draft validates', async ({ page }) => {
  await openEditor(page)
  await page.locator('[data-palette-type="launch"]').click()
  const form = page.getByTestId('widget-form')
  await expect(page.getByTestId('editor-state')).toContainText('1 problem')
  await form.locator('[data-action="add-action"]').click()
  await expect(page.getByTestId('field-error-action-appId')).toBeVisible()
  await form.locator('[data-field="appId"]').fill('730')
  await expect(page.getByTestId('editor-state')).not.toContainText('problem')

  await expect(page.getByTestId('editor-state')).toContainText('Unsaved changes')
  expect((await calls(page)).filter((c) => c.cmd === 'run_action')).toEqual([])
})

test('macro form: type keys, record keys, numbers and toggle', async ({ page }) => {
  await openEditor(page)
  await tile(page, 'mic').click()
  const slot = page.locator('[data-slot="onAction"]')
  await expect(slot.locator('[data-field="keys"]')).toHaveValue('ctrl+m')

  await slot.locator('[data-field="keys"]').fill('')
  await expect(page.getByTestId('field-error-onAction-keys')).toBeVisible()

  await slot.locator('[data-action="record-keys"]').click()
  await page.keyboard.press('Control+q')
  await page.keyboard.press('5')
  await slot.locator('[data-action="record-keys"]').click()
  await expect(slot.locator('[data-field="keys"]')).toHaveValue('ctrl+q 5')
  await expect(page.getByTestId('field-error-onAction-keys')).toHaveCount(0)

  await slot.locator('[data-field="cycleMs"]').fill('250')
  await slot.locator('[data-field="toggle"]').check()
  await expect(page.getByTestId('editor-state')).toContainText('Unsaved changes')
  await expect(page.getByTestId('editor-state')).not.toContainText('problem')
  await expect(slot.getByTestId('risk-note')).toContainText('send keys')
  expect((await calls(page)).filter((c) => c.cmd === 'run_action')).toEqual([])
})

test('recording: Esc stops it and is not recorded; Shift+1 is recorded as shift+1', async ({ page }) => {
  await openEditor(page)
  await tile(page, 'mic').click()
  const slot = page.locator('[data-slot="onAction"]')
  await slot.locator('[data-field="keys"]').fill('')
  await slot.locator('[data-action="record-keys"]').click()
  await page.keyboard.press('Shift+1')
  await page.keyboard.press('Escape')
  await expect(slot.locator('[data-action="record-keys"]')).toHaveAttribute('aria-pressed', 'false')
  await page.keyboard.press('7')
  await expect(slot.locator('[data-field="keys"]')).toHaveValue('shift+1')
})
