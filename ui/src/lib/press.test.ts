import { afterEach, beforeEach, describe, expect, it, vi, type Mock } from 'vitest'
import { createPressDetector, HOLD_MS, MOVE_PX } from './press'

const ev = (x = 0, y = 0, pointerId = 1) => ({ pointerId, clientX: x, clientY: y })

let onTap: Mock<() => void>
let onHold: Mock<() => void>
let d: ReturnType<typeof createPressDetector>

beforeEach(() => {
  vi.useFakeTimers()
  onTap = vi.fn<() => void>()
  onHold = vi.fn<() => void>()
  d = createPressDetector(() => ({ onTap, onHold }))
})
afterEach(() => vi.useRealTimers())

describe('press detector', () => {
  it('has the documented thresholds', () => {
    expect(HOLD_MS).toBe(600)
    expect(MOVE_PX).toBe(10)
  })
  it('fires a tap on a quick release', () => {
    d.down(ev())
    vi.advanceTimersByTime(HOLD_MS - 1)
    d.up(ev())
    expect(onTap).toHaveBeenCalledTimes(1)
    expect(onHold).not.toHaveBeenCalled()
  })
  it('fires hold once at 600 ms, and no tap on release', () => {
    d.down(ev())
    vi.advanceTimersByTime(HOLD_MS)
    expect(onHold).toHaveBeenCalledTimes(1)
    vi.advanceTimersByTime(2000)
    d.up(ev())
    expect(onHold).toHaveBeenCalledTimes(1)
    expect(onTap).not.toHaveBeenCalled()
  })
  it('small movement keeps the press alive', () => {
    d.down(ev(0, 0))
    d.move(ev(MOVE_PX, 0))
    vi.advanceTimersByTime(HOLD_MS)
    expect(onHold).toHaveBeenCalledTimes(1)
  })
  it('moving more than 10 px cancels both', () => {
    d.down(ev(0, 0))
    d.move(ev(MOVE_PX + 1, 0))
    vi.advanceTimersByTime(HOLD_MS * 2)
    d.up(ev(11, 0))
    expect(onHold).not.toHaveBeenCalled()
    expect(onTap).not.toHaveBeenCalled()
  })
  it('cancel (pointercancel) cancels both', () => {
    d.down(ev())
    d.cancel()
    vi.advanceTimersByTime(HOLD_MS * 2)
    d.up(ev())
    expect(onHold).not.toHaveBeenCalled()
    expect(onTap).not.toHaveBeenCalled()
  })
  it('a second pointer cancels the gesture', () => {
    d.down(ev(0, 0, 1))
    d.down(ev(0, 0, 2))
    vi.advanceTimersByTime(HOLD_MS * 2)
    d.up(ev(0, 0, 1))
    expect(onHold).not.toHaveBeenCalled()
    expect(onTap).not.toHaveBeenCalled()
  })
  it('can be used again after a hold', () => {
    d.down(ev())
    vi.advanceTimersByTime(HOLD_MS)
    d.up(ev())
    d.down(ev())
    d.up(ev())
    expect(onHold).toHaveBeenCalledTimes(1)
    expect(onTap).toHaveBeenCalledTimes(1)
  })
  it('reports start and end of a press', () => {
    const onStart = vi.fn()
    const onEnd = vi.fn()
    const p = createPressDetector(() => ({}), { onStart, onEnd })
    p.down(ev())
    expect(onStart).toHaveBeenCalledTimes(1)
    p.up(ev())
    expect(onEnd).toHaveBeenCalledTimes(1)
  })
})
