<script lang="ts">
  import type { PlacedWidget } from '../lib/api'
  import { press } from '../lib/press'
  import TextWidget from './TextWidget.svelte'

  // `hold` is true for a long press: the app then runs the widget's holdAction.
  let {
    widget,
    onpress,
  }: { widget: PlacedWidget; onpress: (id: string, hold?: boolean) => void } = $props()

  const pos = $derived(widget.position)
  let pressed = $state(false)
  const hasHold = $derived('holdAction' in widget && !!widget.holdAction)
</script>

<button
  type="button"
  class="tile"
  class:pressed
  data-widget-id={widget.id}
  style:grid-column="{pos.x + 1} / span {pos.w ?? 1}"
  style:grid-row="{pos.y + 1} / span {pos.h ?? 1}"
  use:press={{
    onTap: () => onpress(widget.id),
    onHold: hasHold ? () => onpress(widget.id, true) : undefined,
    onPressing: (v) => (pressed = v),
  }}
  oncontextmenu={(e) => e.preventDefault()}
>
  {#if widget.type === 'text'}
    <TextWidget {widget} />
  {:else if widget.icon}
    <span class="icon" aria-hidden="true">{widget.icon}</span>
  {/if}
  {#if widget.label}
    <span class="label">{widget.label}</span>
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

  .icon {
    font-size: 2rem;
    line-height: 1;
  }

  .label {
    font-size: 1rem;
  }
</style>
