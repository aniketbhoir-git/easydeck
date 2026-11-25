import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import type { MediaControl } from './generated/MediaControl'

export type MediaStatus = 'playing' | 'paused' | 'stopped' | 'none'
export type MediaCommand = MediaControl

/** What the system media session reports. `thumbnail` is a `data:image/...;base64,` URL. */
export interface MediaState {
  title: string
  artist: string
  status: MediaStatus
  canPlayPause: boolean
  canNext: boolean
  canPrevious: boolean
  thumbnail?: string
}

export const ALL_CONTROLS: MediaControl[] = ['previous', 'play_pause', 'next']

export const mediaGet = () => invoke<MediaState>('media_get')
export const mediaCommand = (command: MediaCommand) => invoke<void>('media_command', { command })

/** Calls `handler` when the backend sees the track, status or controls change (event `media`). */
export function onMedia(handler: (m: MediaState) => void): Promise<UnlistenFn> {
  return listen<MediaState>('media', (e) => handler(e.payload))
}

/**
 * The controls to draw: those the deck lists (all three when it lists none), in the fixed
 * order previous, play/pause, next. A control appears at most once.
 */
export function visibleControls(controls?: MediaControl[]): MediaControl[] {
  if (!controls) return ALL_CONTROLS
  return ALL_CONTROLS.filter((c) => controls.includes(c))
}

/** Whether the session can do `control` right now. */
export function canDo(m: MediaState | null, control: MediaControl): boolean {
  if (!m || m.status === 'none') return false
  return control === 'play_pause' ? m.canPlayPause : control === 'next' ? m.canNext : m.canPrevious
}

const SAFE_THUMBNAIL = /^data:image\/(png|jpeg|gif|webp|bmp);base64,[A-Za-z0-9+/]+={0,2}$/

/** The thumbnail URL when it is a base64 image data URL; anything else gives undefined. */
export function safeThumbnail(url: unknown): string | undefined {
  return typeof url === 'string' && SAFE_THUMBNAIL.test(url) ? url : undefined
}
