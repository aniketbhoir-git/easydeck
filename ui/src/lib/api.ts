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
