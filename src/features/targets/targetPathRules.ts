import type { Target } from '@/lib/types'

export const SLUG_PATTERN = /^[a-z0-9]+(?:-[a-z0-9]+)*$/
export const ABSOLUTE_PATH = /^(?:\/|[A-Za-z]:[\\/]|\\\\)/

export function normalizeDir(value: string): string {
  return value.trim().replace(/[\\/]+$/, '')
}

export function foldDir(value: string): string {
  return normalizeDir(value).replace(/\\/g, '/').toLowerCase()
}

export function isInsideLibrary(dir: string, libraryPath: string): boolean {
  if (!dir || !libraryPath) return false
  const folded = foldDir(dir)
  const library = foldDir(libraryPath)
  return folded === library || folded.startsWith(`${library}/`)
}

export function isDuplicateDir(
  dir: string,
  targets: readonly Target[],
  currentId?: string,
): boolean {
  if (!dir) return false
  const folded = foldDir(dir)
  return targets.some(
    (item) => item.id !== currentId && foldDir(item.skillsDir) === folded,
  )
}
