// Per-widget local state in the browser's localStorage (WebView2 keeps it between runs).
// It never goes in the deck file. Every access is guarded: storage can be missing, full or
// blocked, and then the widget just runs from its default.

export function readState(key: string): string | null {
  try {
    return localStorage.getItem(key)
  } catch {
    return null
  }
}

export function writeState(key: string, value: string): boolean {
  try {
    localStorage.setItem(key, value)
    return true
  } catch {
    return false
  }
}
