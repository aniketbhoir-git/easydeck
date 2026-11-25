<script lang="ts">
  import type { ValidationError } from '../../lib/api'
  import {
    LIMITS,
    keyFromEvent,
    macroHint,
    parseKeys,
    setMacroNumber,
    setMacroToggle,
    type MacroAction,
  } from './actions'

  // Edits one key macro. Keys are typed ("1 2 ctrl+q") or recorded from the keyboard. It emits
  // a new action and never sends a key to the system: while recording, the page only listens.
  let {
    action,
    errors = {},
    idPrefix,
    onchange,
  }: {
    action: MacroAction
    errors?: Record<string, ValidationError[]>
    idPrefix: string
    onchange: (next: MacroAction) => void
  } = $props()

  let text = $state('')
  let recording = $state(false)
  const hint = $derived(macroHint(action))

  // Keep the typed text unless the keys changed from elsewhere (record, discard).
  $effect(() => {
    if (JSON.stringify(parseKeys(text)) !== JSON.stringify(action.keys)) text = action.keys.join(' ')
  })

  function setKeys(keys: string[]) {
    onchange({ ...action, keys })
  }

  // The listener sits on the window only while recording, so Tab, typing in a field and the
  // rest of the page work normally the rest of the time. Esc stops and is not recorded.
  function record(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      e.preventDefault()
      e.stopPropagation()
      recording = false
      return
    }
    e.preventDefault()
    e.stopPropagation()
    if (e.repeat) return
    const name = keyFromEvent(e)
    if (name === null) return
    setKeys([...action.keys, name])
  }

  $effect(() => {
    if (!recording) return
    window.addEventListener('keydown', record, true)
    return () => window.removeEventListener('keydown', record, true)
  })

  function num(e: Event & { currentTarget: HTMLInputElement }) {
    const v = e.currentTarget.value
    return v === '' ? undefined : Number(v)
  }
</script>

<div class="form" data-form="macro">
  <label>
    Keys, in order
    <input
      type="text"
      data-field="keys"
      placeholder="1 2 ctrl+q"
      value={text}
      oninput={(e) => {
        text = e.currentTarget.value
        setKeys(parseKeys(text))
      }}
    />
  </label>
  <div class="row">
    <button
      type="button"
      data-action="record-keys"
      aria-pressed={recording}
      onblur={() => (recording = false)}
      onclick={() => (recording = !recording)}
    >
      {recording ? 'Stop recording' : 'Record keys'}
    </button>
    <button type="button" data-action="clear-keys" onclick={() => setKeys([])}>Clear</button>
  </div>
  {#if recording}<p class="hint">Press the keys now. They are written down, not sent.</p>{/if}
  {#each errors.keys ?? [] as e, i (i)}
    <p class="field-error" role="alert" data-testid={`field-error-${idPrefix}-keys`}>
      {e.message} <span>{e.fix}</span>
    </p>
  {/each}
  {#if hint && !(errors.keys ?? []).length}<p class="hint">{hint}</p>{/if}

  {#each [['cycleMs', 'Repeat every (ms, 0 = once)'], ['keyGapMs', 'Pause between keys (ms)'], ['holdMs', 'Key held down (ms)']] as const as [name, title] (name)}
    <label>
      {title}
      <input
        type="number"
        inputmode="numeric"
        data-field={name}
        min={LIMITS[name].min}
        max={LIMITS[name].max}
        step="1"
        value={action[name] ?? ''}
        oninput={(e) => onchange(setMacroNumber(action, name, num(e)))}
      />
    </label>
    {#each errors[name] ?? [] as e, i (i)}
      <p class="field-error" role="alert" data-testid={`field-error-${idPrefix}-${name}`}>
        {e.message} <span>{e.fix}</span>
      </p>
    {/each}
  {/each}

  <label class="check">
    <input
      type="checkbox"
      data-field="toggle"
      checked={action.toggle ?? false}
      onchange={(e) => onchange(setMacroToggle(action, e.currentTarget.checked))}
    />
    One tap starts the loop, the next tap stops it
  </label>
  <p class="note" data-testid="risk-note">
    This tile can send keys. It stays inert until you approve it.
  </p>
</div>
