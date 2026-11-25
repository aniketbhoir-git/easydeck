import { getContext, setContext } from 'svelte'
import type { Deck } from './api'

const KEY = Symbol('deck-sources')
type Sources = Deck['sources']

export function provideSources(sources: Sources) {
  setContext(KEY, sources)
}

/** Poll interval in ms of a source in the deck; undefined when unknown (the store then assumes 1000). */
export function sourceIntervalMs(id: string): number | undefined {
  const s = getContext<Sources | undefined>(KEY)?.[id]
  if (!s) return undefined
  if (s.kind === 'http_poll') return (s.intervalSeconds ?? 60) * 1000
  return s.intervalMs ?? (s.kind === 'game_watch' ? 2000 : 1000)
}
