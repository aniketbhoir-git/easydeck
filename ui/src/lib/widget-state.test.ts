import { afterEach, describe, expect, it, vi } from 'vitest'
import { readState, writeState } from './widget-state'

afterEach(() => {
  vi.unstubAllGlobals()
  localStorage.clear()
})

describe('widget-state', () => {
  it('round-trips a value', () => {
    expect(writeState('k', '5')).toBe(true)
    expect(readState('k')).toBe('5')
  })
  it('returns null for a missing key', () => {
    expect(readState('nope')).toBeNull()
  })
  it('survives a throwing localStorage', () => {
    vi.stubGlobal('localStorage', {
      getItem: () => {
        throw new Error('blocked')
      },
      setItem: () => {
        throw new Error('full')
      },
    })
    expect(readState('k')).toBeNull()
    expect(writeState('k', '1')).toBe(false)
  })
})
