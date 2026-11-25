<script lang="ts">
  import type { PlacedWidget } from '../lib/api'
  import { press } from '../lib/press'
  import GaugeWidget from './GaugeWidget.svelte'
  import StatusWidget from './StatusWidget.svelte'
  import Counter from './Counter.svelte'
  import Timer from './Timer.svelte'
  import TextWidget from './TextWidget.svelte'
  import Volume from './Volume.svelte'

  // `hold` is true for a long press: the app then runs the widget's holdAction.
  let {
    widget,
    onpress,
  }: { widget: PlacedWidget; onpress: (id: string, hold?: boolean) => void } = $props()

  const pos = $derived(widget.position)
  let pressed = $state(false)
  const hasHold = $derived('holdAction' in widget && !!widget.holdAction)
  // A counter handles its own tap and long press; it makes no IPC call.
  let counter = $state<{ tap: () => void; reset: () => void }>()
  // A timer does the same, and also tells the tile when a countdown has ended.
  let timer = $state<{ tap: () => void; reset: () => void; isEnded: () => boolean }>()
  const own = $derived(
    widget.type === 'counter' ? counter : widget.type === 'timer' ? timer : undefined,
  )
  const isSelfHandled = $derived(widget.type === 'counter' || widget.type === 'timer')
</script>

{#if widget.type === 'volume'}
  <!-- A volume tile handles its own pointer events (slider drag, mute button), so it is a
       div, not the tap/hold button, and makes no run_action call. -->
  <div
    class="tile volume-tile"
    data-widget-id={widget.id}
    style:grid-column="{pos.x + 1} / span {pos.w ?? 1}"
    style:grid-row="{pos.y + 1} / span {pos.h ?? 1}"
  >
    <Volume {widget} w={pos.w ?? 1} h={pos.h ?? 1} />
    {#if widget.label}<span class="label">{widget.label}</span>{/if}
  </div>
{:else}
<button
  type="button"
  class="tile"
  class:pressed
  class:ended={timer?.isEnded()}
  data-widget-id={widget.id}
  style:grid-column="{pos.x + 1} / span {pos.w ?? 1}"
  style:grid-row="{pos.y + 1} / span {pos.h ?? 1}"
  use:press={{
    onTap: () => (isSelfHandled ? own?.tap() : onpress(widget.id)),
    onHold: isSelfHandled
      ? () => own?.reset()
      : hasHold
        ? () => onpress(widget.id, true)
        : undefined,
    onPressing: (v) => (pressed = v),
  }}
  oncontextmenu={(e) => e.preventDefault()}
>
  {#if widget.type === 'text'}
    <TextWidget {widget} />
  {:else if widget.type === 'gauge'}
    <GaugeWidget {widget} />
  {:else if widget.type === 'status'}
    <StatusWidget {widget} />
  {:else if widget.type === 'counter'}
    <Counter {widget} bind:this={counter} />
  {:else if widget.type === 'timer'}
    <Timer {widget} bind:this={timer} />
  {:else if widget.icon}
    <span class="icon" aria-hidden="true">{widget.icon}</span>
  {/if}
  {#if widget.label}
    <span class="label" aria-hidden={widget.type === 'gauge' || widget.type === 'status' ? 'true' : undefined}>{widget.label}</span>
  {/if}
</button>
{/if}

<style>
  .tile {
    min-width: 48px;
    min-height: 48px;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 0.4rem;
    border: 1px solid #3a3d45;
    border-radius: 12px;
    background: #23252b;
    color: inherit;
    font: inherit;
    cursor: pointer;
    user-select: none;
    -webkit-user-select: none;
    touch-action: manipulation;
    -webkit-tap-highlight-color: transparent;
    transition:
      transform 80ms ease-out,
      background-color 80ms ease-out,
      border-color 80ms ease-out;
  }

  .tile.pressed {
    transform: scale(0.94);
    background: #3b4252;
    border-color: #7aa2f7;
  }

  .volume-tile {
    padding: 0.5rem;
    cursor: default;
    touch-action: none;
  }

  .volume-tile :global(.volume) {
    flex: 1;
  }

  /* A countdown that has run out: new colour and a pulse until the next tap. */
  .tile.ended {
    background: #7c2d2d;
    border-color: #f7768e;
    animation: ended-pulse 1s ease-in-out infinite;
  }

  @keyframes ended-pulse {
    50% {
      background: #b04040;
      transform: scale(1.04);
    }
  }

  .icon {
    font-size: 2rem;
    line-height: 1;
  }

  .label {
    font-size: 1rem;
  }
</style>
