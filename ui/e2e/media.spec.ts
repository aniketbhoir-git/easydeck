import { expect, test, type Page } from '@playwright/test'

// Media fixture (?deck=media in src/lib/mock-ipc.ts): one tile with every control and one
// that lists only play/pause, over an in-memory fake media session.

const all = (page: Page) => page.locator('[data-widget-id="media-all"]')
const one = (page: Page) => page.locator('[data-widget-id="media-one"]')

type Win = { __ipcCalls: { cmd: string; args: unknown }[] }
const calls = (page: Page, cmd: string) =>
  page.evaluate(
    (c) => (window as unknown as Win).__ipcCalls.filter((x) => x.cmd === c).map((x) => x.args),
    cmd,
  )

test('shows the track from the fake session, with its thumbnail', async ({ page }) => {
  await page.goto('/?deck=media')
  await expect(all(page).getByTestId('media-title')).toHaveText('Test Track One')
  await expect(all(page).getByTestId('media-artist')).toHaveText('Test Artist')
  await expect(all(page).getByTestId('media-thumbnail')).toHaveAttribute('src', /^data:image\/png;base64,/)
  await expect(all(page).getByTestId('media')).toHaveAttribute('data-state', 'playing')
  await expect(all(page).getByTestId('media-play_pause')).toHaveAccessibleName('Pause')
})

test('each button sends one media_command and runs no action', async ({ page }) => {
  await page.goto('/?deck=media')
  await expect(all(page).getByTestId('media-title')).toBeVisible()
  await all(page).getByTestId('media-previous').click()
  await all(page).getByTestId('media-next').click()
  await all(page).getByTestId('media-play_pause').click()
  expect(await calls(page, 'media_command')).toEqual([
    { command: 'previous' },
    { command: 'next' },
    { command: 'play_pause' },
  ])
  expect(await calls(page, 'run_action')).toHaveLength(0)
  // The session's events update both tiles.
  await expect(all(page).getByTestId('media-title')).toHaveText('Test Track Two')
  await expect(one(page).getByTestId('media-title')).toHaveText('Test Track Two')
  await expect(all(page).getByTestId('media')).toHaveAttribute('data-state', 'paused')
  await expect(all(page).getByTestId('media-play_pause')).toHaveAccessibleName('Play')
})

test('only the listed controls render', async ({ page }) => {
  await page.goto('/?deck=media')
  await expect(one(page).getByRole('button')).toHaveCount(1)
  await expect(one(page).getByTestId('media-play_pause')).toBeVisible()
  await expect(all(page).getByRole('button')).toHaveCount(3)
})

test('a change pushed by the app updates the tile and disables a control', async ({ page }) => {
  await page.goto('/?deck=media')
  await expect(all(page).getByTestId('media-next')).toBeEnabled()
  await page.evaluate(() =>
    (
      window as unknown as { __emitMedia: (p: Record<string, unknown>) => Promise<void> }
    ).__emitMedia({ title: 'Pushed', artist: '', canNext: false }),
  )
  await expect(all(page).getByTestId('media-title')).toHaveText('Pushed')
  await expect(all(page).getByTestId('media-artist')).toHaveCount(0)
  await expect(all(page).getByTestId('media-next')).toBeDisabled()
})

test('shows the neutral state when no player runs', async ({ page }) => {
  await page.goto('/?deck=media&media=none')
  await expect(all(page).getByTestId('media')).toHaveAttribute('data-state', 'none')
  await expect(all(page).getByTestId('media-title')).toHaveText('Nothing playing')
  await expect(all(page).getByTestId('media-thumbnail')).toHaveCount(0)
  for (const c of ['previous', 'play_pause', 'next']) {
    await expect(all(page).getByTestId(`media-${c}`)).toBeDisabled()
  }
})

test('shows the neutral state when the session cannot be read', async ({ page }) => {
  await page.goto('/?deck=media&media=off')
  await expect(all(page).getByTestId('media-title')).toHaveText('Nothing playing')
  await expect(all(page).getByTestId('media-play_pause')).toBeDisabled()
})

test('a thumbnail that is not an image data URL is not drawn', async ({ page }) => {
  await page.goto('/?deck=media')
  await expect(all(page).getByTestId('media-thumbnail')).toBeVisible()
  await page.evaluate(() =>
    (
      window as unknown as { __emitMedia: (p: Record<string, unknown>) => Promise<void> }
    ).__emitMedia({ thumbnail: 'https://example.com/a.png' }),
  )
  await expect(all(page).getByTestId('media-thumbnail')).toHaveCount(0)
})
