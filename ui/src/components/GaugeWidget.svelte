<script lang="ts">
  import type { Widget } from '../lib/api'
  import { formatGaugeValue, gaugeAngle, metricValue, GAUGE_SWEEP_DEG } from '../lib/gauge'
  import { sourceStore } from '../lib/source-store'

  type GaugeData = Extract<Widget, { type: 'gauge' }>
  let { widget }: { widget: GaugeData } = $props()

  const reading = $derived(sourceStore(widget.source))
  const value = $derived(metricValue($reading, widget.metric))
  const angle = $derived(
    value === null ? 0 : gaugeAngle(value, widget.min ?? 0, widget.max ?? 100),
  )

  // Dial: radius 40 in a 100x100 box, 270 degrees, starting at the lower left (rotate 135).
  const R = 40
  const C = 2 * Math.PI * R
  const track = (C * GAUGE_SWEEP_DEG) / 360
  const filled = $derived((C * angle) / 360)
</script>

<div class="gauge" data-testid="gauge" data-state={value === null ? 'nodata' : 'value'}>
  <svg viewBox="0 0 100 100" aria-hidden="true">
    <circle class="track" cx="50" cy="50" r={R} stroke-dasharray="{track} {C}" transform="rotate(135 50 50)" />
    {#if value !== null}
      <circle class="fill" cx="50" cy="50" r={R} stroke-dasharray="{filled} {C}" transform="rotate(135 50 50)" />
    {/if}
  </svg>
  <span class="reading" data-testid="gauge-value">
    {#if value === null}
      <span class="nodata">no data</span>
    {:else}
      {formatGaugeValue(value)}{#if widget.unit}<span class="unit">{widget.unit}</span>{/if}
    {/if}
  </span>
</div>

<style>
  .gauge {
    position: relative;
    width: 5.5rem;
    height: 5.5rem;
  }
  svg {
    width: 100%;
    height: 100%;
  }
  circle {
    fill: none;
    stroke-width: 9;
    stroke-linecap: round;
  }
  .track {
    stroke: #3a3d45;
  }
  .fill {
    stroke: #7aa2f7;
  }
  .reading {
    position: absolute;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 1.4rem;
    font-weight: 600;
    font-variant-numeric: tabular-nums;
  }
  .unit {
    font-size: 0.8rem;
    margin-left: 0.1rem;
    opacity: 0.75;
  }
  .nodata {
    font-size: 0.75rem;
    font-weight: 400;
    opacity: 0.6;
  }
</style>
