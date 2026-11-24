// Test-only IPC layer. main.ts imports this module only when the build mode is "e2e"
// (`vite build --mode e2e`), so it is never part of the production bundle.
import { mockIPC } from '@tauri-apps/api/mocks'
import type { Deck } from './api'

declare global {
  interface Window {
    __ipcCalls: { cmd: string; args: unknown }[]
  }
}

// Mirrors the home page and the in-game page of examples/starter.deck.json,
// reduced to the fields the UI model draws.
export const fixtureDeck: Deck = {
  version: 1,
  name: 'Starter deck',
  settings: { homePage: 'home' },
  sources: {
    stats: { kind: 'system_stats', metrics: ['cpu.load', 'gpu.load', 'ram.used'] },
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

export function installMockIpc(): void {
  window.__ipcCalls = []
  mockIPC((cmd, args) => {
    window.__ipcCalls.push({ cmd, args })
    if (cmd === 'load_deck') return fixtureDeck
    if (cmd === 'run_action') return undefined
    throw new Error(`unmocked command: ${cmd}`)
  })
}
