import { get } from 'svelte/store'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { createSourceStore } from './source-store'

type Handler = (v: Record<string, number | null>) => void

describe('source store', () => {
  let handler: Handler
  const unlisten = vi.fn()
  const subscribe = vi.fn(async (_id: string, h: Handler) => {
    handler = h
    return unlisten
  })

  beforeEach(() => {
    vi.useFakeTimers()
    subscribe.mockClear()
    unlisten.mockClear()
  })
  afterEach(() => vi.useRealTimers())

  it('falls back to no data after 3x the interval of silence, and a new event revives it', async () => {
    const store = createSourceStore(subscribe)('stats', 500)
    const seen: unknown[] = []
    const stop = store.subscribe((v) => seen.push(v))
    await vi.advanceTimersByTimeAsync(0)
    handler({ a: 1 })
    expect(get(store)).toEqual({ a: 1 })
    await vi.advanceTimersByTimeAsync(1499)
    expect(get(store)).toEqual({ a: 1 })
    await vi.advanceTimersByTimeAsync(1)
    expect(get(store)).toBeUndefined()
    handler({ a: 2 })
    expect(get(store)).toEqual({ a: 2 })
    stop()
  })

  it('defaults to a 1000 ms interval and clamps to 250', async () => {
    const make = createSourceStore(subscribe)
    const dflt = make('d')
    const fast = make('f', 10)
    const s1 = dflt.subscribe(() => {})
    await vi.advanceTimersByTimeAsync(0)
    handler({ a: 1 })
    await vi.advanceTimersByTimeAsync(2999)
    expect(get(dflt)).toEqual({ a: 1 })
    await vi.advanceTimersByTimeAsync(1)
    expect(get(dflt)).toBeUndefined()
    s1()
    const s2 = fast.subscribe(() => {})
    await vi.advanceTimersByTimeAsync(0)
    handler({ a: 1 })
    await vi.advanceTimersByTimeAsync(750)
    expect(get(fast)).toBeUndefined()
    s2()
  })

  it('resets to no data when the last subscriber leaves', async () => {
    const store = createSourceStore(subscribe)('stats')
    const stop = store.subscribe(() => {})
    await vi.advanceTimersByTimeAsync(0)
    handler({ a: 1 })
    stop()
    expect(unlisten).toHaveBeenCalled()
    const again = vi.fn()
    const stop2 = store.subscribe(again)
    expect(again).toHaveBeenLastCalledWith(undefined)
    stop2()
  })
})
