import { describe, expect, it } from 'vitest'
import schema from '../../../../schema/deck.schema.json'
import type { Deck, ValidationError } from '../../lib/api'
import { Draft } from '../draft.svelte'
import {
  LIMITS,
  fieldErrors,
  keyFromEvent,
  newLaunch,
  newMacro,
  parseKeys,
  setLaunchField,
  setLaunchKind,
  setMacroNumber,
  setMacroToggle,
} from './actions'

const none = async () => [] as ValidationError[]
const deckWith = (widget: object): Deck =>
  ({
    version: 1,
    name: 'T',
    sources: {},
    pages: [{ id: 'home', title: 'Home', grid: { columns: 3, rows: 2 }, widgets: [widget] }],
  }) as unknown as Deck

describe('launch form mapping', () => {
  it('switching kind clears the other kind field', () => {
    let a = setLaunchField(newLaunch('steam'), 'appId', 440)
    a = setLaunchKind(a, 'exe')
    expect(a).toEqual({ type: 'launch', kind: 'exe' })
    a = setLaunchField(a, 'path', 'C:\g.exe')
    a = setLaunchField(a, 'fallback', 'D:\g.exe')
    a = setLaunchKind(a, 'url')
    expect(a).toEqual({ type: 'launch', kind: 'url' })
    a = setLaunchField(a, 'target', 'https://example.com')
    expect(setLaunchKind(a, 'uri')).toEqual({ type: 'launch', kind: 'uri', target: 'https://example.com' })
    expect(setLaunchKind(a, 'steam')).toEqual({ type: 'launch', kind: 'steam' })
  })

  it('an empty value removes the field', () => {
    const a = setLaunchField(setLaunchField(newLaunch('exe'), 'path', 'x'), 'path', '')
    expect('path' in a).toBe(false)
    expect('appId' in setLaunchField(newLaunch(), 'appId', NaN)).toBe(false)
  })
})

describe('macro form mapping', () => {
  it('parses typed keys', () => {
    expect(parseKeys(' 1  2,ctrl+q ')).toEqual(['1', '2', 'ctrl+q'])
    expect(parseKeys('')).toEqual([])
  })

  it('names recorded keys and skips bare modifiers', () => {
    const e = { key: 'q', ctrlKey: true, shiftKey: false, altKey: false, metaKey: false }
    expect(keyFromEvent(e)).toBe('ctrl+q')
    expect(keyFromEvent({ ...e, key: 'Control' })).toBeNull()
    expect(keyFromEvent({ ...e, key: ' ', ctrlKey: false })).toBe('space')
    expect(keyFromEvent({ ...e, key: 'F5', ctrlKey: false, altKey: true })).toBe('alt+f5')
  })

  it('sets and clears numbers and toggle', () => {
    let a = setMacroNumber({ ...newMacro(), keys: ['1'] }, 'cycleMs', 500)
    expect(a.cycleMs).toBe(500)
    a = setMacroToggle(a, true)
    expect(a.toggle).toBe(true)
    a = setMacroToggle(setMacroNumber(a, 'cycleMs', undefined), false)
    expect(a).toEqual({ type: 'key_macro', keys: ['1'] })
  })

  it('reads limits from the schema', () => {
    expect(LIMITS.appId.min).toBe(1)
    expect(LIMITS.cycleMs.min).toBe(0)
    expect(schema.$defs.Action.oneOf.length).toBeGreaterThan(0)
  })
})

describe('field errors', () => {
  const widget = '/pages/0/widgets/0'
  it('maps a missing launch field to that field, by the message', () => {
    const errors = [
      {
        path: widget,
        rule: 'launch-required-field',
        message: 'widget "a": a launch action of kind exe needs the field "path"',
        fix: 'add "path"',
      },
    ] as ValidationError[]
    const a = newLaunch('exe')
    expect(Object.keys(fieldErrors(errors, widget, `${widget}/action`, a))).toEqual(['path'])
    // another action of the widget that has its field is not blamed
    expect(fieldErrors(errors, widget, `${widget}/holdAction`, setLaunchField(a, 'path', 'x'))).toEqual({})
  })

  it('maps a path under the action to its field', () => {
    const errors = [{ path: `${widget}/action/keys`, rule: 'schema', message: 'm', fix: 'f' }] as ValidationError[]
    expect(Object.keys(fieldErrors(errors, widget, `${widget}/action`, newMacro()))).toEqual(['keys'])
  })
})

describe('forms write to the draft only', () => {
  it('patchSelected sets, removes and revalidates; the given deck stays unchanged', async () => {
    const deck = deckWith({ id: 'g', type: 'launch', position: { x: 0, y: 0 } })
    const seen: Deck[] = []
    const d = new Draft(deck, async (x) => (seen.push(structuredClone(x)), []))
    d.select('g', 'home')
    d.patchSelected({ action: setLaunchField(newLaunch('steam'), 'appId', 440), label: 'Game' })
    expect(d.dirty).toBe(true)
    expect(d.selected!.widget).toMatchObject({ label: 'Game', action: { kind: 'steam', appId: 440 } })
    d.patchSelected({ label: undefined })
    expect('label' in d.selected!.widget).toBe(false)
    expect(deck.pages[0].widgets[0]).not.toHaveProperty('action')
    await d.revalidate()
    expect(seen.length).toBeGreaterThanOrEqual(3)
  })

  it('a steam tile and a macro tile built by the forms pass the mock-free shape checks', () => {
    const steam = setLaunchField(newLaunch('steam'), 'appId', 440)
    const macro = { ...newMacro(), keys: parseKeys('1 2 ctrl+q') }
    const defs = schema.$defs.Action.oneOf as unknown as { properties: Record<string, { minimum?: number }>; required?: string[] }[]
    const launch = defs[0]
    const mac = defs[1]
    for (const [a, def] of [[steam, launch], [macro, mac]] as const) {
      for (const k of Object.keys(a)) expect(def.properties).toHaveProperty(k)
      for (const r of def.required ?? []) expect(a).toHaveProperty(r)
    }
    expect(steam.appId!).toBeGreaterThanOrEqual(LIMITS.appId.min)
  })
})
