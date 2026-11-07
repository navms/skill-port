import { describe, expect, it } from 'vitest'
import { installWith, makeInstall, makeTarget } from '@/test/fixtures'
import {
  coverage,
  hasCopies,
  installCopy,
  installTone,
  issueCount,
  skillWorst,
  stateCopy,
  targetIssueCount,
  targetWorst,
  worstState,
} from './status'

const claude = makeTarget()
const codex = makeTarget({
  id: 'tgt_codex',
  slug: 'codex',
  name: 'Codex',
  skillsDir: '/tmp/skillport-test/codex/skills',
})

describe('stateCopy', () => {
  it('distinguishes linked and copied success states', () => {
    expect(stateCopy('ok', 'symlink', 'zh-CN')).toBe('已链接')
    expect(stateCopy('ok', 'copy', 'zh-CN')).toBe('副本')
    expect(stateCopy('ok', 'symlink', 'en')).toBe('Linked')
    expect(stateCopy('ok', 'copy', 'en')).toBe('Copy')
  })

  it('maps every issue state in both languages', () => {
    expect(stateCopy('stale', 'copy', 'zh-CN')).toBe('副本过期')
    expect(stateCopy('missing', 'symlink', 'zh-CN')).toBe('缺失')
    expect(stateCopy('conflict', 'symlink', 'zh-CN')).toBe('冲突')
    expect(stateCopy('error', 'symlink', 'zh-CN')).toBe('异常')
    expect(stateCopy('pending', 'symlink', 'zh-CN')).toBe('待同步')
    expect(stateCopy('none', 'symlink', 'zh-CN')).toBe('未链接')
    expect(stateCopy('ignored', 'symlink', 'en')).toBe('Ignored')
  })
})

describe('worstState', () => {
  it('ranks conflict above error above missing above stale above pending', () => {
    expect(worstState(['ok', 'stale'])).toBe('stale')
    expect(worstState(['stale', 'missing'])).toBe('missing')
    expect(worstState(['missing', 'error'])).toBe('error')
    expect(worstState(['error', 'conflict'])).toBe('conflict')
    expect(worstState(['ok', 'pending'])).toBe('pending')
    expect(worstState([])).toBe('none')
    expect(worstState(['ok', 'ok'])).toBe('ok')
  })
})

describe('install presentation', () => {
  it('marks ignored conflicts as muted and labelled ignored', () => {
    const install = installWith('tgt_claude', 'conflict', { ignored: true })
    expect(installTone(install)).toBe('muted')
    expect(installCopy(install, 'zh-CN')).toBe('已忽略')
  })

  it('marks unresolved conflicts as danger', () => {
    const install = installWith('tgt_claude', 'conflict')
    expect(installTone(install)).toBe('danger')
    expect(installCopy(install, 'en')).toBe('Conflict')
  })
})

describe('skill and target aggregates', () => {
  const installs = [
    installWith(claude.id, 'ok'),
    installWith(codex.id, 'stale'),
  ]

  it('limits coverage and issues to enabled targets', () => {
    expect(coverage(installs, [claude, codex])).toBe(2)
    expect(issueCount(installs, [claude, codex])).toBe(1)

    const withDisabled = [
      ...installs,
      installWith(codex.id, 'conflict'),
    ]
    expect(
      coverage(withDisabled, [claude, { ...codex, enabled: false }]),
    ).toBe(1)
    expect(skillWorst(withDisabled, [claude, { ...codex, enabled: false }])).toBe(
      'ok',
    )
  })

  it('ignores ignored conflicts in aggregates', () => {
    const ignored = installWith(claude.id, 'conflict', { ignored: true })
    expect(skillWorst([ignored], [claude])).toBe('none')
    expect(issueCount([ignored], [claude])).toBe(0)
    expect(targetWorst([ignored], claude.id)).toBe('none')
    expect(targetIssueCount([ignored], claude.id)).toBe(0)
  })

  it('returns none when a skill has no installs on enabled targets', () => {
    const install = makeInstall({ targetId: codex.id, status: 'missing' })
    expect(skillWorst([install], [claude])).toBe('none')
  })

  it('detects copies only on enabled targets', () => {
    const copied = installWith(claude.id, 'ok', { mode: 'copy' })
    expect(hasCopies([copied], [claude])).toBe(true)
    expect(hasCopies([copied], [{ ...claude, enabled: false }])).toBe(false)
  })
})
