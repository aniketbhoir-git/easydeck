import { render, screen, cleanup, fireEvent } from '@testing-library/svelte'
import { tick } from 'svelte'
import { afterEach, describe, expect, it, vi } from 'vitest'

type VolumeHandler = (v: { level: number; muted: boolean }) => void

const api = vi.hoisted(() => ({
  get: vi.fn(),
  set: vi.fn(),
  mute: vi.fn(),
  handlers: [] as VolumeHandler[],
  unlisten: vi.fn(),
}))

vi.mock('../lib/volume', async (orig) => ({
  ...(await orig<typeof import('../lib/volume')>()),
  volumeGet: api.get,
  volumeSet: api.set,
  volumeMute: api.mute,
  onVolume: vi.fn(async (h: VolumeHandler) => {
    api.handlers.push(h)
    return api.unlisten
  }),
}))

import Volume from './Volume.svelte'

const widget = { id: 'vol', type: 'volume', label: 'Sound', position: { x: 0, y: 0 } } as const
const settle = async () => {
  await Promise.resolve()
  await tick()
  await Promise.resolve()
  await tick()
}

afterEach(() => {
  cleanup()
  vi.clearAllMocks()
  api.handlers.length = 0
})

describe('Volume', () => {
  it('shows the level and mute state from volume_get', async () => {
    api.get.mockResolvedValue({ level: 40, muted: false })
    render(Volume, { widget })
    await settle()
    expect(screen.getByTestId('volume-level').textContent?.trim()).toBe('40%')
    expect(screen.getByTestId('volume-track').getAttribute('aria-valuenow')).toBe('40')
    expect(screen.getByTestId('volume-mute').getAttribute('aria-pressed')).toBe('false')
  })

  it('shows a no-data state when volume_get fails, with mute disabled', async () => {
    api.get.mockRejectedValue(new Error('no device'))
    render(Volume, { widget })
    await settle()
    expect(screen.getByTestId('volume').dataset.state).toBe('nodata')
    expect(screen.getByTestId('volume-level').textContent?.trim()).toBe('no data')
    expect(screen.getByTestId('volume-mute')).toHaveProperty('disabled', true)
  })

  it('mute toggles with a clear off state and calls volume_mute', async () => {
    api.get.mockResolvedValue({ level: 40, muted: false })
    api.mute.mockResolvedValue(undefined)
    render(Volume, { widget })
    await settle()
    await fireEvent.click(screen.getByTestId('volume-mute'))
    expect(api.mute).toHaveBeenCalledWith(true)
    expect(screen.getByTestId('volume').dataset.state).toBe('muted')
    expect(screen.getByTestId('volume-level').textContent?.trim()).toBe('muted')
    expect(screen.getByTestId('volume-mute').getAttribute('aria-label')).toBe('Unmute Sound')
  })

  it('follows a volume event from outside (muted in Windows)', async () => {
    api.get.mockResolvedValue({ level: 40, muted: false })
    render(Volume, { widget })
    await settle()
    api.handlers[0]({ level: 10, muted: true })
    await tick()
    expect(screen.getByTestId('volume-level').textContent?.trim()).toBe('muted')
    expect(screen.getByTestId('volume-track').getAttribute('aria-valuenow')).toBe('10')
  })

  it('arrow keys change the level through the throttled sender', async () => {
    api.get.mockResolvedValue({ level: 40, muted: false })
    api.set.mockResolvedValue(undefined)
    render(Volume, { widget })
    await settle()
    await fireEvent.keyDown(screen.getByTestId('volume-track'), { key: 'ArrowRight' })
    expect(api.set).toHaveBeenCalledWith(45)
  })

  it('resyncs from volume_get once the local-change window ends', async () => {
    vi.useFakeTimers()
    try {
      api.get.mockResolvedValue({ level: 40, muted: false })
      api.set.mockResolvedValue(undefined)
      render(Volume, { widget })
      await settle()
      await fireEvent.keyDown(screen.getByTestId('volume-track'), { key: 'ArrowRight' })
      // An outside change during the window is ignored, and the poller will not re-send it.
      api.get.mockResolvedValue({ level: 80, muted: false })
      api.handlers.forEach((h) => h({ level: 80, muted: false }))
      await settle()
      expect(screen.getByTestId('volume-level').textContent?.trim()).toBe('45%')
      await vi.advanceTimersByTimeAsync(800)
      await settle()
      expect(screen.getByTestId('volume-level').textContent?.trim()).toBe('80%')
    } finally {
      vi.useRealTimers()
    }
  })

  it('stops listening on unmount', async () => {
    api.get.mockResolvedValue({ level: 40, muted: false })
    render(Volume, { widget })
    await settle()
    cleanup()
    expect(api.unlisten).toHaveBeenCalled()
  })
})
