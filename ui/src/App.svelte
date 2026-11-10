<script lang="ts">
  import { loadDeck, type Deck as DeckModel } from './lib/api'
  import Deck from './components/Deck.svelte'

  let deck = $state<DeckModel | null>(null)
  let error = $state<string | null>(null)

  loadDeck().then(
    (d) => (deck = d),
    (e) => (error = e?.message ?? String(e)),
  )
</script>

{#if deck}
  <Deck {deck} />
{:else if error}
  <p class="status">Could not load the deck: {error}</p>
{:else}
  <p class="status">Loading</p>
{/if}
