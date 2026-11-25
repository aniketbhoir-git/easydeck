import { describe, expect, it } from 'vitest'
import { displayText, formatTime, idleState, reset, settle, tap, type TimerConfig } from './timer'

const cd: TimerConfig = { mode: 'countdown', durationS: 5 }
const sw: TimerConfig = { mode: 'stopwatch' }

describe('timer state machine', () => {
  it('tap cycles idle, running, paused, running', () => {
    const a = tap(idleState, sw, 100)
    expect(a.phase).toBe('running')
    const b = tap(a, sw, 1100)
    expect(b).toMatchObject({ phase: 'paused', accumulatedMs: 1000 })
    const c = tap(b, sw, 5000)
    expect(c.phase).toBe('running')
    expect(displayText(c, sw, 5500)).toBe('0:01')
    expect(displayText(c, sw, 7000)).toBe('0:03')
  })
  it('pause keeps elapsed time however long it stays paused', () => {
    const p = tap(tap(idleState, sw, 0), sw, 2500)
    expect(displayText(p, sw, 2500)).toBe('0:02')
    expect(displayText(p, sw, 999_999)).toBe('0:02')
  })
  it('a countdown ends at its duration, also when redraws are late', () => {
    const r = tap(idleState, cd, 1000)
    expect(displayText(r, cd, 1000)).toBe('0:05')
    expect(displayText(r, cd, 4000)).toBe('0:02')
    expect(settle(r, cd, 5999).phase).toBe('running')
    expect(settle(r, cd, 6000).phase).toBe('ended')
    expect(settle(r, cd, 60_000).phase).toBe('ended')
    expect(displayText(r, cd, 60_000)).toBe('0:00')
  })
  it('a tap on an ended countdown clears it back to idle', () => {
    const r = tap(idleState, cd, 0)
    expect(tap(r, cd, 9000)).toEqual(idleState)
    expect(displayText(idleState, cd, 9000)).toBe('0:05')
  })
  it('a tap after the time is up clears it, not pauses it', () => {
    const r = tap(idleState, cd, 0)
    expect(tap(r, cd, 5000).phase).toBe('idle')
  })
  it('reset goes to idle', () => {
    expect(reset()).toEqual(idleState)
    expect(displayText(reset(), sw, 123)).toBe('0:00')
  })
  it('a stopwatch never ends', () => {
    expect(settle(tap(idleState, sw, 0), sw, 1e12).phase).toBe('running')
  })
  it('ignores a clock reading before the start', () => {
    expect(displayText(tap(idleState, sw, 500), sw, 100)).toBe('0:00')
  })
})

describe('formatTime', () => {
  it('formats m:ss and h:mm:ss', () => {
    expect(formatTime(0, false)).toBe('0:00')
    expect(formatTime(59_999, false)).toBe('0:59')
    expect(formatTime(59_001, true)).toBe('1:00')
    expect(formatTime(3_600_000, false)).toBe('1:00:00')
    expect(formatTime(86_399_000, true)).toBe('23:59:59')
  })
})
