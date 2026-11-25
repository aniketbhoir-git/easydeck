<script lang="ts">
  import { PALETTE_TYPES, type WidgetType } from './draft.svelte'

  // One button per widget type. A tap adds the widget in the first free cell of the page.
  let { onadd, disabled = false }: { onadd: (type: WidgetType) => void; disabled?: boolean } =
    $props()
</script>

<section class="palette" aria-label="Widget palette">
  <h2>Add a widget</h2>
  <div class="types">
    {#each PALETTE_TYPES as t (t.type)}
      <button
        type="button"
        data-palette-type={t.type}
        {disabled}
        onclick={() => onadd(t.type)}>{t.name}</button
      >
    {/each}
  </div>
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

  .types {
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: 8px;
  }

  button {
    min-height: 48px;
    padding: 0 6px;
    border: 1px solid #3a3d45;
    border-radius: 10px;
    background: #23252b;
    color: inherit;
    font: inherit;
    font-size: 0.9rem;
    cursor: pointer;
    touch-action: manipulation;
  }

  button:active:not(:disabled) {
    background: #3b4252;
    border-color: #7aa2f7;
  }

  button:disabled {
    opacity: 0.45;
    cursor: default;
  }
</style>
