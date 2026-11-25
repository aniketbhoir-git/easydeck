// Pure toggle logic. No DOM, no storage: see widget-state.ts for persistence.

export type ToggleState = 'on' | 'off'

export interface ToggleLookConfig {
  label?: string
  icon?: string
  color?: string
}

export interface ToggleConfig {
  label?: string
  icon?: string
  on?: ToggleLookConfig
  off?: ToggleLookConfig
}

export interface ToggleView {
  label?: string
  icon?: string
  /** A validated hex colour, or undefined for the default tile colour. */
  color?: string
}

/** The state after one tap. */
export const flip = (s: ToggleState): ToggleState => (s === 'on' ? 'off' : 'on')

/** `#rgb` or `#rrggbb` only. The colour goes into a style attribute, so nothing else passes. */
export const isHexColor = (c: string | undefined): c is string =>
  c !== undefined && /^#(?:[0-9a-fA-F]{3}|[0-9a-fA-F]{6})$/.test(c)

/** What the tile shows in a state. A missing look block or field falls back to the widget's
 *  own label and icon; a missing or invalid colour falls back to the default tile colour. */
export function lookFor(c: ToggleConfig, s: ToggleState): ToggleView {
  const look = s === 'on' ? c.on : c.off
  const color = look?.color
  return {
    label: look?.label ?? c.label,
    icon: look?.icon ?? c.icon,
    color: isHexColor(color) ? color : undefined,
  }
}

/** Reads a stored string. Anything but "on" is off: a toggle starts off. */
export function parseStored(raw: string | null | undefined): ToggleState {
  return raw === 'on' ? 'on' : 'off'
}

/** localStorage key for one widget of one deck (same scheme as the counter). */
export function toggleKey(deckName: string, widgetId: string): string {
  return `easydeck:widget:${deckName}:${widgetId}`
}
