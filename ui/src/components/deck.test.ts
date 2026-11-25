import { render, screen, fireEvent, cleanup } from '@testing-library/svelte'
import { afterEach, describe, expect, it, vi } from 'vitest'

const invoke = vi.fn()
vi.mock('@tauri-apps/api/core', () => ({ invoke: (...a: unknown[]) => invoke(...a) }))

import Deck from './Deck.svelte'
import type { Deck as DeckModel } from '../lib/api'

const launch = { type: 'launch', kind: 'steam', appId: 1 } as const

const deck: DeckModel = {
  version: 1,
  name: 'Test',
  sources: {},
  settings: { homePage: 'home' },
  pages: [
    {
      id: 'home',
      title: 'Home',
      grid: { columns: 4, rows: 2 },
      widgets: [
        { id: 'a', type: 'launch', action: launch, label: 'Alpha', icon: 'A', position: { x: 0, y: 1 } },
        { id: 'b', type: 'launch', action: launch, label: 'Beta', position: { x: 2, y: 0, w: 2, h: 2 } },
      ],
    },
    {
      id: 'other',
      title: 'Other',
      grid: { columns: 1, rows: 1 },
      widgets: [{ id: 'c', type: 'launch', action: launch, label: 'Gamma', position: { x: 0, y: 0 } }],
    },
  ],
}

const holdDeck: DeckModel = {
  ...deck,
  pages: [
    {
      id: 'home',
      title: 'Home',
      grid: { columns: 1, rows: 1 },
      widgets: [
        { id: 'h', type: 'launch', action: launch, holdAction: launch, label: 'Hold', position: { x: 0, y: 0 } },
      ],
    },
  ],
}

afterEach(() => {
  cleanup()
  invoke.mockReset()
})

describe('Deck', () => {
  it('sets the CSS grid from page.grid and places widgets by position', () => {
    const { container } = render(Deck, { deck })
    const page = container.querySelector('.page') as HTMLElement
    // jsdom may normalise `0` to `0px`
    expect(page.style.gridTemplateColumns).toMatch(/^repeat\(4, minmax\(0(px)?, 1fr\)\)$/)
    expect(page.style.gridTemplateRows).toMatch(/^repeat\(2, minmax\(0(px)?, 1fr\)\)$/)

    const a = container.querySelector('[data-widget-id="a"]') as HTMLElement
    expect(a.style.gridColumn).toBe('1 / span 1')
    expect(a.style.gridRow).toBe('2 / span 1')
    const b = container.querySelector('[data-widget-id="b"]') as HTMLElement
    expect(b.style.gridColumn).toBe('3 / span 2')
    expect(b.style.gridRow).toBe('1 / span 2')
  })

  it('shows label and icon', () => {
    render(Deck, { deck })
    expect(screen.getByText('Alpha')).toBeTruthy()
    expect(screen.getByText('A')).toBeTruthy()
  })

  it('calls run_action with the widget id on tap', async () => {
    invoke.mockResolvedValue(undefined)
    render(Deck, { deck })
    await fireEvent.click(screen.getByText('Beta'))
    expect(invoke).toHaveBeenCalledWith('run_action', { widgetId: 'b' })
  })

  it('a locked tile opens the review and calls nothing', async () => {
    const onreview = vi.fn()
    render(Deck, { deck, locked: new Set(['home/b']), onreview })
    await fireEvent.click(screen.getByText('Beta'))
    expect(onreview).toHaveBeenCalledTimes(1)
    expect(invoke).not.toHaveBeenCalled()
    await fireEvent.click(screen.getByText('Alpha'))
    expect(invoke).toHaveBeenCalledWith('run_action', { widgetId: 'a' })
  })

  it('opens the review when the backend answers needs_approval', async () => {
    const onreview = vi.fn()
    const err = { code: 'needs_approval', message: 'x', detail: null, retryable: false }
    invoke.mockRejectedValue(err)
    const spy = vi.spyOn(console, 'error').mockImplementation(() => {})
    render(Deck, { deck, onreview })
    await fireEvent.click(screen.getByText('Beta'))
    await vi.waitFor(() => expect(onreview).toHaveBeenCalledTimes(1))
    spy.mockRestore()
  })

  it('calls run_action with hold on a long press of a tile with a holdAction', async () => {
    vi.useFakeTimers()
    try {
      invoke.mockResolvedValue(undefined)
      render(Deck, { deck: holdDeck })
      const t = screen.getByText('Hold').closest('button') as HTMLElement
      await fireEvent.pointerDown(t, { pointerId: 1 })
      vi.advanceTimersByTime(700)
      await fireEvent.pointerUp(t, { pointerId: 1 })
      expect(invoke).toHaveBeenCalledTimes(1)
      expect(invoke).toHaveBeenCalledWith('run_action', { widgetId: 'h', hold: true })
    } finally {
      vi.useRealTimers()
    }
  })

  it('switches pages', async () => {
    render(Deck, { deck })
    expect(screen.queryByText('Gamma')).toBeNull()
    await fireEvent.click(screen.getByRole('button', { name: 'Other' }))
    expect(screen.getByText('Gamma')).toBeTruthy()
  })

  it('marks a tile pressed on pointerdown and blocks the context menu', async () => {
    const { container } = render(Deck, { deck })
    const a = container.querySelector('[data-widget-id="a"]') as HTMLElement
    await fireEvent.pointerDown(a)
    expect(a.classList.contains('pressed')).toBe(true)
    await fireEvent.pointerUp(a)
    expect(a.classList.contains('pressed')).toBe(false)
    const ev = new MouseEvent('contextmenu', { bubbles: true, cancelable: true })
    a.dispatchEvent(ev)
    expect(ev.defaultPrevented).toBe(true)
  })
})
