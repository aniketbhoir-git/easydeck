import { describe, expect, it } from 'vitest'
import { canDo, safeThumbnail, visibleControls, watchMedia, type MediaState } from './media'

const playing: MediaState = {
  title: 'T',
  artist: 'A',
  status: 'playing',
  canPlayPause: true,
  canNext: false,
  canPrevious: true,
}

describe('visibleControls', () => {
  it('shows all three in a fixed order when the deck lists none', () => {
    expect(visibleControls(undefined)).toEqual(['previous', 'play_pause', 'next'])
  })
  it('shows only the listed controls, in the fixed order', () => {
    expect(visibleControls(['next', 'play_pause'])).toEqual(['play_pause', 'next'])
    expect(visibleControls([])).toEqual([])
  })
})

describe('canDo', () => {
  it('follows what the session reports', () => {
    expect(canDo(playing, 'play_pause')).toBe(true)
    expect(canDo(playing, 'next')).toBe(false)
    expect(canDo(playing, 'previous')).toBe(true)
  })
  it('is false with no session or no player', () => {
    expect(canDo(null, 'play_pause')).toBe(false)
    expect(canDo({ ...playing, status: 'none' }, 'play_pause')).toBe(false)
  })
})

describe('safeThumbnail', () => {
  it('accepts a base64 image data URL', () => {
    expect(safeThumbnail('data:image/png;base64,AAAA')).toBe('data:image/png;base64,AAAA')
    expect(safeThumbnail('data:image/jpeg;base64,/9j/4AAQ==')).toBeDefined()
  })
  it('rejects every other scheme or type', () => {
    for (const bad of [
      'https://example.com/a.png',
      'http://localhost/a.png',
      'file:///C:/a.png',
      'javascript:alert(1)',
      'data:text/html;base64,AAAA',
      'data:image/svg+xml;base64,AAAA',
      'data:image/png;base64,AAAA" onerror="x',
      'data:image/png,AAAA',
      '',
      undefined,
      42,
    ]) {
      expect(safeThumbnail(bad)).toBeUndefined()
    }
  })
})

describe('watchMedia', () => {
  const tick = () => new Promise((r) => setTimeout(r, 0))
  const newer: MediaState = { ...playing, title: 'Newer' }
  const noListen = () => Promise.resolve(() => {})

  it('delivers the first read when no event came before it', async () => {
    const seen: (MediaState | null)[] = []
    watchMedia((m) => seen.push(m), () => Promise.resolve(playing), noListen)
    await tick()
    expect(seen).toEqual([playing])
  })

  it('drops a slow first read that lands after an event', async () => {
    const seen: (MediaState | null)[] = []
    let reply!: (m: MediaState) => void
    let emit!: (m: MediaState) => void
    watchMedia(
      (m) => seen.push(m),
      () => new Promise<MediaState>((r) => (reply = r)),
      (h) => {
        emit = h
        return Promise.resolve(() => {})
      },
    )
    await tick()
    emit(newer)
    reply(playing)
    await tick()
    expect(seen).toEqual([newer])
  })

  it('ignores a failed first read after an event', async () => {
    const seen: (MediaState | null)[] = []
    let fail!: () => void
    let emit!: (m: MediaState) => void
    watchMedia(
      (m) => seen.push(m),
      () => new Promise<MediaState>((_, rej) => (fail = () => rej(new Error('x')))),
      (h) => {
        emit = h
        return Promise.resolve(() => {})
      },
    )
    await tick()
    emit(newer)
    fail()
    await tick()
    expect(seen).toEqual([newer])
  })

  it('ignores a failed or late first read after stop', async () => {
    const seen: (MediaState | null)[] = []
    let fail!: () => void
    const stop = watchMedia(
      (m) => seen.push(m),
      () => new Promise<MediaState>((_, rej) => (fail = () => rej(new Error('x')))),
      noListen,
    )
    stop()
    fail()
    await tick()
    expect(seen).toEqual([])
  })

  it('reports null when the first read fails and nothing else came', async () => {
    const seen: (MediaState | null)[] = []
    watchMedia((m) => seen.push(m), () => Promise.reject(new Error('x')), noListen)
    await tick()
    expect(seen).toEqual([null])
  })
})
