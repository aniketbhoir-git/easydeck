import type { Deck, Page } from './api'

/** The page the deck shows for `id`: that page, else the home page, else the first page. */
export function resolvePage(deck: Deck, id: string | null): Page | undefined {
  return (
    deck.pages.find((p) => p.id === id) ??
    deck.pages.find((p) => p.id === deck.settings?.homePage) ??
    deck.pages[0]
  )
}
