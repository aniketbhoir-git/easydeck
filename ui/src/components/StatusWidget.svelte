<script lang="ts">
  import type { Widget } from '../lib/api'
  import { evaluateStatus } from '../lib/gauge'
  import { sourceStore } from '../lib/source-store'
  import { useSourceInterval } from '../lib/source-interval'

  type StatusData = Extract<Widget, { type: 'status' }>
  let { widget }: { widget: StatusData } = $props()

  const intervalMs = useSourceInterval()
  const reading = $derived(sourceStore(widget.source, intervalMs(widget.source)))
  const state = $derived(evaluateStatus($reading, widget.path, widget.okWhen))
  const text = $derived({ ok: 'OK', problem: 'Problem', nodata: 'No data' }[state])
</script>

<div
  class="status"
  data-testid="status"
  role="img"
  aria-label="{widget.label ?? 'Status'} {{ ok: 'OK', problem: 'problem', nodata: 'no data' }[state]}"
  data-state={state}
>
  <span class="dot" aria-hidden="true"></span>
  <span class="text">{text}</span>
</div>

<style>
  .status {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 1.3rem;
    font-weight: 600;
  }
  .dot {
    width: 0.9rem;
    height: 0.9rem;
    border-radius: 50%;
    background: #6b6f7a;
  }
  [data-state='ok'] .dot {
    background: #9ece6a;
  }
  [data-state='problem'] .dot {
    background: #f7768e;
  }
  [data-state='nodata'] {
    opacity: 0.6;
    font-weight: 400;
  }
</style>
