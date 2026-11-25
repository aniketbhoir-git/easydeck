import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { createLatestThrottle, isVertical, levelFromPointer } from './volume'

describe('createLatestThrottle', () => {
  beforeEach(() => {
    vi.useFakeTimers()
    vi.setSystemTime(1_000_000)
  })
  afterEach(() => vi.useRealTimers())

  it('sends the first value at once and at most one per 50 ms after', () => {
    const send = vi.fn()
    const t = createLatestThrottle(send)
    t.push(10)
    expect(send).toHaveBeenCalledTimes(1)
    for (let i = 11; i <= 20; i++) {
      vi.advanceTimersByTime(4)
      t.push(i)
    }
    expect(send).toHaveBeenCalledTimes(1)
    vi.advanceTimersByTime(50)
    expect(send).toHaveBeenCalledTimes(2)
    expect(send).toHaveBeenLastCalledWith(20)
  })

  it('always sends the final value after the last push', () => {
    const send = vi.fn()
    const t = createLatestThrottle(send)
    t.push(1)
    t.push(2)
    t.push(3)
    vi.advanceTimersByTime(60)
    expect(send.mock.calls.map((c) => c[0])).toEqual([1, 3])
    vi.advanceTimersByTime(500)
    expect(send).toHaveBeenCalledTimes(2)
  })

  it('sends again at once after a quiet period', () => {
    const send = vi.fn()
    const t = createLatestThrottle(send)
    t.push(1)
    vi.advanceTimersByTime(200)
    t.push(2)
    expect(send).toHaveBeenCalledTimes(2)
  })

  it('cancel drops the pending value', () => {
    const send = vi.fn()
    const t = createLatestThrottle(send)
    t.push(1)
    t.push(2)
    t.cancel()
    vi.advanceTimersByTime(200)
    expect(send).toHaveBeenCalledTimes(1)
  })
})

describe('levelFromPointer', () => {
  it('maps a horizontal track left to right', () => {
    expect(levelFromPointer(100, 100, 200, false)).toBe(0)
    expect(levelFromPointer(200, 100, 200, false)).toBe(50)
    expect(levelFromPointer(300, 100, 200, false)).toBe(100)
  })
  it('maps a vertical track bottom to top', () => {
    expect(levelFromPointer(300, 100, 200, true)).toBe(0)
    expect(levelFromPointer(100, 100, 200, true)).toBe(100)
    expect(levelFromPointer(250, 100, 200, true)).toBe(25)
  })
  it('clamps outside the track and survives a zero size', () => {
    expect(levelFromPointer(-50, 0, 100, false)).toBe(0)
    expect(levelFromPointer(500, 0, 100, false)).toBe(100)
    expect(levelFromPointer(5, 0, 0, false)).toBe(0)
  })
})

describe('isVertical', () => {
  it('is vertical only when taller than wide', () => {
    expect(isVertical(1, 2)).toBe(true)
    expect(isVertical(2, 1)).toBe(false)
    expect(isVertical(1, 1)).toBe(false)
    expect(isVertical()).toBe(false)
  })
})
