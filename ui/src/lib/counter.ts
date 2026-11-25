// Pure counter logic. No DOM, no storage: see widget-state.ts for persistence.

export interface CounterConfig {
  start?: number
  step?: number
  min?: number
  max?: number
}

export const counterStart = (c: CounterConfig): number => c.start ?? 0
export const counterStep = (c: CounterConfig): number => c.step ?? 1

export function clamp(value: number, min?: number, max?: number): number {
  if (min !== undefined && value < min) return min
  if (max !== undefined && value > max) return max
  return value
}

/** The value after one tap: adds `step`, then clamps to min and max. */
export function nextValue(value: number, c: CounterConfig): number {
  const sum = value + counterStep(c)
  const safe = Math.max(-Number.MAX_SAFE_INTEGER, Math.min(Number.MAX_SAFE_INTEGER, sum))
  return clamp(safe, c.min, c.max)
}

/** The value after a long press. */
export function resetValue(c: CounterConfig): number {
  return counterStart(c)
}

/** Reads a stored string. Missing, non-integer or unsafe values give `start`; the result
 *  is clamped, so a changed min or max in the deck takes effect. */
export function parseStored(raw: string | null | undefined, c: CounterConfig): number {
  if (raw === null || raw === undefined || !/^-?\d+$/.test(raw.trim())) return counterStart(c)
  const n = Number(raw)
  if (!Number.isSafeInteger(n)) return counterStart(c)
  return clamp(n, c.min, c.max)
}

/** localStorage key for one widget of one deck. */
export function counterKey(deckName: string, widgetId: string): string {
  return `easydeck:widget:${deckName}:${widgetId}`
}
