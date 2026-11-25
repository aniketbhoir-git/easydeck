// The editor's working copy of the deck: an in-memory draft that is validated after every
// change. Nothing here touches the disk. R4-3 adds move, resize, undo and save on top.
//
// The draft starts from what `load_deck` returns, which has every position filled in by
// `layout::resolve`. R4-3 must not save that: it needs the original deck file. See the note
// under R4-3 in docs/ISSUE-BACKLOG.md.
import type { Deck, Page, ValidationError, Widget } from '../lib/api'

export type WidgetType = Widget['type']

/** Widget types the palette offers. `embed` waits for the forms of R4-2. */
export const PALETTE_TYPES: { type: WidgetType; name: string }[] = [
  { type: 'launch', name: 'Launch' },
  { type: 'macro', name: 'Macro' },
  { type: 'toggle', name: 'Toggle' },
  { type: 'text', name: 'Text' },
  { type: 'counter', name: 'Counter' },
  { type: 'timer', name: 'Timer' },
  { type: 'volume', name: 'Volume' },
  { type: 'media', name: 'Media' },
  { type: 'gauge', name: 'Gauge' },
  { type: 'status', name: 'Status' },
]

/** The first free cell in reading order (row by row, left to right), or null when the page
 *  is full. It is the rule of `layout::resolve` in easydeck-core, applied to one new widget. */
export function firstFreeCell(page: Page): { x: number; y: number } | null {
  const { columns, rows } = page.grid
  const taken = new Set<string>()
  for (const w of page.widgets) {
    const p = w.position
    if (!p) continue
    for (let dy = 0; dy < (p.h ?? 1); dy++)
      for (let dx = 0; dx < (p.w ?? 1); dx++) taken.add(`${p.x + dx},${p.y + dy}`)
  }
  for (let y = 0; y < rows; y++)
    for (let x = 0; x < columns; x++) if (!taken.has(`${x},${y}`)) return { x, y }
  return null
}

/** A widget of `type` with only the fields it needs to exist. A type that needs an action or
 *  a source gets none: the draft is invalid until R4-2's forms fill it in. */
function newWidget(deck: Deck, id: string, type: WidgetType, name: string): Widget {
  const base = { id, label: name, position: undefined }
  const source = Object.keys(deck.sources ?? {})[0] ?? ''
  switch (type) {
    case 'text':
      return { ...base, type, kind: 'label', text: name } as Widget
    case 'timer':
      return { ...base, type, mode: 'stopwatch' } as Widget
    case 'gauge':
      return { ...base, type, source, metric: '' } as Widget
    case 'status':
      return { ...base, type, source } as Widget
    default:
      return { ...base, type } as Widget
  }
}

function uniqueId(page: Page, type: string): string {
  const used = new Set(page.widgets.map((w) => w.id))
  for (let n = 1; ; n++) if (!used.has(`${type}-${n}`)) return `${type}-${n}`
}

export class Draft {
  deck = $state<Deck>() as Deck
  errors = $state<ValidationError[]>([])
  selectedId = $state<string | null>(null)
  // Widget ids are unique per page, not per deck (core validate rule "duplicate-id").
  selectedPageId = $state<string | null>(null)
  /** A short message about the last action, for example "This page is full". */
  notice = $state<string | null>(null)
  validating = $state(false)

  #base: string
  #validate: (deck: Deck) => Promise<ValidationError[]>
  #seq = 0

  constructor(deck: Deck, validate: (deck: Deck) => Promise<ValidationError[]>) {
    this.deck = structuredClone($state.snapshot(deck)) as Deck
    this.#base = JSON.stringify(this.deck)
    this.#validate = validate
    void this.revalidate()
  }

  get dirty(): boolean {
    return JSON.stringify(this.deck) !== this.#base
  }

  /** A draft can be saved only when it has changes and the last validation found nothing. */
  get canSave(): boolean {
    return this.dirty && !this.validating && this.errors.length === 0
  }

  select(id: string | null, pageId: string | null = null) {
    this.selectedId = id
    this.selectedPageId = pageId
  }

  /** The widget `selectedId` names, with its page, or undefined. */
  get selected(): { widget: Widget; pageIndex: number; widgetIndex: number } | undefined {
    for (let pageIndex = 0; pageIndex < this.deck.pages.length; pageIndex++) {
      if (this.selectedPageId !== null && this.deck.pages[pageIndex].id !== this.selectedPageId) continue
      const widgetIndex = this.deck.pages[pageIndex].widgets.findIndex((w) => w.id === this.selectedId)
      if (widgetIndex >= 0)
        return { widget: this.deck.pages[pageIndex].widgets[widgetIndex], pageIndex, widgetIndex }
    }
    return undefined
  }

  /** Adds a widget in the first free cell of the page and selects it. Returns its id, or
   *  null when the page is full or does not exist. */
  addWidget(pageId: string | undefined, type: WidgetType): string | null {
    const page = this.deck.pages.find((p) => p.id === pageId)
    if (!page) return null
    const cell = firstFreeCell(page)
    if (!cell) {
      this.notice = `Page "${page.title}" is full`
      return null
    }
    const name = PALETTE_TYPES.find((t) => t.type === type)?.name ?? type
    const widget = newWidget(this.deck, uniqueId(page, type), type, name)
    widget.position = cell
    page.widgets.push(widget)
    this.notice = null
    this.selectedId = widget.id
    this.selectedPageId = page.id
    void this.revalidate()
    return widget.id
  }

  /** Sets fields on the selected widget. A value of `undefined` removes the field. This is
   *  how the forms of R4-2 write to the draft; they never touch the live deck. */
  patchSelected(patch: Record<string, unknown>) {
    const sel = this.selected
    if (!sel) return
    const target = sel.widget as unknown as Record<string, unknown>
    for (const [k, v] of Object.entries(patch)) {
      if (v === undefined) delete target[k]
      else target[k] = v
    }
    void this.revalidate()
  }

  /** Throws the changes away and returns to the state the draft started from. */
  discard() {
    this.deck = JSON.parse(this.#base) as Deck
    this.selectedId = null
    this.selectedPageId = null
    this.notice = null
    void this.revalidate()
  }

  /** Makes the current draft the new starting point. R4-3 calls it after a save. */
  markSaved() {
    this.#base = JSON.stringify(this.deck)
  }

  async revalidate() {
    const seq = ++this.#seq
    this.validating = true
    let errors: ValidationError[]
    try {
      errors = await this.#validate($state.snapshot(this.deck) as Deck)
    } catch (e) {
      const message = e instanceof Error ? e.message : String(e)
      errors = [{ path: '', rule: 'validator', message: `Could not validate: ${message}`, fix: 'try again' }]
    }
    if (seq !== this.#seq) return // a newer change superseded this result
    this.errors = errors
    this.validating = false
  }
}
