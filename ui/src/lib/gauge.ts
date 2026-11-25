/** Sweep of the gauge arc in degrees (a 270 degree dial with the gap at the bottom). */
export const GAUGE_SWEEP_DEG = 270

/** Maps `value` to 0..1 between `min` and `max`. Clamps; returns 0 when min >= max. */
export function gaugeFraction(value: number, min = 0, max = 100): number {
  if (!Number.isFinite(value) || !(max > min)) return 0
  return Math.min(1, Math.max(0, (value - min) / (max - min)))
}

/** Angle of the value arc in degrees, 0 at empty to GAUGE_SWEEP_DEG at full. */
export function gaugeAngle(value: number, min = 0, max = 100): number {
  return gaugeFraction(value, min, max) * GAUGE_SWEEP_DEG
}

/** The number to show: whole numbers as they are, others with one decimal. */
export function formatGaugeValue(value: number): string {
  return Number.isInteger(value) ? String(value) : value.toFixed(1)
}

/** Reads a metric from a source reading. Missing or `null` means no data. */
export function metricValue(
  reading: Record<string, number | null> | undefined,
  metric: string,
): number | null {
  const v = reading?.[metric]
  return typeof v === 'number' && Number.isFinite(v) ? v : null
}

export type StatusState = 'ok' | 'problem' | 'nodata'

/**
 * Status of a value. No reading, or a `null` one, is `nodata`. With `okWhen`, the value is OK
 * when it equals `okWhen` (compared as numbers when `okWhen` is numeric, so `1` matches `1.0`).
 * Without `okWhen`, any non-zero value is OK.
 */
export function evaluateStatus(
  reading: Record<string, number | null> | undefined,
  path: string | undefined,
  okWhen: string | undefined,
): StatusState {
  if (!reading || path === undefined) return 'nodata'
  const v = metricValue(reading, path)
  if (v === null) return 'nodata'
  if (okWhen === undefined) return v !== 0 ? 'ok' : 'problem'
  const expected = Number(okWhen)
  const ok = okWhen.trim() !== '' && Number.isFinite(expected) ? v === expected : String(v) === okWhen
  return ok ? 'ok' : 'problem'
}
