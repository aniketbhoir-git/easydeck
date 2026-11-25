import { invoke } from '@tauri-apps/api/core'

// The deck types are generated from the Rust model (crates/easydeck-core/src/deck/model.rs).
// Do not edit ui/src/lib/generated/; run `cargo run -p easydeck-cli -- gen` instead.
export type { Action } from './generated/Action'
export type { Deck } from './generated/Deck'
export type { Grid } from './generated/Grid'
export type { LaunchKind } from './generated/LaunchKind'
export type { Page } from './generated/Page'
export type { Position } from './generated/Position'
export type { Settings } from './generated/Settings'
export type { Widget } from './generated/Widget'
export type { RiskReview } from './generated/RiskReview'
export type { ReviewItem } from './generated/ReviewItem'
export type { RiskItem } from './generated/RiskItem'
import type { Deck } from './generated/Deck'
import type { Page } from './generated/Page'
import type { Position } from './generated/Position'
import type { Widget } from './generated/Widget'
import type { RiskReview } from './generated/RiskReview'

export interface AppError {
  code: string
  message: string
  detail: string | null
  retryable: boolean
}

export function loadDeck(): Promise<Deck> {
  return invoke<Deck>('load_deck')
}

// `hold` true runs the widget's holdAction (long press); omitted runs its action (tap).
export function runAction(widgetId: string, hold = false): Promise<void> {
  return invoke<void>('run_action', hold ? { widgetId, hold: true } : { widgetId })
}

// A toggle tap sends the state it switches to. The backend picks onAction or offAction.
export function runToggle(widgetId: string, toggle: 'on' | 'off'): Promise<void> {
  return invoke<void>('run_action', { widgetId, toggle })
}

/** The risk items of the deck, each marked approved or not. `notice` comes once. */
export function riskReview(): Promise<RiskReview> {
  return invoke<RiskReview>('risk_review')
}

/** Approves the items with these hashes (only hashes of this deck); returns the new review. */
export function approveRiskItems(hashes: string[]): Promise<RiskReview> {
  return invoke<RiskReview>('approve_risk_items', { hashes })
}

/** A widget whose position is filled in. `load_deck` places every widget in Rust
 *  (easydeck-core `layout::resolve`), so the UI never decides placement itself. */
export type PlacedWidget = Widget & { position: Position }

export function placedWidgets(page: Page): PlacedWidget[] {
  const placed = page.widgets.filter((w): w is PlacedWidget => w.position !== undefined)
  if (placed.length < page.widgets.length) {
    const dropped = page.widgets.filter((w) => w.position === undefined).map((w) => w.id)
    console.warn(`Page "${page.id}": widgets without a position are not drawn: ${dropped.join(', ')}`)
  }
  return placed
}

/** One problem the core validator found in a deck. `path` is a JSON pointer into the deck. */
export interface ValidationError {
  path: string
  rule: string
  message: string
  fix: string
}

/** Validates a draft deck with the core rules. An empty list means the deck is valid. */
export function validateDeck(deck: Deck): Promise<ValidationError[]> {
  return invoke<ValidationError[]>('validate_deck', { deck })
}
