// Test-only IPC layer. main.ts imports this module only when the build mode is "e2e"
// (`vite build --mode e2e`), so it is never part of the production bundle.
import { mockIPC } from '@tauri-apps/api/mocks'
import { emit } from '@tauri-apps/api/event'
import type { Deck } from './api'

declare global {
  interface Window {
    __ipcCalls: { cmd: string; args: unknown }[]
    /** Test helper: pushes a source reading, as the app does for source events. */
    __emitSource: (id: string, payload: Record<string, number | null>) => Promise<void>
    /** Test helper: pushes a volume event, as the app does when Windows changes the volume. */
    __emitVolume: (payload: { level: number; muted: boolean }) => Promise<void>
  }
}

// Mirrors the home page and the in-game page of examples/starter.deck.json,
// reduced to the fields the UI model draws.
export const fixtureDeck: Deck = {
  version: 1,
  name: 'Starter deck',
  settings: { homePage: 'home' },
  sources: {
    // Long interval: the mock sends no values by itself, and tests must not see the gauge go stale.
    stats: { kind: 'system_stats', intervalMs: 600_000, metrics: ['cpu.load', 'gpu.load', 'ram.used'] },
  },
  pages: [
    {
      id: 'home',
      title: 'Home',
      grid: { columns: 4, rows: 2 },
      widgets: [
        { id: 'clock', type: 'text', kind: 'clock', showDate: true, position: { x: 0, y: 0 } },
        {
          id: 'cpu',
          type: 'gauge',
          source: 'stats',
          metric: 'cpu.load',
          unit: '%',
          label: 'CPU',
          icon: '📈',
          position: { x: 1, y: 0 },
        },
        {
          id: 'game-one',
          type: 'launch',
          label: 'Game',
          icon: '🎮',
          position: { x: 0, y: 1 },
          action: { type: 'launch', kind: 'steam', appId: 440 },
        },
        {
          id: 'notes',
          type: 'launch',
          label: 'Notes',
          icon: '📝',
          position: { x: 1, y: 1 },
          action: { type: 'launch', kind: 'exe', path: 'C:/Windows/notepad.exe' },
          holdAction: { type: 'launch', kind: 'url', target: 'https://example.com/notes' },
        },
      ],
    },
    {
      id: 'in-game',
      title: 'In game',
      grid: { columns: 4, rows: 2 },
      widgets: [
        {
          id: 'heal',
          type: 'macro',
          label: 'Heal',
          icon: '❤',
          position: { x: 0, y: 0 },
          action: { type: 'key_macro', keys: ['1'], cycleMs: 3000, toggle: true },
        },
      ],
    },
  ],
}

// What `load_deck` returns for a deck file whose widgets have no `position`: easydeck-core
// `layout::resolve` has filled the first free cells in reading order (the same deck as
// tests/fixtures/valid_no_positions.deck.json). Open the UI with `?deck=no-positions`.
export const noPositionsDeck: Deck = {
  version: 1,
  name: 'No positions',
  sources: {},
  pages: [
    {
      id: 'home',
      title: 'Home',
      grid: { columns: 3, rows: 2 },
      widgets: ['a', 'b', 'c', 'd', 'e'].map((id, i) => ({
        id,
        type: 'launch' as const,
        label: id.toUpperCase(),
        action: { type: 'launch' as const, kind: 'steam' as const, appId: 440 },
        position: { x: i % 3, y: Math.floor(i / 3), w: 1, h: 1 },
      })),
    },
  ],
}

// A deck with one counter tile. Open the UI with `?deck=counter`. It is a separate fixture so
// the starter deck, and its screenshot baselines, stay unchanged.
export const counterDeck: Deck = {
  version: 1,
  name: 'Counter deck',
  sources: {},
  pages: [
    {
      id: 'home',
      title: 'Home',
      grid: { columns: 4, rows: 2 },
      widgets: [
        {
          id: 'water',
          type: 'counter',
          label: 'Water',
          start: 2,
          step: 3,
          max: 20,
          position: { x: 0, y: 0 },
        },
        {
          id: 'game-one',
          type: 'launch',
          label: 'Game',
          position: { x: 1, y: 0 },
          action: { type: 'launch', kind: 'steam', appId: 440 },
        },
      ],
    },
  ],
}

