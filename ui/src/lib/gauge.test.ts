import { describe, expect, it } from 'vitest'
import { evaluateStatus, formatGaugeValue, gaugeAngle, gaugeFraction, metricValue } from './gauge'

describe('gauge mapping', () => {
  it('maps value to angle over a 270 degree sweep', () => {
    expect(gaugeAngle(0)).toBe(0)
    expect(gaugeAngle(50)).toBe(135)
    expect(gaugeAngle(100)).toBe(270)
  })
  it('clamps outside min..max', () => {
    expect(gaugeFraction(-5, 0, 100)).toBe(0)
    expect(gaugeFraction(250, 0, 100)).toBe(1)
  })
  it('uses custom min and max', () => {
    expect(gaugeFraction(60, 20, 100)).toBe(0.5)
  })
  it('handles min == max and min > max without NaN', () => {
    expect(gaugeFraction(5, 10, 10)).toBe(0)
    expect(gaugeFraction(5, 10, 1)).toBe(0)
    expect(gaugeFraction(Number.NaN)).toBe(0)
  })
  it('formats values', () => {
    expect(formatGaugeValue(42)).toBe('42')
    expect(formatGaugeValue(42.46)).toBe('42.5')
  })
})

describe('no data', () => {
  it('is null for a missing reading, a missing metric and a null metric', () => {
    expect(metricValue(undefined, 'cpu.load')).toBeNull()
    expect(metricValue({}, 'cpu.load')).toBeNull()
    expect(metricValue({ 'cpu.load': null }, 'cpu.load')).toBeNull()
    expect(metricValue({ 'cpu.load': 12 }, 'cpu.load')).toBe(12)
  })
})

describe('evaluateStatus', () => {
  it('is nodata without a usable reading', () => {
    expect(evaluateStatus(undefined, 'a', '1')).toBe('nodata')
    expect(evaluateStatus({ a: null }, 'a', '1')).toBe('nodata')
    expect(evaluateStatus({ a: 1 }, undefined, '1')).toBe('nodata')
  })
  it('compares numerically against okWhen', () => {
    expect(evaluateStatus({ a: 1 }, 'a', '1')).toBe('ok')
    expect(evaluateStatus({ a: 1 }, 'a', '1.0')).toBe('ok')
    expect(evaluateStatus({ a: 2 }, 'a', '1')).toBe('problem')
  })
  it('treats non-zero as OK without okWhen', () => {
    expect(evaluateStatus({ a: 3 }, 'a', undefined)).toBe('ok')
    expect(evaluateStatus({ a: 0 }, 'a', undefined)).toBe('problem')
  })
})
