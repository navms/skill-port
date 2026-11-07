import { describe, expect, it } from 'vitest'
import {
  installWith,
  makeSkill,
  makeTarget,
} from '@/test/fixtures'
import {
  EMPTY_FILTER,
  categoryCounts,
  copiedSkillCount,
  filterSkills,
  totalIssueCount,
  totalLinkCount,
} from './filter'

const claude = makeTarget()
const codex = makeTarget({
  id: 'tgt_codex',
  slug: 'codex',
  name: 'Codex',
  skillsDir: '/tmp/skillport-test/codex/skills',
})

const alpha = makeSkill({
  id: 'skl_alpha',
  slug: 'alpha',
  name: 'Alpha Writer',
  description: 'Writes release notes',
  category: '内容',
  installs: [installWith(claude.id, 'ok')],
})

const beta = makeSkill({
  id: 'skl_beta',
  slug: 'beta',
  name: 'Beta Tool',
  description: 'Builds things',
  category: '工具',
  installs: [
    installWith(codex.id, 'stale', { mode: 'copy' }),
    installWith(claude.id, 'conflict'),
  ],
})

const gamma = makeSkill({
  id: 'skl_gamma',
  slug: 'gamma',
  name: 'Gamma',
  description: 'No installs yet',
  category: '工具',
  installs: [],
})

const skills = [alpha, beta, gamma]
const targets = [claude, codex]

describe('filterSkills', () => {
  it('returns everything with an empty filter', () => {
    expect(filterSkills(skills, targets, EMPTY_FILTER)).toHaveLength(3)
  })

  it('matches query against slug, name and description, case-insensitively', () => {
    expect(
      filterSkills(skills, targets, {
        ...EMPTY_FILTER,
        query: 'ALPHA',
      }).map((skill) => skill.slug),
    ).toEqual(['alpha'])
    expect(
      filterSkills(skills, targets, {
        ...EMPTY_FILTER,
        query: 'release',
      }).map((skill) => skill.slug),
    ).toEqual(['alpha'])
  })

  it('filters by issue state across enabled targets', () => {
    expect(
      filterSkills(skills, targets, {
        ...EMPTY_FILTER,
        chip: 'issues',
      }).map((skill) => skill.slug),
    ).toEqual(['beta'])
  })

  it('filters by copies', () => {
    expect(
      filterSkills(skills, targets, {
        ...EMPTY_FILTER,
        chip: 'copied',
      }).map((skill) => skill.slug),
    ).toEqual(['beta'])
  })

  it('filters by category and target', () => {
    expect(
      filterSkills(skills, targets, {
        ...EMPTY_FILTER,
        category: '工具',
      }).map((skill) => skill.slug),
    ).toEqual(['beta', 'gamma'])

    expect(
      filterSkills(skills, targets, {
        ...EMPTY_FILTER,
        targetId: codex.id,
      }).map((skill) => skill.slug),
    ).toEqual(['beta'])
  })
})

describe('counts', () => {
  it('counts categories by frequency', () => {
    expect(categoryCounts(skills)).toEqual([
      { name: '工具', count: 2 },
      { name: '内容', count: 1 },
    ])
  })

  it('counts issue installs and active links', () => {
    expect(totalIssueCount(skills, targets)).toBe(2)
    expect(totalLinkCount(skills, targets)).toBe(3)
    expect(copiedSkillCount(skills, targets)).toBe(1)
  })
})
