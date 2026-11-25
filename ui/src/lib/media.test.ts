import { describe, expect, it } from 'vitest'
import { canDo, safeThumbnail, visibleControls, type MediaState } from './media'

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
