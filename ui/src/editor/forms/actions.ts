// Pure helpers behind the launch and macro forms: how a form value becomes an `Action`, which
// validator errors belong to which field, and the numeric limits. No Svelte, no IPC, so
// vitest covers it. Nothing here runs an action: the forms only edit the draft.
import schema from '../../../../schema/deck.schema.json'
import type { Action, LaunchKind, ValidationError } from '../../lib/api'

export type LaunchAction = Extract<Action, { type: 'launch' }>
export type MacroAction = Extract<Action, { type: 'key_macro' }>

export const LAUNCH_KINDS: { kind: LaunchKind; name: string; field: 'appId' | 'path' | 'target' }[] = [
  { kind: 'steam', name: 'Steam game', field: 'appId' },
  { kind: 'exe', name: 'Program (exe)', field: 'path' },
  { kind: 'uri', name: 'URI', field: 'target' },
  { kind: 'url', name: 'Web link', field: 'target' },
]

/** The one field each launch kind needs, and the only others it may carry. */
export function requiredField(kind: LaunchKind): 'appId' | 'path' | 'target' {
  return LAUNCH_KINDS.find((k) => k.kind === kind)!.field
}

type Limit = { min: number; max: number }
type SchemaProps = Record<string, { minimum?: number }>

function macroProps(): SchemaProps {
  const defs = (schema as unknown as { $defs: { Action: { oneOf: { properties: SchemaProps }[] } } }).$defs
  return defs.Action.oneOf.find((o) => 'keys' in o.properties)!.properties
}
function launchProps(): SchemaProps {
  const defs = (schema as unknown as { $defs: { Action: { oneOf: { properties: SchemaProps }[] } } }).$defs
  return defs.Action.oneOf.find((o) => 'appId' in o.properties)!.properties
}

// The schema gives the lower bound; the upper bound is the largest whole number a JSON number
// in the webview holds exactly.
const MAX = Number.MAX_SAFE_INTEGER
const limit = (props: SchemaProps, name: string): Limit => ({ min: props[name].minimum ?? 0, max: MAX })

/** Numeric limits, read from the generated schema (`schema/deck.schema.json`). */
export const LIMITS = {
  appId: limit(launchProps(), 'appId'),
  cycleMs: limit(macroProps(), 'cycleMs'),
  keyGapMs: limit(macroProps(), 'keyGapMs'),
  holdMs: limit(macroProps(), 'holdMs'),
}

/** A new launch action of `kind`, with the field it needs still empty. */
export function newLaunch(kind: LaunchKind = 'steam'): LaunchAction {
  return { type: 'launch', kind }
}

export function newMacro(): MacroAction {
  return { type: 'key_macro', keys: [] }
}

/** Switches the launch kind. The field of the old kind is dropped, so the action never keeps
 *  an `appId` on an `exe` launch or a `path` on a `url` one. `args` and `fallback` belong to
 *  `exe` only, so they go too. Switching between `uri` and `url` keeps `target`. */
export function setLaunchKind(a: LaunchAction, kind: LaunchKind): LaunchAction {
  const next: LaunchAction = { type: 'launch', kind }
  const field = requiredField(kind)
  if (field === 'target' && a.target !== undefined) next.target = a.target
  return next
}

/** Sets or clears one field of a launch action. `undefined` and "" remove it. */
export function setLaunchField(
  a: LaunchAction,
  field: 'appId' | 'path' | 'target' | 'fallback',
  value: string | number | undefined,
): LaunchAction {
  const next = { ...a } as Record<string, unknown>
  if (value === undefined || value === '' || Number.isNaN(value)) delete next[field]
  else next[field] = value
  return next as LaunchAction
}

export function setMacroNumber(
  a: MacroAction,
  field: 'cycleMs' | 'keyGapMs' | 'holdMs',
  value: number | undefined,
): MacroAction {
  const next = { ...a }
  if (value === undefined || Number.isNaN(value)) delete next[field]
  else next[field] = value
  return next
}

export function setMacroToggle(a: MacroAction, toggle: boolean): MacroAction {
  const next = { ...a }
  if (toggle) next.toggle = true
  else delete next.toggle
  return next
}

/** Splits typed key text on spaces and commas: "1 2 ctrl+q" gives ["1","2","ctrl+q"]. */
export function parseKeys(text: string): string[] {
  return text.split(/[\s,]+/).filter((k) => k !== '')
}

