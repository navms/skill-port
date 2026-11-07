import { IpcError } from './errors'
import type {
  ActivityPage,
  ActivityQuery,
  Api,
  AppSettings,
  ConflictResolution,
  DeleteSkillResult,
  DeleteSkillOptions,
  DeleteTargetOptions,
  ImportReport,
  ImportResultItem,
  Install,
  InstallInput,
  ReconcileReport,
  SkillDetail,
  SkillPatch,
  SkillSummary,
  SuggestedTarget,
  Target,
  TargetIconAsset,
  TargetInput,
  TargetPatch,
} from './types'
import {
  countIssues,
  db,
  emitLibraryChanged,
  findInstall,
  findSkill,
  findTarget,
  logActivity,
  replaceInstall,
  snapshot,
  wait,
} from './mock/db'
import claudeSvg from '@/assets/targets/claude-code.svg?raw'
import codexSvg from '@/assets/targets/codex.svg?raw'
import cursorSvg from '@/assets/targets/cursor.svg?raw'
import deepseekSvg from '@/assets/targets/deepseek.svg?raw'
import geminiSvg from '@/assets/targets/gemini-cli.svg?raw'
import opencodeSvg from '@/assets/targets/opencode.svg?raw'
import qoderSvg from '@/assets/targets/qoder.svg?raw'
import workbuddySvg from '@/assets/targets/workbuddy.svg?raw'
import { parseSkillMd, slugify } from './frontmatter'
import { buildSkillMd, fakeHash } from './mock/seed'
import { basenameOf, titleFromSlug } from './mock/skillFile'
import { applyPlan, buildPlan, cancelPlan } from './mock/sync'

function toSummary(skill: SkillDetail): SkillSummary {
  return snapshot({
    id: skill.id,
    slug: skill.slug,
    name: skill.name,
    description: skill.description,
    category: skill.category,
    sourceKind: skill.sourceKind,
    sourceRef: skill.sourceRef,
    contentHash: skill.contentHash,
    sizeBytes: skill.sizeBytes,
    createdAt: skill.createdAt,
    updatedAt: skill.updatedAt,
    installs: skill.installs,
  })
}

function requireSkill(idOrSlug: string): SkillDetail {
  const skill = findSkill(idOrSlug)
  if (!skill) {
    throw new IpcError({ code: 'SkillNotFound', message: 'Skill not found.' })
  }
  return skill
}

function reject(
  path: string,
  code: ImportResultItem['errorCode'],
  detail: string,
  slug: string | null = null,
): ImportResultItem {
  return { path, slug, ok: false, errorCode: code, detail }
}

function importMarkdown(path: string): ImportResultItem {
  const base = basenameOf(path)
  if (!base) {
    return reject(path, 'PathNotFound', 'The folder path is empty.')
  }

  if (base.includes('missing-skill-md')) {
    return reject(path, 'MissingSkillFile', 'SKILL.md was not found.')
  }

  if (base.includes('no-frontmatter')) {
    return reject(
      path,
      'InvalidFrontmatter',
      'The folder has no YAML frontmatter.',
    )
  }

  const slug = slugify(base)
  if (!slug) {
    return reject(path, 'InvalidSlug', `Cannot derive a slug from "${base}".`)
  }
  if (db.skills.some((skill) => skill.slug === slug)) {
    return reject(path, 'DuplicateSlug', `A skill with slug "${slug}" exists.`, slug)
  }

  const md = base.includes('broken')
    ? '---\nname: Broken Skill\ncategory: 工具\n---\n\n# Broken Skill\n'
    : buildSkillMd({
        name: titleFromSlug(slug),
        description: `Imported from ${path}`,
        body: `Imported skill from ${path}.`,
      })

  const parsed = parseSkillMd(md)
  if (!parsed.ok) {
    return reject(path, parsed.code, parsed.detail, slug)
  }

  const now = new Date().toISOString()
  const skill: SkillDetail = {
    id: `skl_${slug}`,
    slug,
    name: parsed.meta.name,
    description: parsed.meta.description,
    // 分类是 SkillPort 自己的口径，导入时不从 frontmatter 取名。
    category: null,
    sourceKind: 'import',
    sourceRef: path,
    contentHash: fakeHash(md),
    sizeBytes: md.length * 2,
    createdAt: now,
    updatedAt: now,
    installs: [],
    path: `${db.settings.libraryPath}/${slug}`,
    skillMd: md,
  }
  db.skills.push(skill)

  return {
    path,
    slug,
    ok: true,
    errorCode: null,
    detail: null,
  }
}

