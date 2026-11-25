<script lang="ts">
  import type { Action, Widget } from '../../lib/api'
  import type { Draft } from '../draft.svelte'
  import LaunchForm from './LaunchForm.svelte'
  import MacroForm from './MacroForm.svelte'
  import { fieldErrors, newLaunch, newMacro, type LaunchAction, type MacroAction } from './actions'

  // The form of the selected tile: label, icon and its actions. It edits the draft only.
  let { draft, widget, prefix }: { draft: Draft; widget: Widget; prefix: string } = $props()

  type Slot = { key: 'action' | 'holdAction' | 'onAction' | 'offAction'; title: string; optional: boolean }
  const slots = $derived<Slot[]>(
    widget.type === 'launch' || widget.type === 'macro'
      ? [
          { key: 'action', title: 'Tap action', optional: false },
          { key: 'holdAction', title: 'Long-press action', optional: true },
        ]
      : widget.type === 'toggle'
        ? [
            { key: 'onAction', title: 'When switched on', optional: false },
            { key: 'offAction', title: 'When switched off', optional: true },
          ]
        : [],
  )
  // A launch tile holds launch actions, a macro tile key macros; a toggle takes either.
  const choices = $derived(
    widget.type === 'launch'
      ? ['launch']
      : widget.type === 'macro'
        ? ['key_macro']
        : ['launch', 'key_macro'],
  )

  const actionOf = (key: Slot['key']) => (widget as unknown as Record<string, Action | undefined>)[key]
  const set = (key: Slot['key'], a: Action | undefined) => draft.patchSelected({ [key]: a })

  function create(key: Slot['key'], type: string) {
    set(key, type === 'launch' ? newLaunch() : newMacro())
  }
</script>

<div class="widget-form" data-testid="widget-form">
  <label>
    Label
    <input
      type="text"
      data-field="label"
      maxlength="60"
      value={widget.label ?? ''}
      oninput={(e) => draft.patchSelected({ label: e.currentTarget.value || undefined })}
    />
  </label>
  <label>
    Icon (path or https link)
    <input
      type="text"
      data-field="icon"
      value={widget.icon ?? ''}
      oninput={(e) => draft.patchSelected({ icon: e.currentTarget.value || undefined })}
    />
  </label>

  {#each slots as slot (slot.key)}
    {@const a = actionOf(slot.key)}
    {@const errs = fieldErrors(draft.errors, `${prefix}/${slot.key}`)}
    <fieldset data-slot={slot.key}>
      <legend>{slot.title}</legend>
      {#if a === undefined}
        {#each choices as type (type)}
          <button type="button" data-action={`add-${slot.key}`} onclick={() => create(slot.key, type)}>
            {choices.length > 1 ? (type === 'launch' ? 'Add launch' : 'Add key macro') : 'Add action'}
          </button>
        {/each}
      {:else}
        {#if choices.length > 1 && (a.type === 'launch' || a.type === 'key_macro')}
          <label>
            Type
            <select
              data-field="type"
              value={a.type}
              onchange={(e) => create(slot.key, e.currentTarget.value)}
            >
              <option value="launch">Launch</option>
              <option value="key_macro">Key macro</option>
            </select>
          </label>
        {/if}
        {#if a.type === 'launch'}
          <LaunchForm
            action={a as LaunchAction}
            errors={errs}
            idPrefix={slot.key}
            onchange={(next) => set(slot.key, next)}
          />
        {:else if a.type === 'key_macro'}
          <MacroForm
            action={a as MacroAction}
            errors={errs}
            idPrefix={slot.key}
            onchange={(next) => set(slot.key, next)}
          />
        {:else}
          <p class="hint">This action is a {a.type}. Edit it in the deck file.</p>
        {/if}
        {#each errs._ ?? [] as e, i (i)}
          <p class="field-error" role="alert">{e.message} <span>{e.fix}</span></p>
        {/each}
        {#if slot.optional}
          <button type="button" data-action={`remove-${slot.key}`} onclick={() => set(slot.key, undefined)}>
            Remove
          </button>
        {/if}
      {/if}
    </fieldset>
  {/each}
</div>

<style>
  .widget-form,
  .widget-form :global(.form) {
    display: grid;
    gap: 8px;
    margin: 0 0 12px;
  }
  .widget-form :global(label) {
    display: grid;
    gap: 4px;
    color: #9aa0ae;
    font-size: 0.8rem;
  }
  .widget-form :global(.check) {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .widget-form :global(input[type='text']),
  .widget-form :global(input[type='number']),
  .widget-form :global(select) {
    min-height: 48px;
    padding: 0 10px;
    border: 1px solid #3a4052;
    border-radius: 8px;
    background: #1b1f2b;
    color: #e6e8ee;
    font: inherit;
  }
  .widget-form :global(input[type='checkbox']) {
    width: 24px;
    height: 24px;
  }
  .widget-form :global(button) {
    min-height: 48px;
    min-width: 48px;
    padding: 0 12px;
    border: 1px solid #3a4052;
    border-radius: 8px;
    background: #262b3a;
    color: #e6e8ee;
    font: inherit;
  }
  .widget-form :global(.row) {
    display: flex;
    gap: 8px;
  }
  .widget-form :global(.hint),
  .widget-form :global(.note) {
    margin: 0;
    color: #9aa0ae;
    font-size: 0.8rem;
  }
  .widget-form :global(.field-error) {
    margin: 0;
    padding: 6px 8px;
    border: 1px solid #f7768e;
    border-radius: 8px;
    background: #2b1d21;
    font-size: 0.8rem;
  }
  .widget-form :global(.field-error span) {
    display: block;
    color: #c7c9d1;
  }
  fieldset {
    display: grid;
    gap: 8px;
    margin: 0;
    padding: 8px;
    border: 1px solid #3a4052;
    border-radius: 8px;
  }
  legend {
    padding: 0 4px;
    color: #9aa0ae;
    font-size: 0.8rem;
  }
</style>
