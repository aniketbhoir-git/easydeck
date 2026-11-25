<script lang="ts">
  import { loadDeck, validateDeck, type Deck as DeckModel } from './lib/api'
  import Deck from './components/Deck.svelte'
  import EditorShell from './editor/EditorShell.svelte'
  import { Draft } from './editor/draft.svelte'

  let deck = $state<DeckModel | null>(null)
  let error = $state<string | null>(null)
  // Edit mode is on while a draft exists. The run-mode deck stays as loaded.
  let draft = $state<Draft | null>(null)
  let pageId = $state<string | null>(null)

  loadDeck().then(
    (d) => (deck = d),
    (e) => (error = e?.message ?? String(e)),
  )

  // Enter edit mode with a long press on the deck background or with Ctrl+E.
  function edit() {
    if (deck && !draft) draft = new Draft(deck, validateDeck)
  }

  function onkeydown(e: KeyboardEvent) {
    if (e.ctrlKey && !e.shiftKey && !e.altKey && e.key.toLowerCase() === 'e') {
      e.preventDefault()
      edit()
    }
  }
</script>

<svelte:window {onkeydown} />

{#if deck}
  <EditorShell {draft} {pageId} onexit={() => (draft = null)}>
    <Deck
      deck={draft ? draft.deck : deck}
      editing={!!draft}
      selectedId={draft?.selectedId ?? null}
      selectedPageId={draft?.selectedPageId ?? null}
      onselect={(id, pid) => draft?.select(id, pid)}
      onrequestedit={edit}
      bind:pageId
    />
  </EditorShell>
{:else if error}
  <p class="status">Could not load the deck: {error}</p>
{:else}
  <p class="status">Loading</p>
{/if}
