import { useI18n } from '@/i18n'
import { displayPath, formatRelative } from '@/lib/format'
import {
  TONE_PIP,
  TONE_TEXT,
  coverage,
  installCopy,
  installFor,
  installTone,
  issueCount,
  skillWorst,
  stateCopy,
  toneForState,
} from '@/lib/status'
import type { SkillSummary, Target } from '@/lib/types'
import { cn } from '@/lib/utils'

interface SkillTableProps {
  skills: readonly SkillSummary[]
  targets: readonly Target[]
  selectedSlug: string | null
  libraryPath?: string | null
  onSelect: (skill: SkillSummary) => void
}

export function SkillTable({
  skills,
  targets,
  selectedSlug,
  libraryPath,
  onSelect,
}: SkillTableProps) {
  const { t, language } = useI18n()
  const enabledTargets = targets.filter((target) => target.enabled)
  const enabledIds = new Set(enabledTargets.map((target) => target.id))

  return (
    <div className="min-h-0 flex-1 overflow-auto">
      {/* The header lives inside the scroller and sticks, so its columns line
          up with the rows even when a scrollbar takes width. */}
      <div
        aria-hidden
        className="sticky top-0 z-10 grid grid-cols-[minmax(0,1fr)_104px_104px] gap-3 border-b border-line bg-bg px-6 pb-1.5 font-mono text-[10px] tracking-[0.12em] text-faint uppercase min-[1100px]:grid-cols-[minmax(0,1fr)_minmax(0,200px)_104px_104px]"
      >
        <span>{t('library.tableSkill')}</span>
        <span className="hidden truncate min-[1100px]:block">
          {t('library.tableSource')}
        </span>
        <span>{t('library.tableCoverage')}</span>
        <span className="text-right">{t('library.tableModified')}</span>
      </div>

      <div role="group" aria-label={t('library.tableSkill')}>
        {skills.map((skill) => {
          const worst = skillWorst(skill.installs, targets)
          const issues = issueCount(skill.installs, targets)
          const linked = coverage(skill.installs, targets)
          const sourcePath = libraryPath
            ? `${libraryPath}/${skill.slug}`
            : `library/${skill.slug}`
          const isSelected = skill.slug === selectedSlug
          const actionable = skill.installs.filter(
            (install) =>
              enabledIds.has(install.targetId) &&
              !install.ignored &&
              install.status !== 'ok',
          )
          const conflictCount = actionable.filter(
            (install) => install.status === 'conflict',
          ).length
          const errorCount = actionable.filter(
            (install) => install.status === 'error',
          ).length

          let issueText = ''
          if (worst === 'conflict') {
            issueText = t('library.issueConflict', { count: conflictCount })
          } else if (worst === 'error') {
            issueText = t('library.issueError', { count: errorCount })
          } else if (worst === 'stale') {
            issueText = t('library.issueStale')
          } else if (worst === 'missing') {
            issueText = t('library.issueMissing')
          } else if (worst === 'pending') {
            issueText = t('library.issuePending', { count: issues })
          }

          return (
            <button
              key={skill.id}
              type="button"
              aria-current={isSelected ? 'true' : undefined}
              onClick={() => onSelect(skill)}
              className={cn(
                'relative grid w-full grid-cols-[minmax(0,1fr)_104px_104px] items-center gap-3 border-b border-line px-6 py-3 text-left transition-colors hover:bg-row-hover min-[1100px]:grid-cols-[minmax(0,1fr)_minmax(0,200px)_104px_104px]',
                isSelected &&
                  'bg-row-selected before:absolute before:top-2 before:bottom-2 before:left-0 before:w-0.5 before:rounded-r before:bg-accent',
              )}
            >
              <div className="flex min-w-0 items-center gap-2.5">
                <span
                  aria-hidden
                  className={cn(
                    'size-1.5 shrink-0 rounded-full',
                    TONE_PIP[toneForState(worst)],
                  )}
                />
                <div className="min-w-0">
                  <b className="block truncate text-sm font-medium tracking-tight text-ink">
                    {skill.slug}
                  </b>
                  <small className="mt-0.5 block truncate text-[13px] text-muted">
                    {skill.description}
                  </small>
                </div>
              </div>

              <div className="hidden min-w-0 min-[1100px]:block">
                <span
                  title={sourcePath}
                  className="block truncate font-mono text-[11px] text-faint"
                >
                  {displayPath(sourcePath)}
                </span>
              </div>

              <div className="flex min-w-0 items-center">
                <span
                  aria-hidden
                  className="flex h-4 items-center gap-1"
                >
                  {enabledTargets.map((target) => {
                    const install = installFor(skill.installs, target.id)
                    const tone = install ? installTone(install) : 'muted'
                    const label = install
                      ? installCopy(install, language)
                      : stateCopy('none', target.linkMode, language)
                    return (
                      <span
                        key={target.id}
                        title={`${target.name} · ${label}`}
                        className={cn(
                          'h-3.5 w-1 rounded-[1px]',
                          TONE_PIP[tone],
                        )}
                      />
                    )
                  })}
                </span>
                <span className="ml-2.5 font-mono text-[11px] text-muted">
                  {linked} / {enabledTargets.length}
                </span>
              </div>

              <div className="text-right font-mono text-[11px] text-faint">
                <span>{formatRelative(skill.updatedAt, language)}</span>
                {issueText ? (
                  <span
                    className={cn(
                      'mt-0.5 block text-[11px]',
                      TONE_TEXT[toneForState(worst)],
                    )}
                  >
                    {issueText}
                  </span>
                ) : null}
              </div>
            </button>
          )
        })}
      </div>
    </div>
  )
}
