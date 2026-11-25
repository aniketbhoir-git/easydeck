<script lang="ts">
  import type { PlacedWidget } from '../lib/api'
  import { press } from '../lib/press'
  import GaugeWidget from './GaugeWidget.svelte'
  import StatusWidget from './StatusWidget.svelte'
  import Counter from './Counter.svelte'
  import Timer from './Timer.svelte'
  import Toggle from './Toggle.svelte'
  import TextWidget from './TextWidget.svelte'
  import Volume from './Volume.svelte'

  // `hold` is true for a long press: the app then runs the widget's holdAction.
  // In edit mode (`editing`) a tap only selects the tile: no action runs, and a counter, timer,
  // toggle or volume tile changes nothing.
  let {
    widget,
    onpress,
    editing = false,
    selected = false,
    onselect,
    locked = false,
    onreview,
  }: {
    widget: PlacedWidget
    onpress: (id: string, hold?: boolean) => void
    editing?: boolean
    selected?: boolean
    onselect?: (id: string) => void
    // A tile with an unapproved risk item is inert: it shows a lock, and a tap or a long press
    // opens the review screen instead of running anything (SPEC 4.3).
    locked?: boolean
    onreview?: () => void
  } = $props()

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

{#if widget.type === 'volume'}
  <!-- A volume tile handles its own pointer events (slider drag, mute button), so it is a
       div, not the tap/hold button, and makes no run_action call. -->
  <!-- In edit mode the slider and mute button are inert, so a tap lands on this div. -->
  <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
  <div
    class="tile volume-tile"
    class:editing
    class:selected
    data-widget-id={widget.id}
    onclick={() => editing && onselect?.(widget.id)}
    style:grid-column="{pos.x + 1} / span {pos.w ?? 1}"
    style:grid-row="{pos.y + 1} / span {pos.h ?? 1}"
  >
    <div class="volume-wrap" inert={editing}>
      <Volume {widget} w={pos.w ?? 1} h={pos.h ?? 1} />
    </div>
    {#if widget.label}<span class="label">{widget.label}</span>{/if}
  </div>
{:else}
<button
  type="button"
  class="tile"
  class:pressed
  class:editing
  class:selected
  class:ended={timer?.isEnded()}
  aria-pressed={widget.type === 'toggle' ? !!toggle?.isOn() : undefined}
  style:--tile-bg={toggle?.color()}
  data-widget-id={widget.id}
  data-locked={locked && !editing ? 'true' : undefined}
  style:grid-column="{pos.x + 1} / span {pos.w ?? 1}"
  style:grid-row="{pos.y + 1} / span {pos.h ?? 1}"
  use:press={{
    onTap: () =>
      editing
        ? onselect?.(widget.id)
        : locked
        ? onreview?.()
        : widget.type === 'toggle'
        ? toggle?.tap()
        : isSelfHandled
          ? own?.tap()
          : onpress(widget.id),
    onHold: editing
      ? undefined
      : locked
        ? () => onreview?.()
        : widget.type === 'toggle'
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
  {#if locked && !editing}
    <span class="lock" data-testid="lock-badge" role="img" aria-label="Locked: needs approval">🔒</span>
  {/if}
  {#if widget.label && widget.type !== 'toggle'}
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

  .volume-wrap {
    display: contents;
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

  /* Edit mode: a dashed outline marks every tile; the selected one is solid. */
  .tile.editing {
    border-style: dashed;
    border-color: #6b7080;
  }

  .tile.editing.selected {
    border-style: solid;
    border-color: #7aa2f7;
    box-shadow: 0 0 0 2px #7aa2f7;
  }

  .tile {
    position: relative;
  }

  .lock {
    position: absolute;
    top: 6px;
    right: 8px;
    font-size: 1rem;
    line-height: 1;
  }

  .tile[data-locked='true'] {
    border-style: dotted;
    opacity: 0.75;
  }

  .icon {
    font-size: 2rem;
    line-height: 1;
  }

  .label {
    font-size: 1rem;
  }
</style>
