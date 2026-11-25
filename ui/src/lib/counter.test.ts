import { describe, expect, it } from 'vitest'
import { clamp, counterKey, nextValue, parseStored, resetValue } from './counter'

describe('counter', () => {
  it('defaults to start 0 and step 1', () => {
    expect(nextValue(0, {})).toBe(1)
    expect(resetValue({})).toBe(0)
  })
  it('adds step, also negative', () => {
    expect(nextValue(5, { step: 3 })).toBe(8)
    expect(nextValue(5, { step: -2 })).toBe(3)
  })
  it('clamps to min and max', () => {
    expect(nextValue(9, { step: 3, max: 10 })).toBe(10)
    expect(nextValue(1, { step: -3, min: 0 })).toBe(0)
    expect(clamp(5, 0, 10)).toBe(5)
  })
  it('reset gives start', () => {
    expect(resetValue({ start: 4, step: 2 })).toBe(4)
  })
  it('parses a stored value, else start', () => {
    const c = { start: 7, min: 0, max: 10 }
    expect(parseStored('4', c)).toBe(4)
    expect(parseStored(null, c)).toBe(7)
    expect(parseStored(undefined, c)).toBe(7)
    expect(parseStored('abc', c)).toBe(7)
    expect(parseStored('1.5', c)).toBe(7)
    expect(parseStored('', c)).toBe(7)
    expect(parseStored('99999999999999999999', c)).toBe(7)
    expect(parseStored('50', c)).toBe(10)
  })
  it('builds the key from deck name and widget id', () => {
    expect(counterKey('My deck', 'water')).toBe('easydeck:widget:My deck:water')
  })
})
