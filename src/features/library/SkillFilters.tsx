import { ChevronDown } from 'lucide-react'
import {
  DropdownItem,
  DropdownLabel,
  DropdownMenu,
  DropdownSeparator,
} from '@/components/ui/dropdown'
import { useI18n } from '@/i18n'
import { copiedSkillCount, filterSkills } from '@/lib/filter'
import {
  TONE_DOT,
  issueCount,
  targetWorst,
  toneForState,
} from '@/lib/status'
import type { SkillSummary, Target } from '@/lib/types'
import { useUiStore } from '@/lib/uiStore'
import { cn } from '@/lib/utils'

const CHIP =
  'inline-flex h-8 items-center gap-1.5 rounded-[5px] border px-2.5 text-xs whitespace-nowrap transition-colors'
const CHIP_ACTIVE = 'border-line-2 bg-raise text-ink'
const CHIP_IDLE =
  'border-transparent text-muted hover:bg-panel-2 hover:text-ink-2'

interface SkillFiltersProps {
  skills: readonly SkillSummary[]
  targets: readonly Target[]
}

export function SkillFilters({ skills, targets }: SkillFiltersProps) {
  const { t } = useI18n()
  const chip = useUiStore((state) => state.chip)
  const setChip = useUiStore((state) => state.setChip)
  const category = useUiStore((state) => state.category)
  const query = useUiStore((state) => state.query)
  const targetFilter = useUiStore((state) => state.targetFilter)
  const setTargetFilter = useUiStore((state) => state.setTargetFilter)

  const enabledTargets = targets.filter((target) => target.enabled)
  const selectedTarget = targets.find((target) => target.id === targetFilter)
  const allInstalls = skills.flatMap((skill) => skill.installs)
  const issueSkills = skills.filter(
    (skill) => issueCount(skill.installs, targets) > 0,
  ).length
  const copied = copiedSkillCount(skills, targets)
  const visible = filterSkills(skills, targets, {
    query,
    chip,
    category,
    targetId: targetFilter,
  })

  const note = category
    ? t('library.filterNoteCategory', {
        category,
        count: visible.length,
      })
    : targetFilter && selectedTarget
      ? t('library.filterNoteTarget', { target: selectedTarget.name })
      : null

  return (
    <div className="flex shrink-0 flex-wrap items-center gap-1.5 px-6 pb-3">
      <button
        type="button"
        onClick={() => setChip('all')}
        className={cn(CHIP, chip === 'all' ? CHIP_ACTIVE : CHIP_IDLE)}
      >
        {t('library.filterAll')}
        <span className="font-mono text-[10px] text-faint">{skills.length}</span>
      </button>
      <button
        type="button"
        onClick={() => setChip('issues')}
        className={cn(CHIP, chip === 'issues' ? CHIP_ACTIVE : CHIP_IDLE)}
      >
        <span aria-hidden className="size-1.5 rounded-full bg-warn" />
        {t('library.filterIssues')}
        <span className="font-mono text-[10px] text-faint">{issueSkills}</span>
      </button>
      <button
        type="button"
        onClick={() => setChip('copied')}
        className={cn(CHIP, chip === 'copied' ? CHIP_ACTIVE : CHIP_IDLE)}
      >
        {t('library.filterCopied')}
        <span className="font-mono text-[10px] text-faint">{copied}</span>
      </button>

      <DropdownMenu
        align="start"
        className="min-w-[240px]"
        trigger={
          <button
            type="button"
            aria-label={t('library.filterTargetMenu')}
            className={cn(
              CHIP,
              targetFilter ? CHIP_ACTIVE : CHIP_IDLE,
            )}
          >
            {selectedTarget ? (
              <span
                aria-hidden
                className={cn(
                  'size-1.5 rounded-full',
                  selectedTarget.enabled
                    ? TONE_DOT[
                        toneForState(
                          targetWorst(allInstalls, selectedTarget.id),
                        )
                      ]
                    : 'bg-faint',
                )}
              />
            ) : null}
            {selectedTarget?.name ?? t('library.filterAllTargets')}
            <span className="font-mono text-[10px] text-faint">
              {selectedTarget
                ? skills.filter((skill) =>
                    skill.installs.some(
                      (install) => install.targetId === selectedTarget.id,
                    ),
                  ).length
                : enabledTargets.length}
            </span>
            <ChevronDown size={11} aria-hidden />
          </button>
        }
      >
        <DropdownItem
          onSelect={() => setTargetFilter(null)}
          className={cn(!targetFilter && 'bg-raise text-ink')}
        >
          {t('library.filterAllTargets')}
          <span className="ml-auto font-mono text-[10px] text-faint">
            {skills.length}
          </span>
        </DropdownItem>
        <DropdownSeparator />
        <DropdownLabel>{t('nav.targets')}</DropdownLabel>
        {targets.map((target) => {
          const count = skills.filter((skill) =>
            skill.installs.some((install) => install.targetId === target.id),
          ).length
          return (
            <DropdownItem
              key={target.id}
              onSelect={() => setTargetFilter(target.id)}
              className={cn(
                targetFilter === target.id && 'bg-raise text-ink',
                !target.enabled && 'opacity-55',
              )}
            >
              <span
                aria-hidden
                className={cn(
                  'size-1.5 rounded-full',
                  target.enabled
                    ? TONE_DOT[
                        toneForState(targetWorst(allInstalls, target.id))
                      ]
                    : 'bg-faint',
                )}
              />
              {target.name}
              <span className="ml-auto font-mono text-[10px] text-faint">
                {target.enabled ? count : t('library.targetOff')}
              </span>
            </DropdownItem>
          )
        })}
      </DropdownMenu>

      <div className="ml-auto hidden items-center gap-1.5 font-mono text-[10.5px] whitespace-nowrap text-faint md:flex">
        <span
          aria-hidden
          className="size-1.5 rounded-full bg-ok"
        />
        {note}
      </div>
    </div>
  )
}
