import { listen, type UnlistenFn } from '@tauri-apps/api/event'

/** One reading from a source: metric name to a number, or null when it is not available. */
export type SourceValue = Record<string, number | null>

/** The Tauri event the app emits for a deck source id. */
export function sourceEventName(sourceId: string): string {
  return `source:${sourceId}`
}

/** Calls `handler` for each reading of the source. Returns a function that stops listening. */
export function onSourceValue(
  sourceId: string,
  handler: (value: SourceValue) => void,
): Promise<UnlistenFn> {
  return listen<SourceValue>(sourceEventName(sourceId), (event) => handler(event.payload))
}