/** The browser mock has no icons directory; mirror the bundled defaults. */
const MOCK_ICONS: TargetIconAsset[] = [
  { slug: 'claude-code', svg: claudeSvg },
  { slug: 'codex', svg: codexSvg },
  { slug: 'cursor', svg: cursorSvg },
  { slug: 'deepseek', svg: deepseekSvg },
  { slug: 'gemini-cli', svg: geminiSvg },
  { slug: 'opencode', svg: opencodeSvg },
  { slug: 'qoder', svg: qoderSvg },
  { slug: 'workbuddy', svg: workbuddySvg },
]

export const mockApi: Api = {
  async listSkills(): Promise<SkillSummary[]> {
    await wait(80)
    return db.skills.map(toSummary)
  },

  async getSkill(id: string): Promise<SkillDetail> {
    await wait(60)
    return snapshot(requireSkill(id))
  },

  async importSkills(paths: string[]): Promise<ImportReport> {
    await wait(220)
    const items: ImportResultItem[] = []
    for (const raw of paths) {
      const path = raw.trim()
      if (!path) continue
      items.push(importMarkdown(path))
    }

    const imported = items.filter((item) => item.ok).length
    const skipped = items.length - imported
    if (imported > 0) {
      logActivity({
        action: 'import',
        skillId: null,
        skillName: null,
        targetId: null,
        targetName: null,
        result: 'ok',
        detail: `导入 ${imported} 个技能，${skipped} 个被拒绝`,
      })
      emitLibraryChanged()
    }
    return { items, imported, skipped }
  },

  async updateSkill(id: string, patch: SkillPatch): Promise<SkillDetail> {
    await wait(120)
    const skill = requireSkill(id)
    const editsFile =
      patch.skillMd !== undefined ||
      patch.name !== undefined ||
      patch.description !== undefined
    if (!editsFile && patch.category === undefined) return snapshot(skill)

    const nextMd = patch.skillMd ?? skill.skillMd
    const parsed = parseSkillMd(nextMd)
    if (editsFile && !parsed.ok) {
      throw new IpcError({ code: parsed.code, message: parsed.detail })
    }

    const now = new Date().toISOString()
    if (editsFile && parsed.ok) {
      skill.skillMd = nextMd
      skill.name = patch.name ?? parsed.meta.name
      skill.description = patch.description ?? parsed.meta.description
      skill.contentHash = fakeHash(nextMd)
      for (const install of skill.installs) {
        install.lastChecked = now
        if (install.mode === 'copy' && install.status === 'ok') {
          install.status = 'stale'
        }
      }
    }
    // 分类是 SkillPort 自己的字段，只改内存/DB，绝不写进 SKILL.md。
    if (patch.category !== undefined) {
      skill.category = patch.category.trim() || null
    }
    skill.updatedAt = now
    logActivity({
      action: 'update',
      skillId: skill.id,
      skillName: skill.slug,
      targetId: null,
      targetName: null,
      result: 'ok',
      detail: editsFile
        ? `${skill.slug} 的 SKILL.md 已更新`
        : `${skill.slug} 的分类已更新`,
    })
    emitLibraryChanged()
    return snapshot(skill)
  },

  async deleteSkill(
    id: string,
    options: DeleteSkillOptions,
  ): Promise<DeleteSkillResult> {
    await wait(160)
    const skill = requireSkill(id)
    const removedInstalls = skill.installs.length
    const backup = snapshot(skill)
    db.skills = db.skills.filter((candidate) => candidate.id !== skill.id)

    const activityId = logActivity({
      action: 'delete',
      skillId: skill.id,
      skillName: skill.slug,
      targetId: null,
      targetName: null,
      result: 'ok',
      detail: options.deleteTargets
        ? `已删除 ${skill.slug}，并清理 ${removedInstalls} 条链接`
        : `已删除 ${skill.slug}，目标目录中的链接保留为未托管`,
    })
    const undoId = String(activityId)
    db.undo.set(undoId, {
      activityId: undoId,
      kind: 'skill-delete',
      skill: backup,
    })
    emitLibraryChanged()
    return { undoId, removedInstalls }
  },

  async revealSkill(id: string): Promise<void> {
    requireSkill(id)
    await wait(40)
  },

  async pickImportPaths(): Promise<string[]> {
    return []
  },

  async listTargets(): Promise<Target[]> {
    await wait(60)
    return snapshot(db.targets)
  },

  async detectTargets(): Promise<SuggestedTarget[]> {
    await wait(90)
    return []
  },

  async createTarget(input: TargetInput): Promise<Target> {
    await wait(140)
    const slug = slugify(input.slug)
    if (!slug) {
      throw new IpcError({ code: 'InvalidSlug', message: 'Invalid target slug.' })
    }
    if (db.targets.some((target) => target.slug === slug)) {
      throw new IpcError({
        code: 'DuplicateSlug',
        message: 'A target with this slug already exists.',
      })
    }
    if (db.targets.some((target) => target.skillsDir === input.skillsDir)) {
      throw new IpcError({
        code: 'DuplicatePath',
        message: 'Another target already uses this skills folder.',
      })
    }
    const now = new Date().toISOString()
    const target: Target = {
      id: `tgt_${slug}`,
      slug,
      name: input.name.trim(),
      kind: 'skills_dir',
      skillsDir: input.skillsDir.trim(),
      linkMode: input.linkMode,
      enabled: input.enabled,
      detected: false,
      createdAt: now,
      updatedAt: now,
    }
    db.targets.push(target)
    logActivity({
      action: 'target',
      skillId: null,
      skillName: null,
      targetId: target.id,
      targetName: target.name,
      result: 'ok',
      detail: `添加目标 ${target.name}`,
    })
    return snapshot(target)
  },

  async updateTarget(id: string, patch: TargetPatch): Promise<Target> {
    await wait(140)
    const target = db.targets.find((candidate) => candidate.id === id)
    if (!target) {
      throw new IpcError({ code: 'TargetNotFound', message: 'Target not found.' })
    }
    const nextDir = patch.skillsDir?.trim() ?? target.skillsDir
    if (
      patch.skillsDir !== undefined &&
      db.targets.some(
        (candidate) =>
          candidate.id !== target.id && candidate.skillsDir === nextDir,
      )
    ) {
      throw new IpcError({
        code: 'DuplicatePath',
        message: 'Another target already uses this skills folder.',
      })
    }

    const dirChanged = nextDir !== target.skillsDir
    const modeChanged =
      patch.linkMode !== undefined && patch.linkMode !== target.linkMode
    target.name = patch.name?.trim() ?? target.name
    target.slug = patch.slug ?? target.slug
    target.skillsDir = nextDir
    target.linkMode = patch.linkMode ?? target.linkMode
    target.enabled = patch.enabled ?? target.enabled
    target.updatedAt = new Date().toISOString()

    if (dirChanged || modeChanged) {
      const now = new Date().toISOString()
      for (const skill of db.skills) {
        const install = findInstall(skill, target.id)
        if (!install) continue
        install.targetPath = `${target.skillsDir}/${skill.slug}`
        install.mode = target.linkMode
        install.actualKind = null
        install.linkedHash = null
        install.lastChecked = now
        install.status = dirChanged ? 'missing' : 'pending'
      }
      emitLibraryChanged()
    }

    logActivity({
      action: 'target',
      skillId: null,
      skillName: null,
      targetId: target.id,
      targetName: target.name,
      result: 'ok',
      detail: `更新目标 ${target.name}`,
    })
    return snapshot(target)
  },

  async deleteTarget(id: string, options: DeleteTargetOptions): Promise<void> {
    await wait(140)
    const target = db.targets.find((candidate) => candidate.id === id)
    if (!target) {
      throw new IpcError({ code: 'TargetNotFound', message: 'Target not found.' })
    }
    db.targets = db.targets.filter((candidate) => candidate.id !== id)
    let removed = 0
    for (const skill of db.skills) {
      const before = skill.installs.length
      skill.installs = skill.installs.filter(
        (install) => install.targetId !== id,
      )
      removed += before - skill.installs.length
    }
    logActivity({
      action: 'target',
      skillId: null,
      skillName: null,
      targetId: target.id,
      targetName: target.name,
      result: 'ok',
      detail: options.unlink
        ? `删除目标 ${target.name}，移除 ${removed} 条分发记录`
        : `删除目标 ${target.name}，外部目录未改动`,
    })
    emitLibraryChanged()
  },

  async pickDirectory(): Promise<string | null> {
    return null
  },

  async setInstall(
    skillId: string,
    targetId: string,
    input: InstallInput,
  ): Promise<Install | null> {
    await wait(120)
    const skill = requireSkill(skillId)
    const target = findTarget(targetId)
    if (!target) {
      throw new IpcError({ code: 'TargetNotFound', message: 'Target not found.' })
    }
    const existing = findInstall(skill, target.id)

    if (!input.enabled) {
      if (!existing) return null
      replaceInstall(skill.id, target.id, null)
      logActivity({
        action: 'unlink',
        skillId: skill.id,
        skillName: skill.slug,
        targetId: target.id,
        targetName: target.name,
        result: 'ok',
        detail: `取消 ${skill.slug} → ${target.name}`,
      })
      emitLibraryChanged()
      return null
    }

    if (existing?.status === 'conflict') {
      throw new IpcError({
        code: 'ConflictExists',
        message:
          'This target path contains an external directory. Resolve the conflict first.',
      })
    }

    const mode = input.mode ?? existing?.mode ?? target.linkMode
    const now = new Date().toISOString()
    if (existing) {
      existing.mode = mode
      existing.actualKind = mode
      existing.status = 'ok'
      existing.linkedHash = skill.contentHash
      existing.targetPath = `${target.skillsDir}/${skill.slug}`
      existing.lastError = null
      existing.lastChecked = now
      existing.lastSynced = now
      emitLibraryChanged()
      return snapshot(existing)
    }

    const install: Install = {
      id: db.nextInstallId,
      skillId: skill.id,
      targetId: target.id,
      mode,
      actualKind: mode,
      status: 'ok',
      targetPath: `${target.skillsDir}/${skill.slug}`,
      linkedHash: skill.contentHash,
      ignored: false,
      lastError: null,
      lastChecked: now,
      lastSynced: now,
    }
    db.nextInstallId += 1
    skill.installs.push(install)
    logActivity({
      action: mode === 'copy' ? 'copy' : 'link',
      skillId: skill.id,
      skillName: skill.slug,
      targetId: target.id,
      targetName: target.name,
      result: 'ok',
      detail: `${skill.slug} → ${target.name}`,
    })
    emitLibraryChanged()
    return snapshot(install)
  },

  async resolveInstall(
    skillId: string,
    targetId: string,
    resolution: ConflictResolution,
  ): Promise<Install | null> {
    await wait(120)
    const skill = requireSkill(skillId)
    const target = findTarget(targetId)
    const install = target ? findInstall(skill, target.id) : undefined
    if (!target || !install) {
      throw new IpcError({ code: 'InstallNotFound', message: 'Install not found.' })
    }

    if (resolution === 'cancel') {
      replaceInstall(skill.id, target.id, null)
      logActivity({
        action: 'conflict',
        skillId: skill.id,
        skillName: skill.slug,
        targetId: target.id,
        targetName: target.name,
        result: 'ok',
        detail: `取消 ${target.name} 的 Install，外部目录未改动`,
      })
      emitLibraryChanged()
      return null
    }

    if (resolution === 'ignore' || resolution === 'unignore') {
      install.ignored = resolution === 'ignore'
      logActivity({
        action: 'conflict',
        skillId: skill.id,
        skillName: skill.slug,
        targetId: target.id,
        targetName: target.name,
        result: 'ok',
        detail:
          resolution === 'ignore'
            ? `保留并忽略 ${target.name} 的冲突`
            : `取消忽略 ${target.name} 的冲突`,
      })
      emitLibraryChanged()
      return snapshot(install)
    }

    if (install.status !== 'conflict') {
      throw new IpcError({
        code: 'NotAConflict',
        message: 'This install is not a conflict.',
      })
    }
    install.mode = 'copy'
    install.actualKind = 'copy'
    install.status = 'ok'
    install.ignored = false
    install.linkedHash = skill.contentHash
    install.lastError = null
    const now = new Date().toISOString()
    install.lastChecked = now
    install.lastSynced = now
    logActivity({
      action: 'conflict',
      skillId: skill.id,
      skillName: skill.slug,
      targetId: target.id,
      targetName: target.name,
      result: 'ok',
      detail: `接管 ${target.name} 的外部目录为库副本`,
    })
    emitLibraryChanged()
    return snapshot(install)
  },

  async reconcile(): Promise<ReconcileReport> {
    await wait(380)
    const now = new Date().toISOString()
    let checked = 0
    let changed = 0
    for (const skill of db.skills) {
      for (const install of skill.installs) {
        checked += 1
        install.lastChecked = now
        if (
          install.status === 'ok' &&
          install.mode === 'copy' &&
          install.linkedHash !== skill.contentHash
        ) {
          install.status = 'stale'
          changed += 1
        }
      }
    }
    const report: ReconcileReport = {
      checked,
      changed,
      issues: countIssues(),
      checkedAt: now,
    }
    logActivity({
      action: 'reconcile',
      skillId: null,
      skillName: null,
      targetId: null,
      targetName: null,
      result: 'ok',
      detail: `核对 ${report.checked} 条记录 · ${report.issues} 项待处理`,
    })
    db.channels.reconcile.emit(report)
    emitLibraryChanged()
    return report
  },

  async planSync(filter) {
    await wait(150)
    return buildPlan(filter)
  },

  async applySync(planId: string) {
    return applyPlan(planId)
  },

  async cancelSync(planId: string): Promise<void> {
    cancelPlan(planId)
  },

  async listActivity(query?: ActivityQuery): Promise<ActivityPage> {
    await wait(80)
    const limit = query?.limit ?? 30
    const cursor = query?.cursor ? Number(query.cursor) : null
    const sorted = [...db.activity].sort(
      (a, b) => new Date(b.ts).getTime() - new Date(a.ts).getTime(),
    )
    const filtered =
      cursor === null ? sorted : sorted.filter((entry) => entry.id < cursor)
    const items = filtered.slice(0, limit)
    const last = items[items.length - 1]
    const hasMore = filtered.length > limit
    return snapshot({
      items,
      nextCursor: hasMore && last ? String(last.id) : null,
    })
  },

  async undoActivity(activityId: string) {
    await wait(120)
    const record = db.undo.get(activityId)
    if (!record) {
      return { ok: false, message: 'Nothing to undo.' }
    }
    if (record.kind === 'skill-delete' && record.skill) {
      if (!db.skills.some((skill) => skill.id === record.skill?.id)) {
        db.skills.push(snapshot(record.skill))
      }
    }
    if (record.kind === 'sync' && record.installs) {
      for (const change of record.installs) {
        replaceInstall(
          change.skillId,
          change.targetId,
          change.previous ? snapshot(change.previous) : null,
        )
      }
    }
    db.undo.delete(activityId)
    logActivity({
      action: 'restore',
      skillId: record.skill?.id ?? null,
      skillName: record.skill?.slug ?? null,
      targetId: null,
      targetName: null,
      result: 'ok',
      detail: '撤销上一步操作',
    })
    emitLibraryChanged()
    return { ok: true, message: null }
  },

  async listTargetIcons(): Promise<TargetIconAsset[]> {
    await wait(20)
    return MOCK_ICONS
  },

  async revealIconsDir(): Promise<void> {
    await wait(10)
  },

  async getSettings(): Promise<AppSettings> {
    await wait(50)
    return snapshot(db.settings)
  },

  async updateSettings(patch: Partial<AppSettings>): Promise<AppSettings> {
    await wait(100)
    db.settings = { ...db.settings, ...patch }
    return snapshot(db.settings)
  },

  subscribeSyncProgress(listener) {
    return db.channels.syncProgress.subscribe(listener)
  },

  subscribeLibraryChanged(listener) {
    return db.channels.library.subscribe(listener)
  },

  subscribeReconcileFinished(listener) {
    return db.channels.reconcile.subscribe(listener)
  },
}
