// Pure clock formatting. The locale comes from the system (undefined locale) unless a test
// passes one. Nothing here touches the network.
export interface ClockOptions {
  /** true: 12-hour with AM/PM. false: 24-hour. undefined: the locale's own choice. */
  hour12?: boolean
  /** Show seconds. Defaults to false. */
  seconds?: boolean
  /** BCP 47 tag; undefined means the system locale. */
  locale?: string
  /** IANA zone; undefined means the system zone. Used by tests. */
  timeZone?: string
}

export interface ClockText {
  time: string
  date: string
}

export function formatClock(now: Date, options: ClockOptions = {}): ClockText {
  const { hour12, seconds = false, locale, timeZone } = options
  const time = new Intl.DateTimeFormat(locale, {
    hour: 'numeric',
    minute: '2-digit',
    ...(seconds ? { second: '2-digit' } : {}),
    ...(hour12 === undefined ? {} : { hourCycle: hour12 ? 'h12' : 'h23' }),
    timeZone,
  } as Intl.DateTimeFormatOptions).format(now)
  const date = new Intl.DateTimeFormat(locale, {
    weekday: 'short',
    month: 'short',
    day: 'numeric',
    timeZone,
  }).format(now)
  return { time, date }
}

/** Milliseconds until the next whole second, so a tick lands on the boundary. */
export function msToNextSecond(now: number): number {
  return 1000 - (now % 1000)
}
