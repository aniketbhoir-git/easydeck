// Pure timer state machine. No DOM, no timers, no storage. The caller passes the time in as
// `now` (milliseconds from a monotonic clock, `performance.now` in the app), so elapsed time
// never depends on how often anything redraws.
//
// States: idle, running, paused, ended (countdown only).
//   tap:        idle -> running, running -> paused, paused -> running,
//               ended -> idle (the tap clears the pulse and also resets the timer)
//   long press: any -> idle
// A countdown ends when elapsed time reaches its duration. `settle` applies that change; every
// other function settles first, so a late redraw cannot miss or delay the end.

export interface TimerConfig {
  mode: 'countdown' | 'stopwatch'
  /** Countdown only, in seconds. */
  durationS?: number
}

export type TimerPhase = 'idle' | 'running' | 'paused' | 'ended'

export interface TimerState {
  phase: TimerPhase
  /** Milliseconds counted before the current run started. */
  accumulatedMs: number
  /** Clock reading when the current run started. Meaningful only while running. */
  startedAt: number
}

export const idleState: TimerState = { phase: 'idle', accumulatedMs: 0, startedAt: 0 }

export const durationMs = (c: TimerConfig): number =>
  c.mode === 'countdown' ? (c.durationS ?? 0) * 1000 : Infinity

/** Milliseconds counted so far, also past the end of a countdown that has not been settled. */
export function elapsedMs(s: TimerState, now: number): number {
  return s.phase === 'running' ? s.accumulatedMs + Math.max(0, now - s.startedAt) : s.accumulatedMs
}

/** Moves a running countdown to `ended` once its time is up. Anything else is returned as is. */
export function settle(s: TimerState, c: TimerConfig, now: number): TimerState {
  if (s.phase === 'running' && elapsedMs(s, now) >= durationMs(c)) {
    return { phase: 'ended', accumulatedMs: durationMs(c), startedAt: 0 }
  }
  return s
}

/** The state after a tap. */
export function tap(s: TimerState, c: TimerConfig, now: number): TimerState {
  const cur = settle(s, c, now)
  switch (cur.phase) {
    case 'idle':
    case 'paused':
      return { phase: 'running', accumulatedMs: cur.accumulatedMs, startedAt: now }
    case 'running':
      return { phase: 'paused', accumulatedMs: elapsedMs(cur, now), startedAt: 0 }
    case 'ended':
      return idleState
  }
}

/** The state after a long press. */
export function reset(): TimerState {
  return idleState
}

/** The milliseconds to show: time left for a countdown, time counted for a stopwatch. */
export function displayMs(s: TimerState, c: TimerConfig, now: number): number {
  const e = elapsedMs(settle(s, c, now), now)
  return c.mode === 'countdown' ? Math.max(0, durationMs(c) - e) : e
}

/** `m:ss`, or `h:mm:ss` from one hour. A countdown rounds up, so it reads 0:00 only at the end;
 *  a stopwatch rounds down. */
export function formatTime(ms: number, roundUp: boolean): string {
  const total = Math.max(0, roundUp ? Math.ceil(ms / 1000) : Math.floor(ms / 1000))
  const h = Math.floor(total / 3600)
  const m = Math.floor((total % 3600) / 60)
  const sec = String(total % 60).padStart(2, '0')
  return h > 0 ? `${h}:${String(m).padStart(2, '0')}:${sec}` : `${m}:${sec}`
}

export function displayText(s: TimerState, c: TimerConfig, now: number): string {
  return formatTime(displayMs(s, c, now), c.mode === 'countdown')
}