// A deck with a 5 s countdown and a stopwatch. Open the UI with `?deck=timer`. A separate
// fixture, so the starter deck and its screenshot baselines stay unchanged.
export const timerDeck: Deck = {
  version: 1,
  name: 'Timer deck',
  sources: {},
  pages: [
    {
      id: 'home',
      title: 'Home',
      grid: { columns: 4, rows: 2 },
      widgets: [
        {
          id: 'brew',
          type: 'timer',
          label: 'Brew',
          mode: 'countdown',
          durationS: 5,
          position: { x: 0, y: 0 },
        },
        { id: 'lap', type: 'timer', label: 'Lap', mode: 'stopwatch', position: { x: 1, y: 0 } },
      ],
    },
  ],
}

// A deck with two toggles: `mic` has a look per state, `lamp` has only an onAction and no
// looks. Open the UI with `?deck=toggle`. A separate fixture, so the starter deck and its
// screenshot baselines stay unchanged.
export const toggleDeck: Deck = {
  version: 1,
  name: 'Toggle deck',
  sources: {},
  pages: [
    {
      id: 'home',
      title: 'Home',
      grid: { columns: 4, rows: 2 },
      widgets: [
        {
          id: 'mic',
          type: 'toggle',
          label: 'Mic',
          icon: '🎤',
          onAction: { type: 'key_macro', keys: ['ctrl+m'] },
          offAction: { type: 'key_macro', keys: ['ctrl+m'] },
          on: { label: 'Live', icon: '🔴', color: '#7c2d2d' },
          off: { label: 'Muted', color: '#2d4a7c' },
          position: { x: 0, y: 0 },
        },
        {
          id: 'lamp',
          type: 'toggle',
          label: 'Lamp',
          icon: '💡',
          onAction: { type: 'key_macro', keys: ['f9'] },
          position: { x: 1, y: 0 },
        },
      ],
    },
  ],
}

// A deck with a wide and a tall volume tile. Open the UI with `?deck=volume`. A separate
// fixture so the starter deck, and its screenshot baselines, stay unchanged.
export const volumeDeck: Deck = {
  version: 1,
  name: 'Volume deck',
  sources: {},
  pages: [
    {
      id: 'home',
      title: 'Home',
      grid: { columns: 4, rows: 2 },
      widgets: [
        { id: 'vol-wide', type: 'volume', label: 'Volume', position: { x: 0, y: 0, w: 2, h: 1 } },
        { id: 'vol-tall', type: 'volume', position: { x: 2, y: 0, w: 1, h: 2 } },
      ],
    },
  ],
}

// In-memory system volume for the mock. `?volume=off` makes the volume commands fail.
const volume = { level: 40, muted: false }

export function installMockIpc(): void {
  window.__ipcCalls = []
  window.__emitSource = (id, payload) => emit('source:' + id, payload)
  window.__emitVolume = (payload) => {
    Object.assign(volume, payload)
    return emit('volume', payload)
  }
  mockIPC((cmd, args) => {
    window.__ipcCalls.push({ cmd, args })
    if (cmd === 'load_deck') {
      const which = new URLSearchParams(location.search).get('deck')
      if (which === 'no-positions') return noPositionsDeck
      if (which === 'counter') return counterDeck
      if (which === 'timer') return timerDeck
      if (which === 'toggle') return toggleDeck
      if (which === 'volume') return volumeDeck
      return fixtureDeck
    }
    if (cmd === 'run_action') {
      // Test hook: `?fail=<widgetId>` makes run_action reject for that widget.
      const fail = new URLSearchParams(location.search).get('fail')
      if (fail && (args as { widgetId?: string }).widgetId === fail) throw new Error('mock failure')
      return undefined
    }
    if (cmd.startsWith('volume_')) {
      if (new URLSearchParams(location.search).get('volume') === 'off') throw new Error('no device')
      const a = (args ?? {}) as { level?: number; muted?: boolean }
      if (cmd === 'volume_set') volume.level = a.level as number
      if (cmd === 'volume_mute') volume.muted = a.muted as boolean
      return cmd === 'volume_get' ? { ...volume } : undefined
    }
    throw new Error(`unmocked command: ${cmd}`)
  }, { shouldMockEvents: true })
}
