import { IpcError } from '../errors'
import type {
  Install,
  SyncAction,
  SyncActionKind,
  SyncFilter,
  SyncPlan,
  SyncResult,
  SyncResultItem,
} from '../types'
import {
  db,
  emitLibraryChanged,
  findInstall,
  findSkill,
  findTarget,
  logActivity,
  snapshot,
  wait,
} from './db'

const PLAN_TTL_MS = 5 * 60 * 1000
const STEP_DELAY_MS = 220

function kindFor(status: Install['status'], ignored: boolean): SyncActionKind {
  if (status === 'conflict') return ignored ? 'skip' : 'conflict'
  if (status === 'stale' || status === 'error') return 'repair'
  return 'create'
}

export function buildPlan(filter?: SyncFilter): SyncPlan {
  const actions: SyncAction[] = []

  for (const skill of db.skills) {
    if (filter?.skillId && skill.id !== filter.skillId) continue
    for (const install of skill.installs) {
      if (filter?.targetId && install.targetId !== filter.targetId) continue
      const target = findTarget(install.targetId)
      if (!target?.enabled) continue
      if (install.status === 'ok') continue

      actions.push({
        id: `act_${skill.id}_${target.id}`,
        skillId: skill.id,
        skillSlug: skill.slug,
        skillName: skill.name,
        targetId: target.id,
        targetName: target.name,
        targetPath: install.targetPath,
        mode: install.mode,
        kind: kindFor(install.status, install.ignored),
        state: install.status,
        reason: null,
      })
    }
  }

  const now = Date.now()
  const plan: SyncPlan = {
    id: `plan_${now}_${Math.random().toString(36).slice(2, 8)}`,
    createdAt: new Date(now).toISOString(),
    expiresAt: new Date(now + PLAN_TTL_MS).toISOString(),
    actions,
  }
  db.plans.set(plan.id, plan)
  return snapshot(plan)
}

export function cancelPlan(planId: string): void {
  db.cancelledPlans.add(planId)
}

function markOk(
  skillId: string,
  action: SyncAction,
  contentHash: string,
): void {
  const skill = findSkill(skillId)
  if (!skill) return
  const install = findInstall(skill, action.targetId)
  if (!install) return
  const now = new Date().toISOString()
  install.status = 'ok'
  install.mode = action.mode
  install.actualKind = action.mode
  install.linkedHash = contentHash
  install.targetPath = action.targetPath
  install.lastError = null
  install.lastChecked = now
  install.lastSynced = now
}

export async function applyPlan(planId: string): Promise<SyncResult> {
  const plan = db.plans.get(planId)
  if (!plan) {
    throw new IpcError({
      code: 'PlanNotFound',
      message: 'The sync plan no longer exists. Generate a new plan.',
    })
  }
  if (new Date(plan.expiresAt).getTime() <= Date.now()) {
    throw new IpcError({
      code: 'PlanExpired',
      message: 'The sync plan expired. Generate a new plan.',
    })
  }

  const actionable = plan.actions.filter(
    (action) => action.kind === 'create' || action.kind === 'repair',
  )
  const previous: {
    skillId: string
    targetId: string
    previous: Install | null
  }[] = []
  for (const action of actionable) {
    const skill = findSkill(action.skillId)
    if (!skill) continue
    const install = findInstall(skill, action.targetId)
    previous.push({
      skillId: action.skillId,
      targetId: action.targetId,
      previous: install ? snapshot(install) : null,
    })
  }

  const items: SyncResultItem[] = []
  let applied = 0
  let cancelled = false

  for (let index = 0; index < actionable.length; index += 1) {
    if (db.cancelledPlans.has(planId)) {
      cancelled = true
      break
    }
    const action = actionable[index]
    if (!action) continue
    await wait(STEP_DELAY_MS)
    if (db.cancelledPlans.has(planId)) {
      cancelled = true
      break
    }

    const skill = findSkill(action.skillId)
    if (!skill) {
      items.push({
        actionId: action.id,
        ok: false,
        errorCode: 'SkillNotFound',
        message: 'Skill no longer exists.',
      })
      db.channels.syncProgress.emit({
        planId,
        completed: index + 1,
        total: actionable.length,
        actionId: action.id,
        status: 'failed',
      })
      continue
    }

    markOk(skill.id, action, skill.contentHash)
    applied += 1
    items.push({
      actionId: action.id,
      ok: true,
      errorCode: null,
      message: null,
    })
    db.channels.syncProgress.emit({
      planId,
      completed: index + 1,
      total: actionable.length,
      actionId: action.id,
      status: 'ok',
    })
  }

  db.cancelledPlans.delete(planId)
  const failed = items.filter((item) => !item.ok).length

  let undoId: string | null = null
  if (applied > 0) {
    const activityId = logActivity({
      action: 'sync',
      skillId:
        actionable.length === 1 && actionable[0] ? actionable[0].skillId : null,
      skillName:
        actionable.length === 1 && actionable[0]
          ? actionable[0].skillSlug
          : null,
      targetId:
        actionable.length === 1 && actionable[0]
          ? actionable[0].targetId
          : null,
      targetName:
        actionable.length === 1 && actionable[0]
          ? actionable[0].targetName
          : null,
      result: failed > 0 ? 'failed' : 'ok',
      detail: cancelled
        ? `同步已取消，完成 ${applied} 项`
        : `同步完成 ${applied} 项`,
    })
    undoId = String(activityId)
    db.undo.set(undoId, {
      activityId: undoId,
      kind: 'sync',
      installs: previous,
    })
  }

  emitLibraryChanged()

  return {
    planId,
    applied,
    failed,
    cancelled,
    undoId,
    items,
  }
}
