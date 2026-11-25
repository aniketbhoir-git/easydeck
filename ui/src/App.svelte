<script lang="ts">
  import {
    loadDeck,
    validateDeck,
    riskReview,
    approveRiskItems,
    type Deck as DeckModel,
    type RiskReview as RiskReviewModel,
  } from './lib/api'
  import { lockedTiles } from './lib/risk'
  import RiskReview from './components/RiskReview.svelte'
  import Deck from './components/Deck.svelte'
  import EditorShell from './editor/EditorShell.svelte'
  import { Draft } from './editor/draft.svelte'

  let deck = $state<DeckModel | null>(null)
  let error = $state<string | null>(null)
  // Edit mode is on while a draft exists. The run-mode deck stays as loaded.
  let draft = $state<Draft | null>(null)
  let pageId = $state<string | null>(null)

  // Risk items and their approval (SPEC 4.3). Until the review has loaded, nothing is locked in
  // the UI; the backend gate does not depend on it.
  let review = $state<RiskReviewModel | null>(null)
  let reviewOpen = $state(false)
  let notice = $state<string | null>(null)
  const locked = $derived(lockedTiles(review))

  loadDeck().then(
    (d) => (deck = d),
    (e) => (error = e?.message ?? String(e)),
  )
  riskReview().then(
    (r) => {
      review = r
      notice = r.notice
    },
    (e) => console.error('risk_review failed', e),
  )

  // Fetches the current state first, so the screen never shows a stale list.
  async function openReview() {
    try {
      review = await riskReview()
    } catch (e) {
      console.error('risk_review failed', e)
    }
    reviewOpen = true
  }

  async function approve(hashes: string[]) {
    review = await approveRiskItems(hashes)
  }

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
  {#if notice}
    <p class="notice" role="status" data-testid="ledger-notice">
      {notice}
      <button type="button" onclick={() => (notice = null)}>Dismiss</button>
    </p>
  {/if}
  <EditorShell {draft} {pageId} onexit={() => (draft = null)}>
    <Deck
      deck={draft ? draft.deck : deck}
      editing={!!draft}
      selectedId={draft?.selectedId ?? null}
      selectedPageId={draft?.selectedPageId ?? null}
      onselect={(id, pid) => draft?.select(id, pid)}
      onrequestedit={edit}
      {locked}
      onreview={openReview}
      bind:pageId
    />
  </EditorShell>
  {#if reviewOpen && review}
    <RiskReview {review} onapprove={approve} onclose={() => (reviewOpen = false)} />
  {/if}
{:else if error}
  <p class="status">Could not load the deck: {error}</p>
{:else}
  <p class="status">Loading</p>
{/if}

<style>
  .notice {
    position: fixed;
    top: 8px;
    left: 50%;
    transform: translateX(-50%);
    z-index: 20;
    display: flex;
    align-items: center;
    gap: 12px;
    margin: 0;
    padding: 8px 12px;
    border: 1px solid #f7768e;
    border-radius: 12px;
    background: #1d1f24;
  }

  .notice button {
    min-width: 48px;
    min-height: 48px;
    border: 1px solid #3a3d45;
    border-radius: 12px;
    background: #23252b;
    color: inherit;
    font: inherit;
    cursor: pointer;
  }
</style>
