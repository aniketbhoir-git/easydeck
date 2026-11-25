<script lang="ts">
  import type { Deck } from '../lib/api'
  import { runAction } from '../lib/api'
  import { setContext } from 'svelte'
  import Page from './Page.svelte'
  import { provideSources } from '../lib/source-interval'
  import { createPressDetector } from '../lib/press'
  import { resolvePage } from '../lib/pages'

  // `editing` turns the deck into the editor's canvas: taps select tiles and run nothing.
  // A long press on the background (not on a tile or a page button) asks to enter edit mode.
  let {
    deck,
    editing = false,
    selectedId = null,
    onselect,
    onrequestedit,
    pageId = $bindable(null),
  }: {
    deck: Deck
    editing?: boolean
    selectedId?: string | null
    onselect?: (id: string) => void
    onrequestedit?: () => void
    pageId?: string | null
  } = $props()
  // Initial deck is enough: widgets read poll intervals once, when they mount.
  provideSources(deck.sources)

  setContext('deckName', deck.name)

  const current = $derived(resolvePage(deck, pageId))

  async function press(widgetId: string, hold = false) {
    try {
      await runAction(widgetId, hold)
    } catch (err) {
      console.error('run_action failed', err)
    }
  }

  // Background long press: the deck frame, the page switcher bar or the gaps of the grid.
  function holdBackground(node: HTMLElement) {
    const detector = createPressDetector(() => ({ onHold: editing ? undefined : onrequestedit }))
    const onBackground = (t: EventTarget | null) =>
      t instanceof HTMLElement && (t === node || t.matches('.switcher, .page'))
    const down = (e: PointerEvent) => {
      if (onBackground(e.target)) detector.down(e)
    }
    const move = (e: PointerEvent) => detector.move(e)
    const up = (e: PointerEvent) => detector.up(e)
    node.addEventListener('pointerdown', down)
    node.addEventListener('pointermove', move)
    node.addEventListener('pointerup', up)
    node.addEventListener('pointercancel', detector.cancel)
    return {
      destroy() {
        detector.cancel()
        node.removeEventListener('pointerdown', down)
        node.removeEventListener('pointermove', move)
        node.removeEventListener('pointerup', up)
        node.removeEventListener('pointercancel', detector.cancel)
      },
    }
  }
</script>

<div class="deck" class:editing use:holdBackground>
  {#if deck.pages.length > 1}
    <nav class="switcher">
      {#each deck.pages as p (p.id)}
        <button
          type="button"
          class:active={p.id === current?.id}
          onclick={() => (pageId = p.id)}>{p.title}</button
        >
      {/each}
    </nav>
  {/if}
  {#if current}
    <Page page={current} onpress={press} {editing} {selectedId} {onselect} />
  {/if}
</div>

<style>
  .deck {
    display: flex;
    flex-direction: column;
    gap: 12px;
    height: var(--deck-height, 100vh);
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
