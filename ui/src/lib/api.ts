import { invoke } from '@tauri-apps/api/core'

// The deck types are generated from the Rust model (crates/easydeck-core/src/deck/model.rs).
// Do not edit ui/src/lib/generated/; run `cargo run -p easydeck-cli -- gen` instead.
export type { Deck } from './generated/Deck'
export type { Grid } from './generated/Grid'
export type { Page } from './generated/Page'
export type { Position } from './generated/Position'
export type { Settings } from './generated/Settings'
export type { Widget } from './generated/Widget'
import type { Deck } from './generated/Deck'
import type { Page } from './generated/Page'
import type { Position } from './generated/Position'
import type { Widget } from './generated/Widget'

export interface AppError {
  code: string
  message: string
  detail: string | null
  retryable: boolean
}

export function loadDeck(): Promise<Deck> {
  return invoke<Deck>('load_deck')
}

export function runAction(widgetId: string): Promise<void> {
  return invoke<void>('run_action', { widgetId })
}

/** A widget whose position is filled in. `load_deck` places every widget in Rust
 *  (easydeck-core `layout::resolve`), so the UI never decides placement itself. */
export type PlacedWidget = Widget & { position: Position }

export function placedWidgets(page: Page): PlacedWidget[] {
  return page.widgets.filter((w): w is PlacedWidget => w.position !== undefined)
}
