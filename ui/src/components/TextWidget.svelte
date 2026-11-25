<script lang="ts">
  import type { Widget } from '../lib/api'
  import { formatClock, msToNextSecond } from '../lib/clock'

  type TextData = Extract<Widget, { type: 'text' }>
  let { widget }: { widget: TextData } = $props()

  const kind = $derived(widget.kind ?? 'label')
  let now = $state(new Date())

  const parts = $derived(
    formatClock(now, { hour12: widget.hour12, seconds: widget.seconds ?? false }),
  )

  $effect(() => {
    if (kind === 'label') return
    let timer: ReturnType<typeof setTimeout>
    const tick = () => {
      now = new Date()
      timer = setTimeout(tick, msToNextSecond(Date.now()))
    }
    now = new Date()
    timer = setTimeout(tick, msToNextSecond(Date.now()))
    return () => clearTimeout(timer)
  })
</script>

{#if kind === 'clock'}
  <span class="time" data-testid="clock-time">{parts.time}</span>
  {#if widget.showDate}
    <span class="date" data-testid="clock-date">{parts.date}</span>
  {/if}
{:else if kind === 'date'}
  <span class="time date-main" data-testid="clock-date">{parts.date}</span>
{:else if widget.text}
  <span class="text">{widget.text}</span>
{/if}

<style>
  .time {
    font-size: 2.4rem;
    font-weight: 600;
    line-height: 1;
    font-variant-numeric: tabular-nums;
  }

  .date-main {
    font-size: 1.6rem;
  }

  .date {
    font-size: 1rem;
    opacity: 0.75;
  }

  .text {
    font-size: 1.2rem;
  }
</style>
