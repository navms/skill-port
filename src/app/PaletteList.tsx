import { useEffect, type ReactNode, type RefObject } from 'react'
import { useI18n } from '@/i18n'
import { coverage, stateCopy, skillWorst } from '@/lib/status'
import type { SkillSummary, Target } from '@/lib/types'
import { cn } from '@/lib/utils'
import type { ActionItem, PaletteItem } from './paletteTypes'

interface PaletteListProps {
  actions: ActionItem[]
  skills: SkillSummary[]
  targets: Target[]
  active: number
  listRef: RefObject<HTMLDivElement | null>
  onRun: (item: PaletteItem) => void
  onHover: (index: number) => void
}

export function PaletteList({
  actions,
  skills,
  targets,
  active,
  listRef,
  onRun,
  onHover,
}: PaletteListProps) {
  const { t, language } = useI18n()
  const enabledCount = targets.filter((target) => target.enabled).length

  useEffect(() => {
    const node = listRef.current?.querySelector<HTMLElement>(
      '[data-active="true"]',
    )
    node?.scrollIntoView({ block: 'nearest' })
  }, [active, listRef])

  return (
    <div
      ref={listRef}
      id="palette-list"
      role="listbox"
      aria-label={t('palette.placeholder')}
      className="min-h-0 flex-1 overflow-auto p-1.5"
    >
      {actions.length === 0 && skills.length === 0 ? (
        <p className="px-3 py-8 text-center text-xs text-faint">
          {t('palette.empty')}
        </p>
      ) : (
        <>
          {actions.length > 0 ? (
            <div className="px-2 pt-1 pb-1 font-mono text-[9.5px] tracking-[0.14em] text-faint uppercase">
              {t('palette.sectionActions')}
            </div>
          ) : null}
          {actions.map((item, index) => (
            <PaletteRow
              key={`action-${item.id}`}
              id={`palette-item-${index}`}
              active={index === active}
              icon={item.icon}
              label={item.label}
              sub={item.sub}
              onSelect={() => onRun(item)}
              onHover={() => onHover(index)}
            />
          ))}

          {skills.length > 0 ? (
            <div className="px-2 pt-2 pb-1 font-mono text-[9.5px] tracking-[0.14em] text-faint uppercase">
              {t('palette.sectionSkills')}
            </div>
          ) : null}
          {skills.map((skill, offset) => {
            const index = actions.length + offset
            const worst = skillWorst(skill.installs, targets)
            const mode =
              skill.installs.find((i) => i.status === worst)?.mode ??
              skill.installs[0]?.mode ??
              'symlink'
            return (
              <PaletteRow
                key={`skill-${skill.id}`}
                id={`palette-item-${index}`}
                active={index === active}
                icon={<span className="text-faint">/</span>}
                label={skill.slug}
                sub={t('palette.skillSub', {
                  linked: coverage(skill.installs, targets),
                  total: enabledCount,
                  state: stateCopy(worst, mode, language),
                })}
                onSelect={() =>
                  onRun({ kind: 'skill', id: skill.id, skill })
                }
                onHover={() => onHover(index)}
              />
            )
          })}
        </>
      )}
    </div>
  )
}

interface PaletteRowProps {
  id: string
  active: boolean
  icon: ReactNode
  label: string
  sub: string
  onSelect: () => void
  onHover: () => void
}

function PaletteRow({
  id,
  active,
  icon,
  label,
  sub,
  onSelect,
  onHover,
}: PaletteRowProps) {
  return (
    <button
      id={id}
      type="button"
      role="option"
      aria-selected={active}
      data-active={active}
      tabIndex={-1}
      onMouseMove={onHover}
      onClick={onSelect}
      className={cn(
        'flex w-full items-center gap-2.5 rounded-md px-2 py-2 text-left',
        active ? 'bg-raise' : 'hover:bg-panel-2',
      )}
    >
      <span className="flex size-5 shrink-0 items-center justify-center rounded border border-line text-muted">
        {icon}
      </span>
      <span className="min-w-0 flex-1">
        <span className="block truncate text-[12.5px] text-ink-2">
          {label}
        </span>
        <span className="block truncate text-[11px] text-faint">{sub}</span>
      </span>
    </button>
  )
}
