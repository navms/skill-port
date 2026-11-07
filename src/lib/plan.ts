import type { SyncAction, SyncActionKind, SyncPlan } from './types'

export const SYNC_KIND_ORDER: readonly SyncActionKind[] = [
  'create',
  'repair',
  'skip',
  'conflict',
]

export type ActionGroups = Record<SyncActionKind, SyncAction[]>

export interface PlanSummary {
  create: number
  repair: number
  skip: number
  conflict: number
  actionable: number
  total: number
}

export function groupActions(actions: readonly SyncAction[]): ActionGroups {
  const groups: ActionGroups = {
    create: [],
    repair: [],
    skip: [],
    conflict: [],
  }
  for (const action of actions) {
    groups[action.kind].push(action)
  }
  return groups
}

export function summarizePlan(actions: readonly SyncAction[]): PlanSummary {
  const groups = groupActions(actions)
  return {
    create: groups.create.length,
    repair: groups.repair.length,
    skip: groups.skip.length,
    conflict: groups.conflict.length,
    actionable: groups.create.length + groups.repair.length,
    total: actions.length,
  }
}

export function hasActionable(plan: SyncPlan): boolean {
  return plan.actions.some(
    (action) => action.kind === 'create' || action.kind === 'repair',
  )
}

export function planIsExpired(plan: SyncPlan, now = Date.now()): boolean {
  return new Date(plan.expiresAt).getTime() <= now
}
