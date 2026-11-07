import { describe, expect, it } from 'vitest'
import {
  groupActions,
  hasActionable,
  planIsExpired,
  summarizePlan,
} from './plan'
import type { SyncAction, SyncActionKind, SyncPlan } from './types'

function action(
  id: string,
  kind: SyncActionKind,
): SyncAction {
  return {
    id,
    skillId: 'skl_demo',
    skillSlug: 'demo',
    skillName: 'Demo',
    targetId: 'tgt_claude',
    targetName: 'Claude Code',
    targetPath: '/tmp/skillport-test/claude/skills/demo',
    mode: 'symlink',
    kind,
    state: 'ok',
    reason: null,
  }
}

const actions = [
  action('a', 'create'),
  action('b', 'create'),
  action('c', 'repair'),
  action('d', 'skip'),
  action('e', 'conflict'),
]

describe('groupActions', () => {
  it('groups by kind while preserving order', () => {
    const groups = groupActions(actions)
    expect(groups.create.map((item) => item.id)).toEqual(['a', 'b'])
    expect(groups.repair.map((item) => item.id)).toEqual(['c'])
    expect(groups.skip.map((item) => item.id)).toEqual(['d'])
    expect(groups.conflict.map((item) => item.id)).toEqual(['e'])
  })
})

describe('summarizePlan', () => {
  it('counts actionable steps as create plus repair', () => {
    expect(summarizePlan(actions)).toEqual({
      create: 2,
      repair: 1,
      skip: 1,
      conflict: 1,
      actionable: 3,
      total: 5,
    })
  })

  it('reports nothing actionable when only skips and conflicts remain', () => {
    expect(
      summarizePlan([action('d', 'skip'), action('e', 'conflict')]),
    ).toMatchObject({ actionable: 0, total: 2 })
  })
})

describe('plan checks', () => {
  it('detects whether a plan has work to execute', () => {
    expect(
      hasActionable({
        id: 'plan',
        createdAt: '2026-01-01T00:00:00.000Z',
        expiresAt: '2026-01-01T00:05:00.000Z',
        actions,
      }),
    ).toBe(true)
    expect(
      hasActionable({
        id: 'plan',
        createdAt: '2026-01-01T00:00:00.000Z',
        expiresAt: '2026-01-01T00:05:00.000Z',
        actions: [action('d', 'skip')],
      }),
    ).toBe(false)
  })

  it('treats plans as expired at or after expiresAt', () => {
    const plan: SyncPlan = {
      id: 'plan',
      createdAt: '2026-01-01T00:00:00.000Z',
      expiresAt: '2026-01-01T00:05:00.000Z',
      actions,
    }
    expect(
      planIsExpired(plan, new Date('2026-01-01T00:04:59.000Z').getTime()),
    ).toBe(false)
    expect(
      planIsExpired(plan, new Date('2026-01-01T00:05:00.000Z').getTime()),
    ).toBe(true)
  })
})
