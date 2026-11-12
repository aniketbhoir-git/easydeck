import { invoke } from '@tauri-apps/api/core'

// Mirrors the parts of the Rust deck model (src-tauri/src/deck/model.rs) that the UI draws.
export interface Grid {
  columns: number
  rows: number
}

export interface Position {
  x: number
  y: number
  w?: number
  h?: number
}

export interface Widget {
  id: string
  type: string
  label?: string
  icon?: string
  position: Position
}

export interface Page {
  id: string
  title: string
  grid: Grid
  widgets: Widget[]
}

export interface Settings {
  homePage?: string
}

export interface Deck {
  version: number
  name: string
  settings?: Settings
  pages: Page[]
}

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
