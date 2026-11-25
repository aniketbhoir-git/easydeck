import { describe, expect, it } from 'vitest'
import { flip, isHexColor, lookFor, parseStored, toggleKey } from './toggle'

describe('toggle', () => {
  it('flips between on and off', () => {
    expect(flip('off')).toBe('on')
    expect(flip('on')).toBe('off')
  })
  it('uses the look of the state', () => {
    const c = {
      label: 'Mic',
      icon: 'M',
      on: { label: 'Live', icon: 'L', color: '#c0392b' },
      off: { label: 'Muted', color: '#333' },
    }
    expect(lookFor(c, 'on')).toEqual({ label: 'Live', icon: 'L', color: '#c0392b' })
    expect(lookFor(c, 'off')).toEqual({ label: 'Muted', icon: 'M', color: '#333' })
  })
  it('falls back to the widget label and icon and the default colour', () => {
    expect(lookFor({ label: 'Lamp', icon: 'x' }, 'on')).toEqual({
      label: 'Lamp',
      icon: 'x',
      color: undefined,
    })
    expect(lookFor({ label: 'Lamp', on: { color: '#fff' } }, 'off').color).toBeUndefined()
  })
  it('never passes a colour that is not hex', () => {
    expect(lookFor({ on: { color: 'red; background: url(x)' } }, 'on').color).toBeUndefined()
    expect(isHexColor('#abc')).toBe(true)
    expect(isHexColor('#aabbcc')).toBe(true)
    expect(isHexColor('#abcd')).toBe(false)
    expect(isHexColor('abc')).toBe(false)
    expect(isHexColor(undefined)).toBe(false)
  })
  it('reads a stored state, off by default', () => {
    expect(parseStored('on')).toBe('on')
    expect(parseStored('off')).toBe('off')
    expect(parseStored(null)).toBe('off')
    expect(parseStored('junk')).toBe('off')
  })
  it('keys by deck and widget', () => {
    expect(toggleKey('D', 'w')).toBe('easydeck:widget:D:w')
  })
})
