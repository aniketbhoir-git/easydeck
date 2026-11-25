import { expect, test, type Page } from '@playwright/test'

// Risk deck: `?deck=risk` in src/lib/mock-ipc.ts. `ok` is approved; `evil` and `desktop` are
// not. A locked tile runs nothing; a tap opens the review screen.

const tile = (page: Page, id: string) => page.locator(`[data-widget-id="${id}"]`)
const calls = (page: Page, cmd: string) =>
  page.evaluate(
    (c) =>
      (window as unknown as { __ipcCalls: { cmd: string; args: unknown }[] }).__ipcCalls
        .filter((x) => x.cmd === c)
        .map((x) => x.args),
    cmd,
  )

test.beforeEach(async ({ page }) => {
  await page.goto('/?deck=risk')
  await expect(tile(page, 'ok')).toBeVisible()
  await expect(tile(page, 'evil')).toHaveAttribute('data-locked', 'true')
})

test('a locked tile shows a lock and the others work', async ({ page }) => {
  await expect(tile(page, 'desktop')).toHaveAttribute('data-locked', 'true')
  await expect(tile(page, 'evil').getByTestId('lock-badge')).toBeVisible()
  await expect(tile(page, 'ok')).not.toHaveAttribute('data-locked', 'true')
  await expect(tile(page, 'clock')).not.toHaveAttribute('data-locked', 'true')
  await tile(page, 'ok').click()
  expect(await calls(page, 'run_action')).toEqual([{ widgetId: 'ok' }])
  await expect(page.getByTestId('risk-review')).toHaveCount(0)
})

test('tapping a locked tile opens the review screen and runs nothing', async ({ page }) => {
  await tile(page, 'evil').click()
  const dialog = page.getByTestId('risk-review')
  await expect(dialog).toBeVisible()
  expect(await calls(page, 'run_action')).toEqual([])
  // Items are listed by kind; the approved one has no checkbox.
  await expect(dialog.locator('[data-kind="launch"] .row')).toHaveCount(2)
  await expect(dialog.locator('[data-kind="key_macro"] .row')).toHaveCount(1)
  await expect(dialog.locator('[data-kind="network"] .row')).toHaveCount(1)
  await expect(dialog.locator('[data-approved="true"] input')).toHaveCount(0)
  await expect(dialog.getByTestId('system-shortcut')).toHaveCount(1)
  await expect(dialog.getByText('start program C:/Games/evil.exe')).toBeVisible()
})

test('nothing is ticked at first and approving needs a selection', async ({ page }) => {
  await tile(page, 'evil').click()
  const approve = page.getByRole('button', { name: /Approve selected/ })
  await expect(approve).toBeDisabled()
  await expect(page.locator('[data-testid="risk-review"] input:checked')).toHaveCount(0)
})

test('approving the selected item unlocks only that tile', async ({ page }) => {
  await tile(page, 'evil').click()
  await page.getByLabel(/start program C:\/Games\/evil.exe/).check()
  await page.getByRole('button', { name: 'Approve selected (1)' }).click()
  expect(await calls(page, 'approve_risk_items')).toEqual([{ hashes: ['2'.repeat(64)] }])
  await expect(tile(page, 'evil')).not.toHaveAttribute('data-locked', 'true')
  await expect(tile(page, 'desktop')).toHaveAttribute('data-locked', 'true')
  await page.getByRole('button', { name: 'Close' }).click()
  await expect(page.getByTestId('risk-review')).toHaveCount(0)
  await tile(page, 'evil').click()
  expect(await calls(page, 'run_action')).toEqual([{ widgetId: 'evil' }])
})

test('the one-time notice about a damaged ledger is shown', async ({ page }) => {
  await page.goto('/?deck=risk&notice=1')
  await expect(page.getByTestId('ledger-notice')).toContainText('approvals.json.corrupt')
  await page.getByRole('button', { name: 'Dismiss' }).click()
  await expect(page.getByTestId('ledger-notice')).toHaveCount(0)
})
