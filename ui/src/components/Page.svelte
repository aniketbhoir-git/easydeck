<script lang="ts">
  import type { Page } from '../lib/api'
  import Tile from './Tile.svelte'

  let { page, onpress }: { page: Page; onpress: (id: string) => void } = $props()
</script>

<section
  class="page"
  data-page-id={page.id}
  style:grid-template-columns="repeat({page.grid.columns}, minmax(0, 1fr))"
  style:grid-template-rows="repeat({page.grid.rows}, minmax(0, 1fr))"
>
  {#each page.widgets as widget (widget.id)}
    <Tile {widget} {onpress} />
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
