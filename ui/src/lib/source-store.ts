import { readable, type Readable } from 'svelte/store'
import { onSourceValue, type SourceValue } from './sources'

type Subscribe = (id: string, handler: (v: SourceValue) => void) => Promise<() => void>

/** The silence, in poll intervals, after which a reading counts as stale. */
export const STALE_AFTER_INTERVALS = 3
const DEFAULT_INTERVAL_MS = 1000
const MIN_INTERVAL_MS = 250

/**
 * A store of the latest reading per source id. Widgets share one event listener per id; the
 * listener starts with the first subscriber and stops with the last. `undefined` means no
 * data: no reading has arrived yet, the store stopped, or the source has been silent for
 * three poll intervals (the backend sends nothing when a source stops or reloads).
 */
export function createSourceStore(subscribe: Subscribe = onSourceValue) {
  const stores = new Map<string, Readable<SourceValue | undefined>>()
  return (id: string, intervalMs?: number): Readable<SourceValue | undefined> => {
    const interval = Math.max(MIN_INTERVAL_MS, intervalMs ?? DEFAULT_INTERVAL_MS)
    const key = `${id}\u0000${interval}`
    let store = stores.get(key)
    if (!store) {
      store = readable<SourceValue | undefined>(undefined, (set) => {
        let stopped = false
        let unlisten: (() => void) | undefined
        let timer: ReturnType<typeof setTimeout> | undefined
        const arm = () => {
          clearTimeout(timer)
          timer = setTimeout(() => set(undefined), interval * STALE_AFTER_INTERVALS)
        }
        subscribe(id, (v) => {
          if (stopped) return
          set(v)
          arm()
        }).then(
          (fn) => (stopped ? fn() : (unlisten = fn)),
          () => {},
        )
        return () => {
          stopped = true
          clearTimeout(timer)
          unlisten?.()
          set(undefined)
        }
      })
      stores.set(key, store)
    }
    return store
  }
}

export const sourceStore = createSourceStore()
