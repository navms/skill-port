import { ExternalLink, Pencil, RefreshCw, Trash2, X } from 'lucide-react'
import { useEffect, useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { Button } from '@/components/ui/button'
import { Tag } from '@/components/ui/status'
import { useI18n } from '@/i18n'
import { formatBytes, shortHash } from '@/lib/format'
import { useSkillDetail } from '@/lib/queries'
import { skillWorst, type DisplayState } from '@/lib/status'
import type { SkillSummary, Target, Tone } from '@/lib/types'
import { useUiStore } from '@/lib/uiStore'
import { cn } from '@/lib/utils'
import { ConflictDialog } from './ConflictDialog'
import { DeleteSkillDialog } from './DeleteSkillDialog'
import { DistributionSection } from './DistributionSection'
import { SkillEditDialog } from './SkillEditDialog'
import { SkillMdPreview } from './SkillMdPreview'
import { useSkillActions } from './useSkillActions'

interface SkillDetailPanelProps {
  skill: SkillSummary
  targets: readonly Target[]
  drawerOpen: boolean
}

function stateTag(worst: DisplayState): {
  labelKey:
    | 'detail.allGood'
    | 'detail.hasConflict'
    | 'detail.needsAttention'
  tone: Tone
} {
  if (worst === 'ok') return { labelKey: 'detail.allGood', tone: 'ok' }
  if (worst === 'conflict')
    return { labelKey: 'detail.hasConflict', tone: 'danger' }
  if (worst === 'none')
    return { labelKey: 'detail.needsAttention', tone: 'muted' }
  return { labelKey: 'detail.needsAttention', tone: 'warn' }
}

export function SkillDetailPanel({
  skill,
  targets,
  drawerOpen,
}: SkillDetailPanelProps) {
  const { t, language } = useI18n()
  const navigate = useNavigate()
  const openDialog = useUiStore((state) => state.openDialog)
  const setDetailDrawerOpen = useUiStore(
    (state) => state.setDetailDrawerOpen,
  )

  const detailQuery = useSkillDetail(skill.id)
  const detail = detailQuery.data
  const installs = detail?.installs ?? skill.installs
  const worst = skillWorst(installs, targets)
  const tag = stateTag(worst)

  const [editOpen, setEditOpen] = useState(false)
  const [deleteOpen, setDeleteOpen] = useState(false)
  const [conflictTarget, setConflictTarget] = useState<Target | null>(null)

  const { toggleInstall, resolve, reveal, pendingTargetId } = useSkillActions(
    skill,
    targets,
  )

  const conflictInstall = conflictTarget
    ? installs.find((install) => install.targetId === conflictTarget.id)
    : undefined

  // The drawer mounts at translate-x-full and only slides in on the next
  // frame: a transition cannot run from the class an element mounts with.
  const [entered, setEntered] = useState(false)
  useEffect(() => {
    const frame = requestAnimationFrame(() => setEntered(true))
    return () => cancelAnimationFrame(frame)
  }, [])
  const visible = entered && drawerOpen

  return (
    <aside
      aria-label={t('detail.eyebrow')}
      aria-hidden={!visible}
      inert={!visible}
      className={cn(
        'fixed top-11 right-0 bottom-0 z-40 flex w-[min(400px,92vw)] flex-col border-l border-line bg-panel shadow-[-24px_0_60px_-30px_var(--sp-scrim)] transition-transform duration-200 ease-out',
        visible ? 'translate-x-0' : 'translate-x-full',
      )}
    >
      <div className="flex min-h-0 flex-1 flex-col overflow-y-auto p-5 pb-6 [scrollbar-gutter:stable]">
        <div className="flex items-center gap-2">
          <h2 className="min-w-0 flex-1 text-[19px] leading-tight font-semibold tracking-tight break-words text-ink">
            {detail?.name ?? skill.name}
          </h2>
          <Button
            variant="secondary"
            size="icon"
            onClick={() => reveal.mutate()}
            aria-label={t('detail.reveal')}
            title={t('detail.reveal')}
            disabled={reveal.isPending}
          >
            <ExternalLink size={14} aria-hidden />
          </Button>
          <Button
            variant="secondary"
            size="icon"
            onClick={() => setDetailDrawerOpen(false)}
            aria-label={t('detail.close')}
          >
            <X size={14} aria-hidden />
          </Button>
        </div>

        <p className="mt-2 text-[13px] leading-relaxed text-muted">
          {detail?.description ?? skill.description}
        </p>

        <div className="mt-2.5 flex flex-wrap gap-1.5">
          <Tag>{skill.category ?? '—'}</Tag>
          <Tag tone={tag.tone}>{t(tag.labelKey)}</Tag>
          <Tag mono>{formatBytes(skill.sizeBytes)}</Tag>
        </div>

        <div className="mt-4 flex gap-2">
          <Button
            variant="primary"
            className="flex-1"
            onClick={() =>
              openDialog({
                kind: 'sync',
                skillId: skill.id,
                skillName: skill.slug,
              })
            }
          >
            <RefreshCw size={13} aria-hidden />
            {worst === 'ok' ? t('detail.resync') : t('detail.syncNow')}
          </Button>
          <Button
            variant="secondary"
            size="icon"
            onClick={() => setDeleteOpen(true)}
            aria-label={t('detail.deleteSkill')}
            title={t('detail.deleteSkill')}
            className="text-faint hover:text-danger"
          >
            <Trash2 size={14} aria-hidden />
          </Button>
        </div>

        <DistributionSection
          installs={installs}
          targets={targets}
          language={language}
          pendingTargetId={pendingTargetId}
          onToggle={(target, enabled) =>
            toggleInstall.mutate({ targetId: target.id, enabled })
          }
          onResolve={setConflictTarget}
          onUnignore={(target) =>
            resolve.mutate({ targetId: target.id, resolution: 'unignore' })
          }
          onManageTargets={() => navigate('/targets')}
        />

        <section className="mt-5 flex min-h-0 flex-1 flex-col">
          <div className="flex items-center justify-between border-b border-line pb-1.5">
            <h3 className="font-mono text-[10px] font-medium tracking-[0.13em] text-faint uppercase">
              {t('detail.skillMd')}
            </h3>
            <div className="flex items-center gap-2">
              <span className="font-mono text-[10px] text-faint">
                {shortHash(skill.contentHash)}
              </span>
              <button
                type="button"
                onClick={() => setEditOpen(true)}
                disabled={!detail}
                aria-label={t('detail.editSkill')}
                title={t('detail.editSkill')}
                className="rounded p-0.5 text-faint transition-colors hover:text-ink disabled:opacity-40"
              >
                <Pencil size={12} aria-hidden />
              </button>
            </div>
          </div>
          {detailQuery.isLoading ? (
            <p className="py-3 text-xs text-muted">{t('detail.loading')}</p>
          ) : detailQuery.isError ? (
            <p role="alert" className="py-3 text-xs text-danger">
              {t('detail.loadError')}
            </p>
          ) : detail ? (
            <SkillMdPreview source={detail.skillMd} />
          ) : null}
        </section>
      </div>

      {detail && editOpen ? (
        <SkillEditDialog
          skill={detail}
          onOpenChange={(open) => {
            if (!open) setEditOpen(false)
          }}
        />
      ) : null}

      {deleteOpen ? (
        <DeleteSkillDialog
          skill={skill}
          onOpenChange={(open) => {
            if (!open) setDeleteOpen(false)
          }}
        />
      ) : null}

      {conflictTarget && conflictInstall ? (
        <ConflictDialog
          skill={skill}
          target={conflictTarget}
          install={conflictInstall}
          onOpenChange={(open) => {
            if (!open) setConflictTarget(null)
          }}
        />
      ) : null}
    </aside>
  )
}
