import {
  coverage,
  hasCopies,
  issueCount,
} from './status'
import type { SkillSummary, Target } from './types'

export type FilterChip = 'all' | 'issues' | 'copied'

export interface SkillFilter {
  query: string
  chip: FilterChip
  category: string | null
  targetId: string | null
}

export const EMPTY_FILTER: SkillFilter = {
  query: '',
  chip: 'all',
  category: null,
  targetId: null,
}

function matchesQuery(skill: SkillSummary, query: string): boolean {
  if (!query) return true
  const haystack =
    `${skill.slug} ${skill.name} ${skill.description} ${skill.category ?? ''}`.toLowerCase()
  return haystack.includes(query)
}

export function filterSkills(
  skills: readonly SkillSummary[],
  targets: readonly Target[],
  filter: SkillFilter,
): SkillSummary[] {
  const query = filter.query.trim().toLowerCase()

  return skills.filter((skill) => {
    if (filter.category && skill.category !== filter.category) return false
    if (
      filter.targetId &&
      !skill.installs.some((install) => install.targetId === filter.targetId)
    ) {
      return false
    }
    if (filter.chip === 'issues' && issueCount(skill.installs, targets) === 0) {
      return false
    }
    if (filter.chip === 'copied' && !hasCopies(skill.installs, targets)) {
      return false
    }
    return matchesQuery(skill, query)
  })
}

export interface CategoryCount {
  name: string
  count: number
}

export function categoryCounts(
  skills: readonly SkillSummary[],
): CategoryCount[] {
  const counts = new Map<string, number>()
  for (const skill of skills) {
    if (!skill.category) continue
    counts.set(skill.category, (counts.get(skill.category) ?? 0) + 1)
  }
  return [...counts.entries()]
    .map(([name, count]) => ({ name, count }))
    .sort((a, b) => b.count - a.count || a.name.localeCompare(b.name))
}

export function totalIssueCount(
  skills: readonly SkillSummary[],
  targets: readonly Target[],
): number {
  return skills.reduce(
    (total, skill) => total + issueCount(skill.installs, targets),
    0,
  )
}

export function totalLinkCount(
  skills: readonly SkillSummary[],
  targets: readonly Target[],
): number {
  return skills.reduce(
    (total, skill) => total + coverage(skill.installs, targets),
    0,
  )
}

export function copiedSkillCount(
  skills: readonly SkillSummary[],
  targets: readonly Target[],
): number {
  return skills.filter((skill) => hasCopies(skill.installs, targets)).length
}
