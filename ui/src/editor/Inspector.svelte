<script lang="ts">
  import type { Action } from '../lib/api'
  import type { Draft } from './draft.svelte'
  import WidgetForm from './forms/WidgetForm.svelte'
  import { shownByForm } from './forms/actions'

  // The selected tile's details, its form (R4-2) and the validation problems. Move, resize
  // and delete (R4-3) are not built yet.
  let { draft }: { draft: Draft } = $props()

  const selected = $derived(draft.selected)
  const prefix = $derived(
    selected ? `/pages/${selected.pageIndex}/widgets/${selected.widgetIndex}` : null,
  )
  const slotActions = (w: object): Record<string, Action | undefined> => {
    const r = w as Record<string, Action | undefined>
    return { action: r.action, holdAction: r.holdAction, onAction: r.onAction, offAction: r.offAction }
  }
  // The tile's problems that no form field shows (a field error is not listed twice).
  const own = $derived(
    prefix === null || !selected
      ? []
      : draft.errors.filter(
          (e) =>
            (e.path === prefix || e.path.startsWith(prefix + '/')) &&
            !shownByForm(e, prefix, slotActions(selected.widget)),
        ),
  )
  const shown = $derived(
    prefix === null
      ? []
      : draft.errors.filter((e) => e.path === prefix || e.path.startsWith(prefix + '/')),
  )
  const others = $derived(draft.errors.filter((e) => !shown.includes(e)))
</script>

<section class="inspector" aria-label="Inspector">
  <h2>Selected tile</h2>
  {#if selected}
    {@const w = selected.widget}
    <dl data-testid="inspector-details">
      <dt>Id</dt>
      <dd>{w.id}</dd>
      <dt>Type</dt>
      <dd>{w.type}</dd>
      {#if w.label}<dt>Label</dt><dd>{w.label}</dd>{/if}
      {#if w.position}
        <dt>Cell</dt>
        <dd>column {w.position.x + 1}, row {w.position.y + 1}</dd>
      {/if}
    </dl>
    {#if prefix}{#key w.id}<WidgetForm {draft} widget={w} {prefix} />{/key}{/if}
    {#if own.length}
      <ul class="errors" data-testid="inspector-errors">
        {#each own as e, i (i)}
          <li><strong>{e.message}</strong> <span>{e.fix}</span></li>
        {/each}
      </ul>
    {/if}
  {:else}
    <p class="hint">Tap a tile to select it.</p>
  {/if}

  {#if others.length}
    <h2>Other problems</h2>
    <ul class="errors" data-testid="deck-errors">
      {#each others as e, i (i)}
        <li>
          <strong>{e.message}</strong>
          {#if e.path}<code>{e.path}</code>{/if}
          <span>{e.fix}</span>
        </li>
      {/each}
    </ul>
  {/if}
</section>

<style>
  h2 {
    margin: 0 0 8px;
    font-size: 0.85rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    color: #9aa0ae;
  }

  section {
    font-size: 0.9rem;
  }

  dl {
    display: grid;
    grid-template-columns: auto 1fr;
    gap: 4px 12px;
    margin: 0 0 12px;
  }

  dt {
    color: #9aa0ae;
  }

  dd {
    margin: 0;
    overflow-wrap: anywhere;
  }

  .hint {
    margin: 0 0 12px;
    color: #9aa0ae;
  }

  .errors {
    margin: 0 0 12px;
    padding: 0;
    list-style: none;
    display: grid;
    gap: 6px;
  }

  .errors li {
    padding: 8px 10px;
    border: 1px solid #f7768e;
    border-radius: 8px;
    background: #2b1d21;
  }

  .errors span,
  .errors code {
    display: block;
    color: #c7c9d1;
    font-size: 0.8rem;
  }
</style>
