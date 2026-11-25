// Shared tap / long-press detector. Framework-free; `press` below is the Svelte action.
// Long press: held >= HOLD_MS and moved <= MOVE_PX. A long press never also fires the tap.
// Without an onHold handler there is no long press: any press is a tap.
// pointercancel, moving more than MOVE_PX, or a second pointer cancels both.

export const HOLD_MS = 600
export const MOVE_PX = 10

export interface PressHandlers {
  onTap?: () => void
  onHold?: () => void
}

export interface PointerLike {
  pointerId: number
  clientX: number
  clientY: number
}

export interface PressDetector {
  down(e: PointerLike): void
  move(e: PointerLike): void
  up(e: PointerLike): void
  cancel(): void
}

export function createPressDetector(
  getHandlers: () => PressHandlers,
  hooks: { onStart?: () => void; onEnd?: () => void; holdMs?: number } = {},
): PressDetector {
  let id: number | null = null
  let startX = 0
  let startY = 0
  let timer: ReturnType<typeof setTimeout> | undefined
  let held = false

  function reset() {
    if (timer !== undefined) clearTimeout(timer)
    timer = undefined
    const was = id !== null
    id = null
    held = false
    if (was) hooks.onEnd?.()
  }

  return {
    down(e) {
      if (id !== null) {
        reset() // a second pointer cancels the gesture
        return
      }
      id = e.pointerId
      startX = e.clientX
      startY = e.clientY
      held = false
      hooks.onStart?.()
      // No hold handler: a long press stays a tap, so a slow touch is not lost.
      if (!getHandlers().onHold) return
      timer = setTimeout(() => {
        timer = undefined
        held = true
        getHandlers().onHold?.()
      }, hooks.holdMs ?? HOLD_MS)
    },
    move(e) {
      if (id !== e.pointerId) return
      if (Math.hypot(e.clientX - startX, e.clientY - startY) > MOVE_PX) reset()
    },
    up(e) {
      if (id !== e.pointerId) return
      const fireTap = !held
      reset()
      if (fireTap) getHandlers().onTap?.()
    },
    cancel: reset,
  }
}

export interface PressOptions extends PressHandlers {
  /** Called with true while a press is in progress (for the pressed style). */
  onPressing?: (pressing: boolean) => void
}

/** Svelte action: `use:press={{ onTap, onHold, onPressing }}`. Replaces the click handler. */
export function press(node: HTMLElement, options: PressOptions) {
  let opts = options
  const detector = createPressDetector(() => opts, {
    onStart: () => opts.onPressing?.(true),
    onEnd: () => opts.onPressing?.(false),
  })
  const down = (e: PointerEvent) => {
    if (e.pointerType === 'mouse' && e.button !== 0) return
    detector.down(e)
  }
  const move = (e: PointerEvent) => detector.move(e)
  const up = (e: PointerEvent) => detector.up(e)
  const cancel = () => detector.cancel()
  // Keyboard activation (Enter/Space) arrives as a click with detail 0; pointer clicks are
  // handled by the pointer events above.
  const click = (e: MouseEvent) => {
    if (e.detail === 0) opts.onTap?.()
  }
  node.addEventListener('pointerdown', down)
  node.addEventListener('pointermove', move)
  node.addEventListener('pointerup', up)
  node.addEventListener('pointercancel', cancel)
  node.addEventListener('pointerleave', cancel)
  node.addEventListener('click', click)
  return {
    update(next: PressOptions) {
      opts = next
    },
    destroy() {
      detector.cancel()
      node.removeEventListener('pointerdown', down)
      node.removeEventListener('pointermove', move)
      node.removeEventListener('pointerup', up)
      node.removeEventListener('pointercancel', cancel)
      node.removeEventListener('pointerleave', cancel)
      node.removeEventListener('click', click)
    },
  }
}
