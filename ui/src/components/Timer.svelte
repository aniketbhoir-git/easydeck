<script lang="ts">
  import type { Widget } from '../lib/api'
  import { displayText, idleState, reset as resetState, settle, tap as tapState } from '../lib/timer'

  type TimerData = Extract<Widget, { type: 'timer' }>
  let { widget }: { widget: TimerData } = $props()

  // Monotonic: unaffected by changes to the system date or time.
  const clock = () => performance.now()

  // State is in memory only: it is lost when the app closes.
  const config = $derived({ mode: widget.mode, durationS: widget.durationS })
  let timerState = $state(idleState)
  // The clock reading of the last redraw. Only the redraw reads it; the time itself comes from
  // `clock`, so a late or skipped redraw never changes the result.
  let now = $state(clock())
  const view = $derived(settle(timerState, config, now))

  $effect(() => {
    if (view.phase !== 'running') return
    const id = setInterval(() => (now = clock()), 100)
    return () => clearInterval(id)
  })

  export function tap() {
    now = clock()
    timerState = tapState(timerState, config, now)
  }

  export function reset() {
    now = clock()
    timerState = resetState()
  }

  /** True from the end of a countdown until the next tap. The tile uses it for its colour. */
  export function isEnded() {
    return view.phase === 'ended'
  }
</script>

<span class="value" data-testid="timer-value" data-phase={view.phase}
  >{displayText(timerState, config, now)}</span
>

<style>
  .value {
    font-size: 2.4rem;
    font-weight: 600;
    line-height: 1;
    font-variant-numeric: tabular-nums;
  }
</style>
