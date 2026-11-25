<script lang="ts">
  import { onMount } from 'svelte'
  import type { Widget } from '../lib/api'
  import {
    canDo,
    mediaCommand,
    safeThumbnail,
    visibleControls,
    watchMedia,
    type MediaCommand,
    type MediaState,
  } from '../lib/media'

  type MediaData = Extract<Widget, { type: 'media' }>
  let { widget }: { widget: MediaData } = $props()

  const controls = $derived(visibleControls(widget.controls))

  // null means the session could not be read: shown like "nothing playing", buttons off.
  let media = $state<MediaState | null>(null)

  const none = $derived(!media || media.status === 'none')
  const thumbnail = $derived(safeThumbnail(media?.thumbnail))
  const playing = $derived(media?.status === 'playing')

  onMount(() => watchMedia((m) => (media = m)))

  const glyph: Record<MediaCommand, string> = { previous: '⏮', play_pause: '⏯', next: '⏭' }
  function label(c: MediaCommand): string {
    if (c === 'previous') return 'Previous track'
    if (c === 'next') return 'Next track'
    return playing ? 'Pause' : 'Play'
  }
  function icon(c: MediaCommand): string {
    return c === 'play_pause' ? (playing ? '⏸' : '▶') : glyph[c]
  }
</script>

<div class="media" data-testid="media" data-state={none ? 'none' : (media?.status ?? 'none')}>
  <div class="now">
    {#if thumbnail}
      <img class="art" data-testid="media-thumbnail" src={thumbnail} alt="" />
    {/if}
    <div class="text">
      {#if none}
        <span class="title dim" data-testid="media-title">Nothing playing</span>
      {:else}
        <span class="title" data-testid="media-title">{media?.title || 'Unknown title'}</span>
        {#if media?.artist}<span class="artist" data-testid="media-artist">{media.artist}</span>{/if}
      {/if}
    </div>
  </div>
  {#if controls.length}
    <div class="buttons">
      {#each controls as c (c)}
        <button
          type="button"
          class="btn"
          data-testid="media-{c}"
          aria-label={label(c)}
          disabled={!canDo(media, c)}
          onclick={() => mediaCommand(c).catch(() => {})}
        >
          {icon(c)}
        </button>
      {/each}
    </div>
  {/if}
</div>

<style>
  .media {
    display: flex;
    flex-direction: column;
    justify-content: space-between;
    gap: 0.5rem;
    width: 100%;
    height: 100%;
    min-height: 0;
  }
  .now {
    flex: 1;
    min-height: 0;
    display: flex;
    align-items: center;
    gap: 0.6rem;
  }
  .art {
    flex: none;
    width: 56px;
    height: 56px;
    border-radius: 8px;
    object-fit: cover;
    background: #1b1d22;
  }
  .text {
    min-width: 0;
    display: flex;
    flex-direction: column;
    text-align: left;
  }
  .title,
  .artist {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .title {
    font-weight: 600;
  }
  .artist,
  .dim {
    color: #9aa0ad;
  }
  .buttons {
    display: flex;
    gap: 0.5rem;
  }
  .btn {
    flex: 1;
    min-width: 48px;
    min-height: 48px;
    border: 1px solid #3a3d45;
    border-radius: 10px;
    background: #2d3038;
    color: inherit;
    font-size: 1.3rem;
    cursor: pointer;
    touch-action: manipulation;
  }
  .btn:active:not(:disabled) {
    background: #3b4252;
    border-color: #7aa2f7;
  }
  .btn:disabled {
    opacity: 0.4;
    cursor: default;
  }
</style>
