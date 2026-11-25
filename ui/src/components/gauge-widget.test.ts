import { render, screen, cleanup } from '@testing-library/svelte'
import { tick } from 'svelte'
import { afterEach, describe, expect, it, vi } from 'vitest'

type Handler = (v: Record<string, number | null>) => void
const handlers: Handler[] = []
const unlisten = vi.fn()
vi.mock('../lib/sources', () => ({
  onSourceValue: vi.fn(async (_id: string, h: Handler) => {
    handlers.push(h)
    return unlisten
  }),
}))

import GaugeWidget from './GaugeWidget.svelte'
import StatusWidget from './StatusWidget.svelte'

const pos = { x: 0, y: 0 }
afterEach(() => {
  cleanup()
  handlers.length = 0
  unlisten.mockClear()
})

describe('GaugeWidget', () => {
  const widget = { id: 'cpu', type: 'gauge', source: 'stats-a', metric: 'cpu.load', unit: '%', position: pos } as const

  it('shows no data until a reading arrives, then the value, then no data for null', async () => {
    render(GaugeWidget, { widget })
    await tick()
    expect(screen.getByTestId('gauge').dataset.state).toBe('nodata')
    expect(screen.getByTestId('gauge-value').textContent).toContain('no data')
    handlers[0]({ 'cpu.load': 42 })
    await tick()
    expect(screen.getByTestId('gauge').dataset.state).toBe('value')
    expect(screen.getByTestId('gauge-value').textContent).toContain('42')
    handlers[0]({ 'cpu.load': null })
    await tick()
    expect(screen.getByTestId('gauge').dataset.state).toBe('nodata')
  })

  it('stops listening on unmount', async () => {
    render(GaugeWidget, { widget: { ...widget, source: 'stats-b' } })
    await tick()
    await Promise.resolve()
    cleanup()
    await Promise.resolve()
    expect(unlisten).toHaveBeenCalled()
  })
})

describe('StatusWidget', () => {
  it('shows OK, problem and no data', async () => {
    render(StatusWidget, {
      widget: { id: 's', type: 'status', source: 'net', path: 'up', okWhen: '1', position: pos },
    })
    await tick()
    expect(screen.getByTestId('status').dataset.state).toBe('nodata')
    handlers[0]({ up: 1 })
    await tick()
    expect(screen.getByTestId('status').dataset.state).toBe('ok')
    handlers[0]({ up: 0 })
    await tick()
    expect(screen.getByTestId('status').dataset.state).toBe('problem')
  })
})
