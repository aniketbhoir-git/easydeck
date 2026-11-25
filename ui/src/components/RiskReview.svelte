<script lang="ts">
  import type { RiskReview, ReviewItem } from '../lib/api'
  import { placeName } from '../lib/risk'

  // The review screen (SPEC §4.3): lists the risk items of the deck by kind and approves the
  // ones the user ticks. Nothing is ticked at first, and approving is the only way an item
  // starts to run. The backend refuses to approve a hash that is not in the deck.
  let {
    review,
    onapprove,
    onclose,
  }: {
    review: RiskReview
    onapprove: (hashes: string[]) => Promise<void>
    onclose: () => void
  } = $props()

  const groups = [
    { kind: 'launch', title: 'Programs and links' },
    { kind: 'key_macro', title: 'Key macros' },
    { kind: 'shell', title: 'Shell commands' },
    { kind: 'network', title: 'Network' },
  ] as const

  let selected = $state<Set<string>>(new Set())
  let busy = $state(false)
  let error = $state<string | null>(null)

  const pending = $derived(review.items.filter((r) => !r.approved))
  const chosen = $derived([...selected].filter((h) => pending.some((r) => r.item.hash === h)))

  function toggle(hash: string, on: boolean) {
    const next = new Set(selected)
    if (on) next.add(hash)
    else next.delete(hash)
    selected = next
  }

  async function approve() {
    busy = true
    error = null
    try {
      await onapprove(chosen)
      selected = new Set()
    } catch (e) {
      error = (e as { message?: string })?.message ?? String(e)
    } finally {
      busy = false
    }
  }

  const rows = (kind: string): ReviewItem[] => review.items.filter((r) => r.item.kind === kind)
</script>

<div class="backdrop">
  <div class="dialog" role="dialog" aria-modal="true" aria-labelledby="risk-title" data-testid="risk-review">
    <header>
      <h2 id="risk-title">Review what this deck can run</h2>
      <button type="button" class="close" onclick={onclose} aria-label="Close">Close</button>
    </header>
    <p class="hint">
      {#if pending.length === 0}
        Everything in this deck is approved.
      {:else}
        {pending.length} item{pending.length === 1 ? '' : 's'} wait{pending.length === 1 ? 's' : ''} for
        your approval. Tick the ones you trust. Until then they stay locked.
      {/if}
    </p>

    <div class="list">
      {#each groups as group (group.kind)}
        {@const items = rows(group.kind)}
        {#if items.length > 0}
          <section data-kind={group.kind}>
            <h3>{group.title}</h3>
            {#each items as row (row.item.hash)}
              <div class="row" class:done={row.approved} data-approved={row.approved}>
                {#if row.approved}
                  <span class="state">Approved</span>
                {:else}
                  <input
                    type="checkbox"
                    id={'risk-' + row.item.hash}
                    checked={selected.has(row.item.hash)}
                    onchange={(e) => toggle(row.item.hash, e.currentTarget.checked)}
                  />
                {/if}
                <label for={'risk-' + row.item.hash}>
                  <span class="summary">{row.item.summary}</span>
                  {#if row.item.systemShortcut}
                    <span class="badge" data-testid="system-shortcut">System shortcut</span>
                  {/if}
                  <span class="where">{row.item.locations.map(placeName).join(', ')}</span>
                </label>
              </div>
            {/each}
          </section>
        {/if}
      {/each}
    </div>

    {#if error}<p class="error" role="alert">Could not approve: {error}</p>{/if}
    <footer>
      <button type="button" class="primary" disabled={chosen.length === 0 || busy} onclick={approve}>
        Approve selected ({chosen.length})
      </button>
    </footer>
  </div>
</div>

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    z-index: 10;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 12px;
    background: rgba(0, 0, 0, 0.6);
  }

  .dialog {
    display: flex;
    flex-direction: column;
    gap: 8px;
    width: min(720px, 100%);
    max-height: 100%;
    box-sizing: border-box;
    padding: 16px;
    border: 1px solid #3a3d45;
    border-radius: 12px;
    background: #1d1f24;
  }

  header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
  }

  h2 {
    margin: 0;
    font-size: 1.1rem;
  }

  h3 {
    margin: 8px 0 4px;
    font-size: 0.95rem;
    color: #aab;
  }

  .hint {
    margin: 0;
    color: #aab;
  }

  .list {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
  }

  .row {
    display: flex;
    align-items: center;
    gap: 12px;
    min-height: 48px;
    padding: 4px 0;
  }

  .row input {
    flex: none;
    width: 28px;
    height: 28px;
  }

  .state {
    flex: none;
    width: 84px;
    color: #7ec699;
    font-size: 0.85rem;
  }

  label {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }

  .summary {
    overflow-wrap: anywhere;
  }

  .where {
    color: #8a8fa0;
    font-size: 0.8rem;
  }

  .badge {
    align-self: flex-start;
    padding: 0 6px;
    border-radius: 6px;
    background: #7c2d2d;
    font-size: 0.8rem;
  }

  .error {
    margin: 0;
    color: #f7768e;
  }

  footer {
    display: flex;
    justify-content: flex-end;
  }

  button {
    min-width: 48px;
    min-height: 48px;
    padding: 0 1rem;
    border: 1px solid #3a3d45;
    border-radius: 12px;
    background: #23252b;
    color: inherit;
    font: inherit;
    cursor: pointer;
    touch-action: manipulation;
  }

  button.primary {
    border-color: #7aa2f7;
    background: #3b4252;
  }

  button:disabled {
    opacity: 0.5;
    cursor: default;
  }
</style>
