import type { Language } from './types'

const THRESHOLDS: readonly {
  limit: number
  divisor: number
  unit: Intl.RelativeTimeFormatUnit
}[] = [
  { limit: 60, divisor: 1, unit: 'second' },
  { limit: 3600, divisor: 60, unit: 'minute' },
  { limit: 86400, divisor: 3600, unit: 'hour' },
  { limit: 604800, divisor: 86400, unit: 'day' },
  { limit: 2629800, divisor: 604800, unit: 'week' },
  { limit: 31557600, divisor: 2629800, unit: 'month' },
]

/** Past timestamps read as "2 分钟前" / "2 minutes ago". */
export function formatRelative(
  iso: string,
  language: Language,
  now = Date.now(),
): string {
  const timestamp = new Date(iso).getTime()
  if (Number.isNaN(timestamp)) return '—'

  const locale = language === 'zh-CN' ? 'zh-CN' : 'en'
  const rtf = new Intl.RelativeTimeFormat(locale, { numeric: 'auto' })
  const deltaSeconds = (timestamp - now) / 1000
  const abs = Math.abs(deltaSeconds)

  if (abs < 45) return rtf.format(0, 'second')

  for (const { limit, divisor, unit } of THRESHOLDS) {
    if (abs < limit) {
      return rtf.format(Math.round(deltaSeconds / divisor), unit)
    }
  }
  return rtf.format(Math.round(deltaSeconds / 31557600), 'year')
}

export function formatDateTime(iso: string, language: Language): string {
  const date = new Date(iso)
  if (Number.isNaN(date.getTime())) return '—'
  return new Intl.DateTimeFormat(language === 'zh-CN' ? 'zh-CN' : 'en', {
    dateStyle: 'medium',
    timeStyle: 'short',
  }).format(date)
}

export function formatBytes(bytes: number): string {
  if (!Number.isFinite(bytes) || bytes < 0) return '—'
  if (bytes < 1024) return `${bytes} B`
  const kb = bytes / 1024
  if (kb < 1024) return `${kb.toFixed(kb < 10 ? 1 : 0)} KB`
  const mb = kb / 1024
  return `${mb.toFixed(mb < 10 ? 1 : 0)} MB`
}

export function shortHash(hash: string, length = 7): string {
  return hash.slice(0, length)
}

/** Shorten a home-directory path for display without changing the value. */
export function displayPath(path: string): string {
  return path.replace(/^\/Users\/[^/]+/, '~').replace(/^[A-Z]:\\Users\\[^\\]+/, '%USERPROFILE%')
}
