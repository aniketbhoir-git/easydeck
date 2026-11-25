<script lang="ts">
  import type { Snippet } from 'svelte'
  import type { Deck } from '../lib/api'
  import { resolvePage } from '../lib/pages'
  import type { Draft, WidgetType } from './draft.svelte'
  import Palette from './Palette.svelte'
  import Inspector from './Inspector.svelte'

  // The frame around the deck. It is always mounted, so the deck below it keeps its state
  // (counter value, running timer) when edit mode starts and ends. With no `draft` it is
  // transparent. With a draft it adds the bar, the palette and the inspector.
  //
  // `onsave` is optional: the deck file cannot be written until R4-3. Without it the Save
  // buttons stay disabled and leaving with changes offers only "Discard".
  let {
    draft,
    pageId,
    onexit,
    onsave,
    children,
  }: {
    draft: Draft | null
    pageId: string | null
    onexit: () => void
    onsave?: (deck: Deck) => Promise<void>
    children: Snippet
  } = $props()

  let confirming = $state(false)
  let saveError = $state<string | null>(null)

  const page = $derived(draft ? resolvePage(draft.deck, pageId) : undefined)
  const savable = $derived(!!draft && !!onsave && draft.canSave)

  function add(type: WidgetType) {
    draft?.addWidget(page?.id, type)
  }

  function done() {
    if (draft?.dirty) confirming = true
    else onexit()
  }

  async function save() {
    if (!draft || !onsave || !draft.canSave) return
    saveError = null
    try {
      await onsave(draft.deck)
      draft.markSaved()
      confirming = false
      onexit()
    } catch (e) {
      saveError = e instanceof Error ? e.message : String(e)
    }
  }

  function discard() {
    confirming = false
    onexit()
  }
</script>

<div class="shell" class:active={!!draft} style:--deck-height={draft ? '100%' : undefined}>
  {#if draft}
    <header class="bar" data-testid="editor-bar">
      <strong>Edit mode</strong>
      <span class="state" data-testid="editor-state">
        {#if draft.dirty}Unsaved changes{:else}No changes{/if}
        {#if draft.errors.length}
          · {draft.errors.length} {draft.errors.length === 1 ? 'problem' : 'problems'}
        {/if}
        {#if draft.notice}· {draft.notice}{/if}
        {#if saveError}· Save failed: {saveError}{/if}
      </span>
      <span class="spacer"></span>
      <button type="button" data-action="save" disabled={!savable} onclick={save}>Save</button>
      <button
        type="button"
        data-action="discard"
        disabled={!draft.dirty}
        onclick={() => draft.discard()}>Discard</button
      >
      <button type="button" data-action="done" class="primary" onclick={done}>Done</button>
    </header>
  {/if}
  <div class="main">
    <div class="stage">{@render children()}</div>
    {#if draft}
      <aside class="side">
        <Palette onadd={add} disabled={!page} />
        <Inspector {draft} />
      </aside>
    {/if}
  </div>

  {#if draft && confirming}
    <div class="backdrop">
      <div class="dialog" role="dialog" aria-modal="true" aria-labelledby="leave-title">
        <h2 id="leave-title">Unsaved changes</h2>
        <p>
          Leave edit mode and lose your changes?
          {#if !onsave}Saving is not available yet.{/if}
          {#if onsave && !draft.canSave}Fix the problems before you save.{/if}
        </p>
        <div class="buttons">
          <button type="button" data-action="confirm-save" disabled={!savable} onclick={save}
            >Save</button
          >
          <button type="button" data-action="confirm-discard" onclick={discard}>Discard</button>
          <button
            type="button"
            data-action="keep-editing"
            class="primary"
            onclick={() => (confirming = false)}>Keep editing</button
          >
        </div>
      </div>
    </div>
  {/if}
</div>

<style>
  .shell {
    height: 100%;
    display: flex;
    flex-direction: column;
    box-sizing: border-box;
  }

  .main {
    flex: 1;
    min-height: 0;
    display: flex;
  }

  .stage {
    flex: 1;
    min-width: 0;
    min-height: 0;
  }

  .bar {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 4px 12px;
    min-height: 56px;
    box-sizing: border-box;
    background: #1d1f24;
    border-bottom: 1px solid #3a3d45;
  }

  .state {
    color: #c7c9d1;
    font-size: 0.9rem;
  }

  .spacer {
    flex: 1;
  }

  .side {
    width: 320px;
    flex: none;
    box-sizing: border-box;
    padding: 12px;
    overflow-y: auto;
    display: grid;
    align-content: start;
    gap: 16px;
    border-left: 1px solid #3a3d45;
    background: #1b1c20;
  }

  button {
    min-width: 48px;
    min-height: 48px;
    padding: 0 14px;
    border: 1px solid #3a3d45;
    border-radius: 10px;
    background: #23252b;
    color: inherit;
    font: inherit;
    cursor: pointer;
    touch-action: manipulation;
  }

  button.primary {
    background: #3b4252;
    border-color: #7aa2f7;
  }

  button:disabled {
    opacity: 0.45;
    cursor: default;
  }

  .backdrop {
    position: fixed;
    inset: 0;
    display: grid;
    place-items: center;
    background: rgb(0 0 0 / 0.6);
    z-index: 10;
  }

  .dialog {
    width: min(420px, calc(100vw - 32px));
    padding: 20px;
    border: 1px solid #3a3d45;
    border-radius: 14px;
    background: #1d1f24;
  }

  .dialog h2 {
    margin: 0 0 8px;
    font-size: 1.1rem;
  }

  .dialog p {
    margin: 0 0 16px;
    color: #c7c9d1;
  }

  .buttons {
    display: flex;
    gap: 8px;
    justify-content: flex-end;
    flex-wrap: wrap;
  }
</style>
