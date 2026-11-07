import { describe, expect, it } from 'vitest'
import { makeTarget } from '@/test/fixtures'
import {
  foldDir,
  isDuplicateDir,
  isInsideLibrary,
  normalizeDir,
} from './targetPathRules'

describe('normalizeDir', () => {
  it('trims and strips trailing separators', () => {
    expect(normalizeDir('  /Users/me/.claude/skills/  ')).toBe(
      '/Users/me/.claude/skills',
    )
    expect(normalizeDir('C:\\Users\\me\\skills\\')).toBe(
      'C:\\Users\\me\\skills',
    )
  })
})

describe('foldDir', () => {
  it('folds separators and case for comparison', () => {
    expect(foldDir('C:\\Users\\Me\\Skills')).toBe('c:/users/me/skills')
  })
})

describe('isInsideLibrary', () => {
  it('matches the library itself and its descendants', () => {
    expect(isInsideLibrary('/data/library', '/data/library')).toBe(true)
    expect(isInsideLibrary('/data/library/x', '/data/library')).toBe(true)
    expect(isInsideLibrary('/data/library-2', '/data/library')).toBe(false)
  })

  it('is false when either side is empty', () => {
    expect(isInsideLibrary('', '/data/library')).toBe(false)
    expect(isInsideLibrary('/data/library', '')).toBe(false)
  })
})

describe('isDuplicateDir', () => {
  const targets = [
    makeTarget({ id: 'a', skillsDir: '/Users/me/.claude/skills' }),
    makeTarget({ id: 'b', skillsDir: '/Users/me/.codex/skills' }),
  ]

  it('detects another target using the same directory', () => {
    expect(isDuplicateDir('/Users/me/.codex/skills/', targets)).toBe(true)
    expect(isDuplicateDir('C:\\Users\\Me\\.codex\\skills', targets)).toBe(
      false,
    )
  })

  it('ignores the target being edited', () => {
    expect(
      isDuplicateDir('/Users/me/.codex/skills', targets, 'b'),
    ).toBe(false)
  })
})