// Recorded key names come from the physical key (`KeyboardEvent.code`), never from `e.key`,
// which depends on the layout and on Shift ("Shift+1" gives "!"). The model takes free-text
// names (`"1"`, `"ctrl+q"`, see `KeyMacro.keys`); this is the lower-case set the forms write:
// a-z, 0-9, f1-f24, named keys, and the punctuation keys. A comma is left out, because the
// typed text splits on it. Modifiers join with `+`, in the order ctrl, alt, shift, win.
const KEY_BY_CODE: Record<string, string> = {
  Space: 'space',
  Enter: 'enter',
  Tab: 'tab',
  Backspace: 'backspace',
  Delete: 'delete',
  Insert: 'insert',
  Home: 'home',
  End: 'end',
  PageUp: 'pageup',
  PageDown: 'pagedown',
  ArrowUp: 'up',
  ArrowDown: 'down',
  ArrowLeft: 'left',
  ArrowRight: 'right',
  Minus: '-',
  Equal: '=',
  BracketLeft: '[',
  BracketRight: ']',
  Semicolon: ';',
  Quote: "'",
  Backquote: '`',
  Backslash: '\\',
  Period: '.',
  Slash: '/',
}
for (let i = 0; i < 26; i++) KEY_BY_CODE[`Key${String.fromCharCode(65 + i)}`] = String.fromCharCode(97 + i)
for (let i = 0; i < 10; i++) KEY_BY_CODE[`Digit${i}`] = String(i)
for (let i = 1; i <= 24; i++) KEY_BY_CODE[`F${i}`] = `f${i}`

/** The key name a recorded key press gives, for example "ctrl+q" or "shift+=". It is null for a
 *  bare modifier press and for any key outside the allow-list (Escape, Dead, Unidentified,
 *  the numpad, media keys): nothing is recorded for those. */
export function keyFromEvent(e: {
  code: string
  ctrlKey: boolean
  shiftKey: boolean
  altKey: boolean
  metaKey: boolean
}): string | null {
  const name = Object.hasOwn(KEY_BY_CODE, e.code) ? KEY_BY_CODE[e.code] : undefined
  if (name === undefined) return null
  const mods = [e.ctrlKey && 'ctrl', e.altKey && 'alt', e.shiftKey && 'shift', e.metaKey && 'win']
  return [...mods.filter(Boolean), name].join('+')
}

/** Errors for one action, by field. `actionPath` is the JSON pointer of the action, for
 *  example `/pages/0/widgets/1/holdAction`. The core validator reports a missing launch field
 *  at `<actionPath>/<field>`, so each slot gets only its own errors. An error at the action
 *  path itself is under `_`. */
export function fieldErrors(errors: ValidationError[], actionPath: string): Record<string, ValidationError[]> {
  const out: Record<string, ValidationError[]> = {}
  for (const e of errors) {
    if (e.path === actionPath) (out._ ??= []).push(e)
    else if (e.path.startsWith(actionPath + '/')) {
      const field = e.path.slice(actionPath.length + 1).split('/')[0]
      ;(out[field] ??= []).push(e)
    }
  }
  return out
}

const MACRO_FIELDS = ['keys', 'cycleMs', 'keyGapMs', 'holdMs']

/** True when a form field of the tile shows this error already, so the inspector list leaves
 *  it out. `actions` maps each action slot of the widget to its action. */
export function shownByForm(
  e: ValidationError,
  prefix: string,
  actions: Record<string, Action | undefined>,
): boolean {
  for (const [slot, a] of Object.entries(actions)) {
    if (a === undefined) continue
    const path = `${prefix}/${slot}`
    if (e.path === path) return true
    if (!e.path.startsWith(path + '/')) continue
    const field = e.path.slice(path.length + 1).split('/')[0]
    if (a.type === 'launch') return field === requiredField(a.kind)
    if (a.type === 'key_macro') return MACRO_FIELDS.includes(field)
  }
  return false
}

/** Light hints the user sees before the validator answers. They never block anything. */
export function launchHint(a: LaunchAction): string | null {
  if (a.kind === 'url' && a.target && !/^https?:\/\//i.test(a.target)) return 'A web link starts with http:// or https://'
  if (a.kind === 'uri' && a.target && !a.target.includes(':')) return 'A URI has a scheme, for example steam://open/main'
  if (a.kind === 'exe' && a.path && /^\s|\s$/.test(a.path)) return 'The path has a space at an end'
  return null
}

export function macroHint(a: MacroAction): string | null {
  return a.keys.length === 0 ? 'Add at least one key' : null
}
