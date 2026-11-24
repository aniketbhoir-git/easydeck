import { render, screen, cleanup } from '@testing-library/svelte'
import { tick } from 'svelte'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import TextWidget from './TextWidget.svelte'

const clock = {
  id: 'c',
  type: 'text',
  kind: 'clock',
  seconds: true,
  hour12: false,
  showDate: true,
  position: { x: 0, y: 0 },
} as const

beforeEach(() => {
  vi.useFakeTimers()
  vi.setSystemTime(new Date('2026-11-25T10:00:00.400Z'))
})
afterEach(() => {
  cleanup()
  vi.useRealTimers()
})

describe('TextWidget', () => {
  it('shows a label', () => {
    render(TextWidget, { widget: { id: 'l', type: 'text', text: 'Hello', position: { x: 0, y: 0 } } })
    expect(screen.getByText('Hello')).toBeTruthy()
    expect(vi.getTimerCount()).toBe(0)
  })

  it('ticks on the second boundary and shows the date line', async () => {
    render(TextWidget, { widget: clock })
    const before = screen.getByTestId('clock-time').textContent
    expect(screen.getByTestId('clock-date')).toBeTruthy()
    vi.advanceTimersByTime(1000)
    await tick()
    expect(screen.getByTestId('clock-time').textContent).not.toBe(before)
  })

  it('clears its timer on unmount', () => {
    const { unmount } = render(TextWidget, { widget: clock })
    expect(vi.getTimerCount()).toBe(1)
    unmount()
    expect(vi.getTimerCount()).toBe(0)
  })
})
