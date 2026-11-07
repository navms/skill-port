import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { toIpcError } from './errors'
import type {
  ActivityPage,
  ActivityQuery,
  Api,
  AppSettings,
  ConflictResolution,
  DeleteSkillOptions,
  DeleteSkillResult,
  DeleteTargetOptions,
  ImportReport,
  Install,
  InstallInput,
  ReconcileReport,
  SkillDetail,
  SkillPatch,
  SkillSummary,
  SuggestedTarget,
  SyncFilter,
  SyncPlan,
  SyncProgress,
  SyncResult,
  Target,
  TargetIconAsset,
  TargetInput,
  TargetPatch,
  Unsubscribe,
  UndoResult,
} from './types'

async function call<T>(
  command: string,
  args?: Record<string, unknown>,
): Promise<T> {
  try {
    return await invoke<T>(command, args)
  } catch (error) {
    throw toIpcError(error)
  }
}

function subscribe<T>(
  event: string,
  listener: (payload: T) => void,
): Unsubscribe {
  let unlisten: (() => void) | null = null
  let disposed = false
  listen<T>(event, (payload) => {
    listener(payload.payload)
  })
    .then((stop) => {
      if (disposed) stop()
      else unlisten = stop
    })
    .catch((error: unknown) => {
      console.error(`Failed to subscribe to ${event}`, error)
    })
  return () => {
    disposed = true
    unlisten?.()
    unlisten = null
  }
}

export function createTauriApi(): Api {
  return {
    listSkills: () => call<SkillSummary[]>('library_list'),
    getSkill: (id) => call<SkillDetail>('library_get', { id }),
    importSkills: (paths) =>
      call<ImportReport>('library_import', { paths }),
    updateSkill: (id, patch: SkillPatch) =>
      call<SkillDetail>('library_update', { id, patch }),
    deleteSkill: (id, options: DeleteSkillOptions) =>
      call<DeleteSkillResult>('library_delete', { id, options }),
    revealSkill: (id) => call<void>('library_reveal', { id }),

    async pickImportPaths(): Promise<string[]> {
      const { open } = await import('@tauri-apps/plugin-dialog')
      const selected = await open({
        directory: true,
        multiple: true,
        title: 'Import skill folders',
      })
      if (!selected) return []
      return Array.isArray(selected) ? selected : [selected]
    },

    listTargets: () => call<Target[]>('targets_list'),
    detectTargets: () =>
      call<SuggestedTarget[]>('target_detect_suggestions'),
    createTarget: (input: TargetInput) =>
      call<Target>('target_create', { input }),
    updateTarget: (id, patch: TargetPatch) =>
      call<Target>('target_update', { id, patch }),
    deleteTarget: (id, options: DeleteTargetOptions) =>
      call<void>('target_delete', { id, options }),
    revealTarget: (id) => call<void>('target_reveal', { id }),
    pickDirectory: (initialDir) =>
      call<string | null>('target_pick_dir', { initialDir }),
    listTargetIcons: () => call<TargetIconAsset[]>('target_icon_list'),
    revealIconsDir: () => call<void>('target_icon_reveal'),

    setInstall: (skillId, targetId, input: InstallInput) =>
      call<Install | null>('installs_set', { skillId, targetId, input }),
    resolveInstall: (
      skillId,
      targetId,
      resolution: ConflictResolution,
    ) =>
      call<Install | null>('installs_resolve', {
        skillId,
        targetId,
        resolution,
      }),

    reconcile: () => call<ReconcileReport>('reconcile'),
    planSync: (filter?: SyncFilter) =>
      call<SyncPlan>('sync_plan', { filter }),
    applySync: (planId) => call<SyncResult>('sync_apply', { planId }),
    cancelSync: (planId) => call<void>('sync_cancel', { planId }),

    listActivity: (query?: ActivityQuery) =>
      call<ActivityPage>('activity_list', { query }),
    undoActivity: (activityId) =>
      call<UndoResult>('activity_undo', { activityId }),

    getSettings: () => call<AppSettings>('settings_get'),
    updateSettings: (patch: Partial<AppSettings>) =>
      call<AppSettings>('settings_set', { patch }),

    subscribeSyncProgress: (listener) =>
      subscribe<SyncProgress>('sync://progress', listener),
    subscribeLibraryChanged: (listener) =>
      subscribe<void>('library://changed', () => listener()),
    subscribeReconcileFinished: (listener) =>
      subscribe<ReconcileReport>('reconcile://finished', listener),
  }
}
