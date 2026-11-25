import { readable, type Readable } from 'svelte/store'
import { onSourceValue, type SourceValue } from './sources'

type Subscribe = (id: string, handler: (v: SourceValue) => void) => Promise<() => void>

/**
 * A store of the latest reading per source id. Widgets share one event listener per id; the
 * listener starts with the first subscriber and stops with the last. `undefined` means no
 * reading has arrived yet.
 */
export function createSourceStore(subscribe: Subscribe = onSourceValue) {
  const stores = new Map<string, Readable<SourceValue | undefined>>()
  return (id: string): Readable<SourceValue | undefined> => {
    let store = stores.get(id)
    if (!store) {
      store = readable<SourceValue | undefined>(undefined, (set) => {
        let stopped = false
        let unlisten: (() => void) | undefined
        subscribe(id, (v) => set(v)).then(
          (fn) => (stopped ? fn() : (unlisten = fn)),
          () => {},
        )
        return () => {
          stopped = true
          unlisten?.()
        }
      })
      stores.set(id, store)
    }
    return store
  }
}

export const sourceStore = createSourceStore()
