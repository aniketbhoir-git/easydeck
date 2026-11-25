import { describe, expect, it } from 'vitest'
import { sourceEventName } from './sources'

describe('sourceEventName', () => {
  it('prefixes the source id', () => {
    expect(sourceEventName('stats')).toBe('source:stats')
    expect(sourceEventName('my-pc_1')).toBe('source:my-pc_1')
  })
})
