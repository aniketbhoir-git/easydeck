<script lang="ts">
  import type { LaunchKind, ValidationError } from '../../lib/api'
  import {
    LAUNCH_KINDS,
    LIMITS,
    launchHint,
    requiredField,
    setLaunchField,
    setLaunchKind,
    type LaunchAction,
  } from './actions'

  // Edits one launch action. It emits a new action and never runs anything.
  let {
    action,
    errors = {},
    idPrefix,
    onchange,
  }: {
    action: LaunchAction
    errors?: Record<string, ValidationError[]>
    idPrefix: string
    onchange: (next: LaunchAction) => void
  } = $props()

  const field = $derived(requiredField(action.kind))
  const hint = $derived(launchHint(action))

  function num(e: Event & { currentTarget: HTMLInputElement }) {
    const v = e.currentTarget.value
    return v === '' ? undefined : Number(v)
  }
</script>

<div class="form" data-form="launch">
  <label>
    Kind
    <select
      data-field="kind"
      value={action.kind}
      onchange={(e) => onchange(setLaunchKind(action, e.currentTarget.value as LaunchKind))}
    >
      {#each LAUNCH_KINDS as k (k.kind)}<option value={k.kind}>{k.name}</option>{/each}
    </select>
  </label>

  {#if field === 'appId'}
    <label>
      Steam app id
      <input
        type="number"
        inputmode="numeric"
        data-field="appId"
        min={LIMITS.appId.min}
        max={LIMITS.appId.max}
        step="1"
        value={action.appId ?? ''}
        oninput={(e) => onchange(setLaunchField(action, 'appId', num(e)))}
      />
    </label>
  {:else if field === 'path'}
    <label>
      Program path
      <input
        type="text"
        data-field="path"
        placeholder="C:\Games\game.exe"
        value={action.path ?? ''}
        oninput={(e) => onchange(setLaunchField(action, 'path', e.currentTarget.value))}
      />
    </label>
    <label>
      Fallback (optional)
      <input
        type="text"
        data-field="fallback"
        value={action.fallback ?? ''}
        oninput={(e) => onchange(setLaunchField(action, 'fallback', e.currentTarget.value))}
      />
    </label>
  {:else}
    <label>
      {action.kind === 'url' ? 'Web link' : 'URI'}
      <input
        type="text"
        data-field="target"
        placeholder={action.kind === 'url' ? 'https://example.com' : 'steam://open/main'}
        value={action.target ?? ''}
        oninput={(e) => onchange(setLaunchField(action, 'target', e.currentTarget.value))}
      />
    </label>
  {/if}

  {#each errors[field] ?? [] as e, i (i)}
    <p class="field-error" role="alert" data-testid={`field-error-${idPrefix}-${field}`}>
      {e.message} <span>{e.fix}</span>
    </p>
  {/each}
  {#if hint}<p class="hint">{hint}</p>{/if}
  <p class="note" data-testid="risk-note">
    This tile can run a program. It stays inert until you approve it.
  </p>
</div>
