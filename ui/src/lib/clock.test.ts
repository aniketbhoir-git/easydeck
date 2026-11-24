import { describe, expect, it } from 'vitest'
import { formatClock, msToNextSecond } from './clock'

const flat = (s: string) => s.replace(/\s/g, ' ')
const at = new Date('2026-11-25T15:05:09Z')
const base = { locale: 'en-US', timeZone: 'UTC' }

describe('formatClock', () => {
  it('formats 24-hour time without seconds', () => {
    expect(formatClock(at, { ...base, hour12: false }).time).toBe('15:05')
  })
  it('formats 24-hour time with seconds', () => {
    expect(formatClock(at, { ...base, hour12: false, seconds: true }).time).toBe('15:05:09')
  })
  it('formats 12-hour time', () => {
    expect(flat(formatClock(at, { ...base, hour12: true }).time)).toBe('3:05 PM')
    expect(flat(formatClock(at, { ...base, hour12: true, seconds: true }).time)).toBe('3:05:09 PM')
  })
  it('shows midnight as 00:05 in 24-hour mode', () => {
    expect(formatClock(new Date('2026-11-25T00:05:00Z'), { ...base, hour12: false }).time).toBe('00:05')
  })
  it('formats the date line', () => {
    expect(formatClock(at, base).date).toBe('Wed, Nov 25')
  })
  it('follows the locale', () => {
    expect(formatClock(at, { locale: 'de-DE', timeZone: 'UTC' }).time).toBe('15:05')
  })
})

describe('msToNextSecond', () => {
  it('aligns to the second boundary', () => {
    expect(msToNextSecond(10_250)).toBe(750)
    expect(msToNextSecond(10_000)).toBe(1000)
  })
})
