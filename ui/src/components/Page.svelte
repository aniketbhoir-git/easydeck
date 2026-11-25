<script lang="ts">
  import { placedWidgets, type Page } from '../lib/api'
  import Tile from './Tile.svelte'

  let {
    page,
    onpress,
    editing = false,
    selectedId = null,
    onselect,
  }: {
    page: Page
    onpress: (id: string) => void
    editing?: boolean
    selectedId?: string | null
    onselect?: (id: string) => void
  } = $props()
</script>

<section
  class="page"
  data-page-id={page.id}
  style:grid-template-columns="repeat({page.grid.columns}, minmax(0, 1fr))"
  style:grid-template-rows="repeat({page.grid.rows}, minmax(0, 1fr))"
>
  {#each placedWidgets(page) as widget (widget.id)}
    <Tile {widget} {onpress} {editing} selected={editing && widget.id === selectedId} {onselect} />
  {/each}
</section>

<style>
  .page {
    display: grid;
    gap: 12px;
    flex: 1;
    min-height: 0;
  }
</style>
