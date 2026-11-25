import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'

/** The default output device's volume: level 0 to 100, and mute. */
export interface VolumeState {
  level: number
  muted: boolean
}

/** One `volume_set` per 50 ms at most while dragging. */
export const SET_INTERVAL_MS = 50

export const volumeGet = () => invoke<VolumeState>('volume_get')
export const volumeSet = (level: number) => invoke<void>('volume_set', { level })
export const volumeMute = (muted: boolean) => invoke<void>('volume_mute', { muted })

/** Calls `handler` when the backend sees the volume or mute change (the volume event). */
export function onVolume(handler: (v: VolumeState) => void): Promise<UnlistenFn> {
  return listen<VolumeState>('volume', (e) => handler(e.payload))
}

/** Level 0 to 100 for a pointer position along the track. Vertical: the bottom is 0. */
export function levelFromPointer(
  pointer: number,
  start: number,
  size: number,
  vertical: boolean,
): number {
  if (!(size > 0)) return 0
  const along = vertical ? (start + size - pointer) / size : (pointer - start) / size
  return Math.round(Math.min(1, Math.max(0, along)) * 100)
}

/** A slider is vertical on a tile taller than it is wide; otherwise horizontal. */
export function isVertical(w = 1, h = 1): boolean {
  return h > w
}

/**
 * Sends the latest pushed value at most once per `intervalMs`. The first push goes out at
 * once; later pushes inside the window wait, and only the newest goes out when the window
 * ends, so the final value is always sent.
 */
export function createLatestThrottle<T>(send: (v: T) => void, intervalMs = SET_INTERVAL_MS) {
  let lastSent = -Infinity
  let pending: { v: T } | undefined
  let timer: ReturnType<typeof setTimeout> | undefined

  const fire = () => {
    timer = undefined
    if (!pending) return
    const { v } = pending
    pending = undefined
    lastSent = Date.now()
    send(v)
  }

  return {
    push(v: T) {
      pending = { v }
      if (timer !== undefined) return
      const wait = lastSent + intervalMs - Date.now()
      if (wait <= 0) fire()
      else timer = setTimeout(fire, wait)
    },
    cancel() {
      if (timer !== undefined) clearTimeout(timer)
      timer = undefined
      pending = undefined
    },
  }
}
