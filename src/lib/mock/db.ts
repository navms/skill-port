import type {
  ActivityAction,
  ActivityEntry,
  Install,
  ReconcileReport,
  SkillDetail,
  SyncPlan,
  SyncProgress,
} from '../types'
import { issueCount } from '../status'
import { buildSeed } from './seed'

export interface UndoRecord {
  activityId: string
  kind: 'skill-delete' | 'sync'
  skill?: SkillDetail
  installs?: {
    skillId: string
    targetId: string
    previous: Install | null
  }[]
}

type Listener<T> = (payload: T) => void

function createChannel<T>() {
  const listeners = new Set<Listener<T>>()
  return {
    subscribe(listener: Listener<T>): () => void {
      listeners.add(listener)
      return () => {
        listeners.delete(listener)
      }
    },
    emit(payload: T): void {
      for (const listener of listeners) listener(payload)
    },
  }
}

const seed = buildSeed()
const maxInstallId = seed.skills
  .flatMap((skill) => skill.installs)
  .reduce((max, install) => Math.max(max, install.id), 0)

export const db = {
  targets: seed.targets,
  skills: seed.skills,
  activity: seed.activity,
  settings: seed.settings,
  nextInstallId: maxInstallId + 1,
  nextActivityId: seed.nextActivityId,
  nextSkillSequence: 1,
  plans: new Map<string, SyncPlan>(),
  cancelledPlans: new Set<string>(),
  undo: new Map<string, UndoRecord>(),
  channels: {
    library: createChannel<void>(),
    reconcile: createChannel<ReconcileReport>(),
    syncProgress: createChannel<SyncProgress>(),
  },
}

export function snapshot<T>(value: T): T {
  return structuredClone(value)
}

export function wait(ms: number): Promise<void> {
  return new Promise((resolve) => {
    setTimeout(resolve, ms)
  })
}

export function findSkill(idOrSlug: string): SkillDetail | undefined {
  return db.skills.find(
    (skill) => skill.id === idOrSlug || skill.slug === idOrSlug,
  )
}

export function findTarget(idOrSlug: string) {
  return db.targets.find(
    (target) => target.id === idOrSlug || target.slug === idOrSlug,
  )
}

export function findInstall(
  skill: SkillDetail,
  targetId: string,
): Install | undefined {
  return skill.installs.find((install) => install.targetId === targetId)
}

export function replaceInstall(
  skillId: string,
  targetId: string,
  install: Install | null,
): void {
  const skill = db.skills.find((candidate) => candidate.id === skillId)
  if (!skill) return
  skill.installs = skill.installs.filter(
    (candidate) => candidate.targetId !== targetId,
  )
  if (install) skill.installs.push(install)
}

export function logActivity(
  entry: Omit<ActivityEntry, 'id' | 'ts'> & { ts?: string },
): number {
  const id = db.nextActivityId
  db.nextActivityId += 1
  db.activity.unshift({
    ...entry,
    id,
    ts: entry.ts ?? new Date().toISOString(),
  })
  return id
}

export function countIssues(): number {
  return db.skills.reduce(
    (total, skill) => total + issueCount(skill.installs, db.targets),
    0,
  )
}

export function emitLibraryChanged(): void {
  db.channels.library.emit()
}

const ACTION_LABELS: Record<ActivityAction, string> = {
  import: '导入',
  link: '链接',
  copy: '拷贝',
  unlink: '取消分发',
  sync: '同步',
  reconcile: '核对',
  conflict: '冲突',
  delete: '删除',
  update: '更新',
  target: '目标',
  restore: '撤销',
}

export function activityLabel(action: ActivityAction): string {
  return ACTION_LABELS[action]
}
