<script lang="ts">
  import { getContext } from 'svelte'
  import { runToggle } from '../lib/api'
  import type { Widget } from '../lib/api'
  import { flip, lookFor, parseStored, toggleKey } from '../lib/toggle'
  import type { ToggleState } from '../lib/toggle'
  import { readState, writeState } from '../lib/widget-state'

  type ToggleData = Extract<Widget, { type: 'toggle' }>
  let { widget }: { widget: ToggleData } = $props()

  const deckName = getContext<(() => string) | undefined>('deckName')
  const key = $derived(toggleKey(deckName?.() ?? '', widget.id))
  // The state lives on this device, never in the deck file. A toggle starts off.
  let current = $state<ToggleState>('off')
  // Runs on mount and whenever the key changes (renamed deck): reload from storage.
  $effect.pre(() => {
    current = parseStored(readState(key))
  })
  let busy = false
  const view = $derived(lookFor(widget, current))

  /** Flips at once; the backend runs the action for the new state. If it rejects, the state
   *  goes back and nothing is saved. A tap while a call is pending is ignored. */
  export async function tap() {
    if (busy) return
    busy = true
    // The key can change while the call is pending (renamed deck). Then the new key already
    // holds its own state: do not write to it and do not revert over it.
    const k = key
    const before = current
    const target = flip(before)
    current = target
    try {
      await runToggle(widget.id, target)
      if (key === k) writeState(k, target)
    } catch (err) {
      if (key === k) current = before
      console.error('run_action failed', err)
    } finally {
      busy = false
    }
  }

  export function isOn() {
    return current === 'on'
  }

  /** The colour of the current look, for the tile background. */
  export function color() {
    return view.color
  }
</script>

{#if view.icon}
  <span class="icon" aria-hidden="true">{view.icon}</span>
{/if}
{#if view.label}
  <span class="label" data-testid="toggle-label">{view.label}</span>
{/if}

<style>
  .icon {
    font-size: 2rem;
    line-height: 1;
  }

  .label {
    font-size: 1rem;
  }
</style>
