<script lang="ts">
  import type { Deck } from '../lib/api'
  import { runAction } from '../lib/api'
  import Page from './Page.svelte'

  let { deck }: { deck: Deck } = $props()

  let selected = $state<string | null>(null)
  const current = $derived(
    deck.pages.find((p) => p.id === selected) ??
      deck.pages.find((p) => p.id === deck.settings?.homePage) ??
      deck.pages[0],
  )

  async function press(widgetId: string) {
    try {
      await runAction(widgetId)
    } catch (err) {
      console.error('run_action failed', err)
    }
  }
</script>

<div class="deck">
  {#if deck.pages.length > 1}
    <nav class="switcher">
      {#each deck.pages as p (p.id)}
        <button
          type="button"
          class:active={p.id === current?.id}
          onclick={() => (selected = p.id)}>{p.title}</button
        >
      {/each}
    </nav>
  {/if}
  {#if current}
    <Page page={current} onpress={press} />
  {/if}
</div>

<style>
  .deck {
    display: flex;
    flex-direction: column;
    gap: 12px;
    height: 100vh;
    box-sizing: border-box;
    padding: 12px;
  }

  .switcher {
    display: flex;
    gap: 8px;
  }

  .switcher button {
    min-width: 48px;
    min-height: 48px;
    padding: 0 1rem;
    border: 1px solid #3a3d45;
    border-radius: 12px;
    background: #1d1f24;
    color: inherit;
    font: inherit;
    cursor: pointer;
    user-select: none;
    touch-action: manipulation;
  }

  .switcher button.active {
    background: #3b4252;
    border-color: #7aa2f7;
  }
</style>
