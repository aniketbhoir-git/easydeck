import { afterEach, describe, expect, it, vi } from 'vitest'
import { placedWidgets, type Page } from './api'

const widget = (id: string, position?: { x: number; y: number }) =>
  ({ id, type: 'text', text: id, ...(position ? { position } : {}) }) as Page['widgets'][number]

const page = (widgets: Page['widgets']): Page => ({
  id: 'home',
  title: 'Home',
  grid: { columns: 2, rows: 2 },
  widgets,
})

afterEach(() => vi.restoreAllMocks())

describe('placedWidgets', () => {
  it('returns widgets with a position and does not warn', () => {
    const warn = vi.spyOn(console, 'warn').mockImplementation(() => {})
    const out = placedWidgets(page([widget('a', { x: 0, y: 0 }), widget('b', { x: 1, y: 0 })]))
    expect(out.map((w) => w.id)).toEqual(['a', 'b'])
    expect(warn).not.toHaveBeenCalled()
  })

  it('logs the ids of dropped widgets', () => {
    const warn = vi.spyOn(console, 'warn').mockImplementation(() => {})
    const out = placedWidgets(page([widget('a', { x: 0, y: 0 }), widget('b'), widget('c')]))
    expect(out.map((w) => w.id)).toEqual(['a'])
    expect(warn).toHaveBeenCalledTimes(1)
    expect(String(warn.mock.calls[0][0])).toContain('b, c')
  })
})
