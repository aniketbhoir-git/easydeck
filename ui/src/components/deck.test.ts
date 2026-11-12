import { render, screen, fireEvent, cleanup } from '@testing-library/svelte'
import { afterEach, describe, expect, it, vi } from 'vitest'

const invoke = vi.fn()
vi.mock('@tauri-apps/api/core', () => ({ invoke: (...a: unknown[]) => invoke(...a) }))

import Deck from './Deck.svelte'
import type { Deck as DeckModel } from '../lib/api'

const deck: DeckModel = {
  version: 1,
  name: 'Test',
  settings: { homePage: 'home' },
  pages: [
    {
      id: 'home',
      title: 'Home',
      grid: { columns: 4, rows: 2 },
      widgets: [
        { id: 'a', type: 'launch', label: 'Alpha', icon: 'A', position: { x: 0, y: 1 } },
        { id: 'b', type: 'launch', label: 'Beta', position: { x: 2, y: 0, w: 2, h: 2 } },
      ],
    },
    {
      id: 'other',
      title: 'Other',
      grid: { columns: 1, rows: 1 },
      widgets: [{ id: 'c', type: 'launch', label: 'Gamma', position: { x: 0, y: 0 } }],
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
