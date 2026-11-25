import { describe, expect, it } from 'vitest'
import type { RiskReview } from './generated/RiskReview'
import { lockedTiles, pendingCount, placeName, tileKey } from './risk'

const item = (approved: boolean, ...locations: string[]) => ({
  approved,
  item: {
    kind: 'launch' as const,
    hash: 'a'.repeat(64),
    summary: 's',
    systemShortcut: false,
    locations,
  },
})

describe('lockedTiles', () => {
  it('locks the tiles of unapproved items only', () => {
    const review: RiskReview = {
      notice: null,
      items: [
        item(true, 'pages/home/widgets/ok/action'),
        item(false, 'pages/home/widgets/evil/action', 'pages/two/widgets/evil/holdAction'),
        item(false, 'sources/api'),
      ],
    }
    expect([...lockedTiles(review)].sort()).toEqual([tileKey('home', 'evil'), tileKey('two', 'evil')])
    expect(pendingCount(review)).toBe(2)
  })

  it('locks nothing without a review', () => {
    expect(lockedTiles(null).size).toBe(0)
    expect(pendingCount(null)).toBe(0)
  })
})

describe('placeName', () => {
  it('names tiles and sources', () => {
    expect(placeName('pages/home/widgets/play/action')).toBe('home / play (action)')
    expect(placeName('sources/api')).toBe('source api')
    expect(placeName('x')).toBe('x')
  })
})
