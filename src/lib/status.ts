import type {
  Install,
  InstallState,
  Language,
  LinkMode,
  Target,
  Tone,
} from './types'

/**
 * Single source of truth for state → copy/colour. Components must not hardcode
 * "副本过期" or a status colour anywhere else.
 */

export type DisplayState = InstallState | 'none' | 'ignored'

interface Copy {
  zh: string
  en: string
}

const STATE_COPY: Record<Exclude<DisplayState, 'ok'>, Copy> = {
  pending: { zh: '待同步', en: 'Pending' },
  stale: { zh: '副本过期', en: 'Stale' },
  missing: { zh: '缺失', en: 'Missing' },
  conflict: { zh: '冲突', en: 'Conflict' },
  error: { zh: '异常', en: 'Error' },
  none: { zh: '未链接', en: 'Not linked' },
  ignored: { zh: '已忽略', en: 'Ignored' },
}

const OK_COPY: Record<LinkMode, Copy> = {
  symlink: { zh: '已链接', en: 'Linked' },
  copy: { zh: '副本', en: 'Copy' },
}

const TONE_BY_STATE: Record<DisplayState, Tone> = {
  ok: 'ok',
  pending: 'warn',
  stale: 'warn',
  missing: 'warn',
  conflict: 'danger',
  error: 'danger',
  none: 'muted',
  ignored: 'muted',
}

const RANK: Record<DisplayState, number> = {
  ok: 1,
  pending: 2,
  stale: 3,
  missing: 4,
  error: 5,
  conflict: 6,
  none: 0,
  ignored: 0,
}

export const TONE_TEXT: Record<Tone, string> = {
  ok: 'text-ok',
  warn: 'text-warn',
  danger: 'text-danger',
  muted: 'text-faint',
}

export const TONE_DOT: Record<Tone, string> = {
  ok: 'bg-ok',
  warn: 'bg-warn',
  danger: 'bg-danger',
  muted: 'bg-faint',
}

export const TONE_PIP: Record<Tone, string> = {
  ok: 'bg-ok-pip',
  warn: 'bg-warn',
  danger: 'bg-danger',
  muted: 'bg-pip-off',
}

export const TONE_SOFT: Record<Tone, string> = {
  ok: 'border-ok-soft bg-ok-soft',
  warn: 'border-warn-soft bg-warn-soft',
  danger: 'border-danger-soft bg-danger-soft',
  muted: 'border-line bg-panel-2',
}

export function stateCopy(
  state: DisplayState,
  mode: LinkMode,
  language: Language,
): string {
  if (state === 'ok') return OK_COPY[mode][language === 'zh-CN' ? 'zh' : 'en']
  return STATE_COPY[state][language === 'zh-CN' ? 'zh' : 'en']
}

/** Status for an install, including the copy/symlink distinction. */
export function installCopy(install: Install, language: Language): string {
  if (install.status === 'conflict' && install.ignored) {
    return stateCopy('ignored', install.mode, language)
  }
  return stateCopy(install.status, install.mode, language)
}

export function installTone(install: Install): Tone {
  if (install.status === 'conflict' && install.ignored) return 'muted'
  return TONE_BY_STATE[install.status]
}

export function toneForState(state: DisplayState): Tone {
  return TONE_BY_STATE[state]
}

export function worstState(
  states: readonly InstallState[],
): DisplayState {
  let worst: DisplayState = 'none'
  for (const state of states) {
    if (RANK[state] > RANK[worst]) worst = state
  }
  return worst
}

export function isIssueState(state: InstallState): boolean {
  return (
    state === 'pending' ||
    state === 'stale' ||
    state === 'missing' ||
    state === 'conflict' ||
    state === 'error'
  )
}

function isEnabledTarget(targets: readonly Target[], targetId: string): boolean {
  return targets.find((target) => target.id === targetId)?.enabled === true
}

/** Issues that still need a decision, limited to enabled targets. */
export function issueCount(
  installs: readonly Install[],
  targets: readonly Target[],
): number {
  return installs.filter(
    (install) =>
      isEnabledTarget(targets, install.targetId) &&
      !install.ignored &&
      isIssueState(install.status),
  ).length
}

export function skillIssueCount(
  installs: readonly Install[],
  targets: readonly Target[],
): number {
  return issueCount(installs, targets)
}

export function skillWorst(
  installs: readonly Install[],
  targets: readonly Target[],
): DisplayState {
  const states = installs
    .filter((install) => isEnabledTarget(targets, install.targetId))
    .filter((install) => !install.ignored)
    .map((install) => install.status)
  if (states.length === 0) return 'none'
  return worstState(states)
}

export function coverage(
  installs: readonly Install[],
  targets: readonly Target[],
): number {
  return installs.filter((install) =>
    isEnabledTarget(targets, install.targetId),
  ).length
}

export function targetWorst(
  installs: readonly Install[],
  targetId: string,
): DisplayState {
  const states = installs
    .filter((install) => install.targetId === targetId)
    .filter((install) => !install.ignored)
    .map((install) => install.status)
  if (states.length === 0) return 'none'
  return worstState(states)
}

export function targetIssueCount(
  installs: readonly Install[],
  targetId: string,
): number {
  return installs.filter(
    (install) =>
      install.targetId === targetId &&
      !install.ignored &&
      isIssueState(install.status),
  ).length
}

export function targetDistributionCount(
  installs: readonly Install[],
  targetId: string,
): number {
  return installs.filter((install) => install.targetId === targetId).length
}

export function hasCopies(
  installs: readonly Install[],
  targets: readonly Target[],
): boolean {
  return installs.some(
    (install) =>
      install.mode === 'copy' &&
      isEnabledTarget(targets, install.targetId),
  )
}

export function installFor(
  installs: readonly Install[],
  targetId: string,
): Install | undefined {
  return installs.find((install) => install.targetId === targetId)
}
