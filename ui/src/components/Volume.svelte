<script lang="ts">
  import { onMount } from 'svelte'
  import type { Widget } from '../lib/api'
  import {
    createLatestThrottle,
    isVertical,
    levelFromPointer,
    onVolume,
    volumeGet,
    volumeMute,
    volumeSet,
  } from '../lib/volume'

  type VolumeData = Extract<Widget, { type: 'volume' }>
  let { widget, w = 1, h = 1 }: { widget: VolumeData; w?: number; h?: number } = $props()

  const vertical = $derived(isVertical(w, h))
  const name = $derived(widget.label ?? 'Volume')

  // null level means no data: the volume could not be read.
  let level = $state<number | null>(null)
  let muted = $state(false)
  let dragging = $state(false)
  let track: HTMLDivElement | undefined = $state()

  const sender = createLatestThrottle((v: number) => {
    volumeSet(v).catch(() => (level = null))
  })

  onMount(() => {
    let stopped = false
    let unlisten: (() => void) | undefined
    volumeGet()
      .then((v) => {
        if (!dragging) {
          level = v.level
          muted = v.muted
        }
      })
      .catch(() => (level = null))
    onVolume((v) => {
      muted = v.muted
      if (!dragging) level = v.level
    })
      .then((u) => (stopped ? u() : (unlisten = u)))
      .catch(() => {})
    return () => {
      stopped = true
      unlisten?.()
      sender.cancel()
    }
  })

  function move(e: PointerEvent) {
    if (!track) return
    const r = track.getBoundingClientRect()
    const v = vertical
      ? levelFromPointer(e.clientY, r.top, r.height, true)
      : levelFromPointer(e.clientX, r.left, r.width, false)
    level = v
    sender.push(v)
  }

  function down(e: PointerEvent) {
    if (e.pointerType === 'mouse' && e.button !== 0) return
    dragging = true
    track?.setPointerCapture?.(e.pointerId)
    move(e)
  }

  function up() {
    dragging = false
  }

  function key(e: KeyboardEvent) {
    const step = { ArrowRight: 5, ArrowUp: 5, ArrowLeft: -5, ArrowDown: -5 }[e.key]
    if (step === undefined || level === null) return
    e.preventDefault()
    level = Math.min(100, Math.max(0, level + step))
    sender.push(level)
  }

  function toggleMute() {
    if (level === null) return
    muted = !muted
    volumeMute(muted).catch(() => (level = null))
  }
</script>

<div
  class="volume"
  class:vertical
  class:muted
  data-testid="volume"
  data-state={level === null ? 'nodata' : muted ? 'muted' : 'on'}
>
  <button
    type="button"
    class="mute"
    data-testid="volume-mute"
    aria-pressed={muted}
    aria-label={muted ? `Unmute ${name}` : `Mute ${name}`}
    disabled={level === null}
    onclick={toggleMute}
  >
    {muted ? '🔇' : '🔊'}
  </button>
  <div
    class="track"
    bind:this={track}
    data-testid="volume-track"
    role="slider"
    tabindex="0"
    aria-label={name}
    aria-orientation={vertical ? 'vertical' : 'horizontal'}
    aria-valuemin="0"
    aria-valuemax="100"
    aria-valuenow={level ?? undefined}
    aria-valuetext={level === null ? 'no data' : muted ? `${level}%, muted` : `${level}%`}
    aria-disabled={level === null}
    onpointerdown={down}
    onpointermove={(e) => dragging && move(e)}
    onpointerup={up}
    onpointercancel={up}
    onkeydown={key}
  >
    {#if level !== null}
      <div class="fill" style:--level={level}></div>
    {/if}
    <span class="reading" data-testid="volume-level">
      {level === null ? 'no data' : muted ? 'muted' : `${level}%`}
    </span>
  </div>
</div>

<style>
  .volume {
    display: flex;
    flex-direction: row;
    align-items: stretch;
    gap: 0.5rem;
    width: 100%;
    height: 100%;
    min-height: 0;
  }
  .volume.vertical {
    flex-direction: column-reverse;
  }
  .mute {
    flex: none;
    min-width: 48px;
    min-height: 48px;
    border: 1px solid #3a3d45;
    border-radius: 10px;
    background: #2d3038;
    color: inherit;
    font-size: 1.4rem;
    cursor: pointer;
    touch-action: manipulation;
  }
  .muted .mute {
    background: #5a2d33;
    border-color: #c0626b;
  }
  .mute:disabled {
    opacity: 0.4;
    cursor: default;
  }
  .track {
    position: relative;
    flex: 1;
    min-width: 48px;
    min-height: 48px;
    overflow: hidden;
    border-radius: 10px;
    background: #1b1d22;
    display: flex;
    align-items: center;
    justify-content: center;
    touch-action: none;
    cursor: pointer;
  }
  .track:focus-visible {
    outline: 2px solid #7aa2f7;
  }
  .fill {
    position: absolute;
    inset: 0;
    background: #7aa2f7;
    transform-origin: left center;
    transform: scaleX(calc(var(--level) / 100));
    pointer-events: none;
  }
  .vertical .fill {
    transform-origin: center bottom;
    transform: scaleY(calc(var(--level) / 100));
  }
  .muted .fill {
    background: #6b7280;
  }
  .reading {
    position: relative;
    font-weight: 600;
    font-variant-numeric: tabular-nums;
    text-shadow: 0 0 4px #000;
    pointer-events: none;
  }
</style>
