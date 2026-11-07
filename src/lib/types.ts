/**
 * IPC contract shared with `src-tauri/src/core/model.rs`.
 * Names are camelCase over IPC and must stay in sync with the Rust model.
 */

export type ThemeMode = 'system' | 'light' | 'dark'
export type Language = 'zh-CN' | 'en'

export type LinkMode = 'symlink' | 'copy'
export type TargetKind = 'skills_dir'
export type SourceKind = 'local' | 'import' | 'git'
export type ActualKind = 'symlink' | 'junction' | 'copy'

export type InstallState =
  | 'pending'
  | 'ok'
  | 'stale'
  | 'missing'
  | 'conflict'
  | 'error'

export type Tone = 'ok' | 'warn' | 'danger' | 'muted'

export interface Target {
  id: string
  slug: string
  name: string
  kind: TargetKind
  skillsDir: string
  linkMode: LinkMode
  enabled: boolean
  /** True when skillsDir came from detection rather than manual entry. */
  detected: boolean
  createdAt: string
  updatedAt: string
}

export interface Install {
  id: number
  skillId: string
  targetId: string
  mode: LinkMode
  actualKind: ActualKind | null
  status: InstallState
  targetPath: string
  linkedHash: string | null
  /** User chose "keep and ignore" for an external conflict. */
  ignored: boolean
  lastError: string | null
  lastChecked: string | null
  lastSynced: string | null
}

/**
 * `sizeBytes` is derived from the library tree when scanning; `category` is a
 * SkillPort-owned column. Imports always leave it empty; it is only changed
 * from the edit dialog and is never written back to SKILL.md.
 */
export interface SkillSummary {
  id: string
  slug: string
  name: string
  description: string
  category: string | null
  sourceKind: SourceKind
  sourceRef: string | null
  contentHash: string
  sizeBytes: number
  createdAt: string
  updatedAt: string
  installs: Install[]
}

export interface SkillDetail extends SkillSummary {
  path: string
  skillMd: string
}

export interface SkillPatch {
  skillMd?: string
  name?: string
  description?: string
  /** SkillPort-owned grouping stored in the DB. Empty string clears it. */
  category?: string
}

export interface DeleteSkillOptions {
  /** Remove skill-owned links/copies from target directories first. */
  deleteTargets: boolean
}

export interface DeleteSkillResult {
  /** Activity id that can be passed to `undoActivity`, when a backup exists. */
  undoId: string | null
  removedInstalls: number
}

export type ImportErrorCode =
  | 'MissingSkillFile'
  | 'InvalidFrontmatter'
  | 'MissingName'
  | 'MissingDescription'
  | 'InvalidSlug'
  | 'DuplicateSlug'
  | 'PathNotFound'
  | 'IoError'

export interface ImportResultItem {
  path: string
  slug: string | null
  ok: boolean
  errorCode: ImportErrorCode | null
  detail: string | null
}

export interface ImportReport {
  items: ImportResultItem[]
  imported: number
  skipped: number
}

export interface TargetInput {
  slug: string
  name: string
  skillsDir: string
  linkMode: LinkMode
  enabled: boolean
}

export type TargetPatch = Partial<TargetInput>

export interface SuggestedTarget {
  slug: string
  name: string
  skillsDir: string
  /** Whether the directory currently exists on disk. */
  exists: boolean
  alreadyAdded: boolean
}

export interface InstallInput {
  enabled: boolean
  mode?: LinkMode
}

export type ConflictResolution = 'adopt' | 'ignore' | 'unignore' | 'cancel'

export type SyncActionKind = 'create' | 'repair' | 'skip' | 'conflict'

export interface SyncAction {
  id: string
  skillId: string
  skillSlug: string
  skillName: string
  targetId: string
  targetName: string
  targetPath: string
  mode: LinkMode
  kind: SyncActionKind
  state: InstallState
  reason: string | null
}

export interface SyncPlan {
  id: string
  createdAt: string
  /** Plans expire after five minutes and must be re-planned. */
  expiresAt: string
  actions: SyncAction[]
}

export interface SyncFilter {
  skillId?: string
  targetId?: string
}

export interface SyncResultItem {
  actionId: string
  ok: boolean
  errorCode: string | null
  message: string | null
}

export interface SyncResult {
  planId: string
  applied: number
  failed: number
  cancelled: boolean
  undoId: string | null
  items: SyncResultItem[]
}

export interface SyncProgress {
  planId: string
  completed: number
  total: number
  actionId: string | null
  status: 'running' | 'ok' | 'failed' | 'conflict'
}

export interface ReconcileReport {
  checked: number
  changed: number
  issues: number
  checkedAt: string
}

export type ActivityAction =
  | 'import'
  | 'link'
  | 'copy'
  | 'unlink'
  | 'sync'
  | 'reconcile'
  | 'conflict'
  | 'delete'
  | 'update'
  | 'target'
  | 'restore'

export interface ActivityEntry {
  id: number
  ts: string
  action: ActivityAction
  skillId: string | null
  skillName: string | null
  targetId: string | null
  targetName: string | null
  result: 'ok' | 'failed'
  detail: string | null
}

export interface ActivityQuery {
  limit?: number
  cursor?: string | null
}

export interface ActivityPage {
  items: ActivityEntry[]
  nextCursor: string | null
}

export interface UndoResult {
  ok: boolean
  message: string | null
}

export interface AppSettings {
  defaultLinkMode: LinkMode
  libraryPath: string
  dataDir: string
  backupRetention: number
}

export interface DeleteTargetOptions {
  unlink: boolean
}

/** A harness icon file from the data directory, keyed by target slug. */
export interface TargetIconAsset {
  slug: string
  svg: string
}

export type Unsubscribe = () => void

export interface Api {
  listSkills(): Promise<SkillSummary[]>
  getSkill(id: string): Promise<SkillDetail>
  importSkills(paths: string[]): Promise<ImportReport>
  updateSkill(id: string, patch: SkillPatch): Promise<SkillDetail>
  deleteSkill(id: string, options: DeleteSkillOptions): Promise<DeleteSkillResult>
  revealSkill(id: string): Promise<void>
  pickImportPaths(): Promise<string[]>

  listTargets(): Promise<Target[]>
  detectTargets(): Promise<SuggestedTarget[]>
  createTarget(input: TargetInput): Promise<Target>
  updateTarget(id: string, patch: TargetPatch): Promise<Target>
  deleteTarget(id: string, options: DeleteTargetOptions): Promise<void>
  pickDirectory(initialDir?: string): Promise<string | null>
  listTargetIcons(): Promise<TargetIconAsset[]>
  revealIconsDir(): Promise<void>

  setInstall(
    skillId: string,
    targetId: string,
    input: InstallInput,
  ): Promise<Install | null>
  resolveInstall(
    skillId: string,
    targetId: string,
    resolution: ConflictResolution,
  ): Promise<Install | null>

  reconcile(): Promise<ReconcileReport>
  planSync(filter?: SyncFilter): Promise<SyncPlan>
  applySync(planId: string): Promise<SyncResult>
  cancelSync(planId: string): Promise<void>

  listActivity(query?: ActivityQuery): Promise<ActivityPage>
  undoActivity(activityId: string): Promise<UndoResult>

  getSettings(): Promise<AppSettings>
  updateSettings(patch: Partial<AppSettings>): Promise<AppSettings>

  subscribeSyncProgress(listener: (progress: SyncProgress) => void): Unsubscribe
  subscribeLibraryChanged(listener: () => void): Unsubscribe
  subscribeReconcileFinished(
    listener: (report: ReconcileReport) => void,
  ): Unsubscribe
}
