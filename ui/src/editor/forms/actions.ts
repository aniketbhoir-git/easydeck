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

const MODIFIER_KEYS = new Set(['Control', 'Shift', 'Alt', 'Meta', 'AltGraph'])
const KEY_NAMES: Record<string, string> = {
  ' ': 'space',
  ArrowUp: 'up',
  ArrowDown: 'down',
  ArrowLeft: 'left',
  ArrowRight: 'right',
  Escape: 'esc',
}

/** The key name a recorded key press gives, for example "ctrl+q", or null for a bare
 *  modifier press (the next key completes the combination). */
export function keyFromEvent(e: {
  key: string
  ctrlKey: boolean
  shiftKey: boolean
  altKey: boolean
  metaKey: boolean
}): string | null {
  if (MODIFIER_KEYS.has(e.key)) return null
  const name = KEY_NAMES[e.key] ?? e.key.toLowerCase()
  const mods = [e.ctrlKey && 'ctrl', e.altKey && 'alt', e.shiftKey && 'shift', e.metaKey && 'win']
  return [...mods.filter(Boolean), name].join('+')
}

/** Errors for one action, by field. `actionPath` is the JSON pointer of the action, for
 *  example `/pages/0/widgets/1/action`. The core validator reports a missing launch field at
 *  the widget path (rule `launch-required-field`) and names the field in the message, so that
 *  error goes to the field the action lacks. Anything else at the action path is under `_`. */
export function fieldErrors(
  errors: ValidationError[],
  widgetPath: string,
  actionPath: string,
  action: Action | undefined,
): Record<string, ValidationError[]> {
  const out: Record<string, ValidationError[]> = {}
  const add = (key: string, e: ValidationError) => (out[key] ??= []).push(e)
  for (const e of errors) {
    if (e.path === actionPath || e.path.startsWith(actionPath + '/')) {
      const rest = e.path.slice(actionPath.length + 1).split('/')[0]
      add(rest === '' ? '_' : rest, e)
    } else if (
      e.path === widgetPath &&
      e.rule === 'launch-required-field' &&
      action?.type === 'launch'
    ) {
      const field = requiredField(action.kind)
      const missing = action[field] === undefined
      if (missing && e.message.includes(`"${field}"`)) add(field, e)
    }
  }
  return out
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
