import { describe, expect, it } from 'vitest'
import type { Deck, Page, ValidationError } from '../lib/api'
import { Draft, firstFreeCell } from './draft.svelte'

const launch = { type: 'launch', kind: 'steam', appId: 1 } as const

function page(widgets: Page['widgets'], columns = 3, rows = 2): Page {
  return { id: 'home', title: 'Home', grid: { columns, rows }, widgets }
}

function deckOf(p: Page): Deck {
  return { version: 1, name: 'T', sources: {}, pages: [p] }
}

const ok = async () => [] as ValidationError[]

describe('firstFreeCell', () => {
  it('returns the first cell of an empty page', () => {
    expect(firstFreeCell(page([]))).toEqual({ x: 0, y: 0 })
  })

  it('goes row by row, left to right', () => {
    const w = (id: string, x: number, y: number) => ({
      id,
      type: 'launch' as const,
      action: launch,
      position: { x, y },
    })
    expect(firstFreeCell(page([w('a', 0, 0), w('b', 1, 0)]))).toEqual({ x: 2, y: 0 })
    expect(firstFreeCell(page([w('a', 0, 0), w('b', 1, 0), w('c', 2, 0)]))).toEqual({ x: 0, y: 1 })
  })

  it('skips cells covered by a wide or tall widget', () => {
    const big = { id: 'big', type: 'volume' as const, position: { x: 0, y: 0, w: 2, h: 2 } }
    expect(firstFreeCell(page([big]))).toEqual({ x: 2, y: 0 })
  })

  it('returns null when the page is full', () => {
    const big = { id: 'big', type: 'volume' as const, position: { x: 0, y: 0, w: 3, h: 2 } }
    expect(firstFreeCell(page([big]))).toBeNull()
  })
})

describe('Draft', () => {
  it('starts clean and does not touch the deck it was given', () => {
    const deck = deckOf(page([]))
    const d = new Draft(deck, ok)
    expect(d.dirty).toBe(false)
    d.addWidget('home', 'counter')
    expect(deck.pages[0].widgets).toHaveLength(0)
  })

  it('adds a widget in the first free cell, selects it and marks the draft dirty', () => {
    const d = new Draft(
      deckOf(page([{ id: 'a', type: 'launch', action: launch, position: { x: 0, y: 0 } }])),
      ok,
    )
    const id = d.addWidget('home', 'counter')
    expect(id).toBe('counter-1')
    expect(d.deck.pages[0].widgets[1].position).toEqual({ x: 1, y: 0 })
    expect(d.selectedId).toBe('counter-1')
    expect(d.dirty).toBe(true)
  })

  it('gives each added widget a unique id', () => {
    const d = new Draft(deckOf(page([])), ok)
    expect(d.addWidget('home', 'counter')).toBe('counter-1')
    expect(d.addWidget('home', 'counter')).toBe('counter-2')
  })

  it('refuses to add to a full page and says so', () => {
    const d = new Draft(deckOf(page([], 1, 1)), ok)
    expect(d.addWidget('home', 'volume')).toBe('volume-1')
    expect(d.addWidget('home', 'volume')).toBeNull()
    expect(d.notice).toContain('full')
    expect(d.deck.pages[0].widgets).toHaveLength(1)
  })

  it('returns null for an unknown page', () => {
    expect(new Draft(deckOf(page([])), ok).addWidget('nope', 'text')).toBeNull()
  })

  it('discard returns to the starting deck', () => {
    const d = new Draft(deckOf(page([])), ok)
    d.addWidget('home', 'text')
    d.discard()
    expect(d.dirty).toBe(false)
    expect(d.deck.pages[0].widgets).toHaveLength(0)
    expect(d.selectedId).toBeNull()
  })

  it('markSaved makes the current deck the clean state', () => {
    const d = new Draft(deckOf(page([])), ok)
    d.addWidget('home', 'text')
    d.markSaved()
    expect(d.dirty).toBe(false)
  })

  it('validates after every change and cannot save an invalid draft', async () => {
    const calls: Deck[] = []
    const bad: ValidationError = {
      path: '/pages/0/widgets/0/action',
      rule: 'required',
      message: 'action is required',
      fix: 'set it',
    }
    const validate = async (deck: Deck) => {
      calls.push(deck)
      return deck.pages[0].widgets.some((w) => w.type === 'launch') ? [bad] : []
    }
    const d = new Draft(deckOf(page([])), validate)
    await d.revalidate()
    expect(d.errors).toEqual([])

    d.addWidget('home', 'launch')
    await d.revalidate()
    expect(d.errors).toEqual([bad])
    expect(d.dirty).toBe(true)
    expect(d.canSave).toBe(false)
    expect(calls.length).toBeGreaterThanOrEqual(3)
  })

  it('can save a valid draft with changes, and not a clean one', async () => {
    const d = new Draft(deckOf(page([])), ok)
    await d.revalidate()
    expect(d.canSave).toBe(false)
    d.addWidget('home', 'counter')
    await d.revalidate()
    expect(d.canSave).toBe(true)
  })

  it('keeps only the newest validation result', async () => {
    const resolvers: ((e: ValidationError[]) => void)[] = []
    const validate = () => new Promise<ValidationError[]>((r) => resolvers.push(r))
    const d = new Draft(deckOf(page([])), validate)
    const second = d.revalidate()
    resolvers[1]([])
    await second
    resolvers[0]([{ path: '', rule: 'x', message: 'stale', fix: '' }])
    await Promise.resolve()
    expect(d.errors).toEqual([])
  })

  it('reports a validator failure as an error, not as valid', async () => {
    const d = new Draft(deckOf(page([])), async () => {
      throw new Error('ipc down')
    })
    await d.revalidate()
    expect(d.errors[0].message).toContain('ipc down')
    expect(d.canSave).toBe(false)
  })
})
