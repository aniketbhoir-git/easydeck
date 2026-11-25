<script lang="ts">
  import { getContext } from 'svelte'
  import type { Widget } from '../lib/api'
  import { counterKey, nextValue, parseStored, resetValue } from '../lib/counter'
  import { readState, writeState } from '../lib/widget-state'

  type CounterData = Extract<Widget, { type: 'counter' }>
  let { widget }: { widget: CounterData } = $props()

  const deckName = getContext<(() => string) | undefined>('deckName')
  const key = $derived(counterKey(deckName?.() ?? '', widget.id))
  let value = $state(0)
  // Runs on mount, when the key changes (renamed deck) and when the widget changes (a new
  // min or max): reload from storage and clamp.
  $effect.pre(() => {
    value = parseStored(readState(key), widget)
  })

  function set(v: number) {
    value = v
    writeState(key, String(v))
  }

  export function tap() {
    set(nextValue(value, widget))
  }

  export function reset() {
    set(resetValue(widget))
  }
</script>

<span class="value" data-testid="counter-value">{value}</span>

<style>
  .value {
    font-size: 2.4rem;
    font-weight: 600;
    line-height: 1;
    font-variant-numeric: tabular-nums;
  }
</style>
