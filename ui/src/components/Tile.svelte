<script lang="ts">
  import type { PlacedWidget } from '../lib/api'
  import { press } from '../lib/press'
  import GaugeWidget from './GaugeWidget.svelte'
  import StatusWidget from './StatusWidget.svelte'
  import Counter from './Counter.svelte'
  import Timer from './Timer.svelte'
  import Toggle from './Toggle.svelte'
  import TextWidget from './TextWidget.svelte'

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
  // A toggle flips itself and tells the backend which state it switched to. No long press.
  let toggle = $state<{ tap: () => void; color: () => string | undefined; isOn: () => boolean }>()
</script>

<button
  type="button"
  class="tile"
  class:pressed
  class:ended={timer?.isEnded()}
  aria-pressed={widget.type === 'toggle' ? !!toggle?.isOn() : undefined}
  style:--tile-bg={toggle?.color()}
  data-widget-id={widget.id}
  style:grid-column="{pos.x + 1} / span {pos.w ?? 1}"
  style:grid-row="{pos.y + 1} / span {pos.h ?? 1}"
  use:press={{
    onTap: () =>
      widget.type === 'toggle'
        ? toggle?.tap()
        : isSelfHandled
          ? own?.tap()
          : onpress(widget.id),
    onHold:
      widget.type === 'toggle'
        ? undefined
        : isSelfHandled
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
  {:else if widget.type === 'toggle'}
    <Toggle {widget} bind:this={toggle} />
  {:else if widget.icon}
    <span class="icon" aria-hidden="true">{widget.icon}</span>
  {/if}
  {#if widget.label && widget.type !== 'toggle'}
    <span class="label" aria-hidden={widget.type === 'gauge' || widget.type === 'status' ? 'true' : undefined}>{widget.label}</span>
  {/if}
</button>

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
    background: var(--tile-bg, #23252b);
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
