import type { RiskReview } from './generated/RiskReview'

/** Key of a tile in the sets below: `<pageId>/<widgetId>`. */
export const tileKey = (pageId: string, widgetId: string) => `${pageId}/${widgetId}`

/**
 * The tiles that hold an unapproved risk item (SPEC §4.3). Such a tile is inert: it shows a
 * lock badge and a tap opens the review screen. An item's `locations` are id paths like
 * `pages/home/widgets/play/action`; an item in `sources/…` belongs to no tile.
 * The backend decides what runs; this only draws the lock.
 */
export function lockedTiles(review: RiskReview | null): Set<string> {
  const out = new Set<string>()
  for (const { item, approved } of review?.items ?? []) {
    if (approved) continue
    for (const loc of item.locations) {
      const m = /^pages\/([^/]+)\/widgets\/([^/]+)\//.exec(loc)
      if (m) out.add(tileKey(m[1], m[2]))
    }
  }
  return out
}

/** Number of items that wait for approval. */
export function pendingCount(review: RiskReview | null): number {
  return review?.items.filter((r) => !r.approved).length ?? 0
}

/** A short place name for an item location, for the review list. */
export function placeName(loc: string): string {
  const w = /^pages\/([^/]+)\/widgets\/([^/]+)\/(.+)$/.exec(loc)
  if (w) return `${w[1]} / ${w[2]} (${w[3]})`
  const s = /^sources\/(.+)$/.exec(loc)
  return s ? `source ${s[1]}` : loc
}
