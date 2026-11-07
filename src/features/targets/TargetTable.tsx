import { MoreHorizontal, Pencil, Trash2 } from 'lucide-react'
import { useMemo } from 'react'
import {
  DropdownItem,
  DropdownMenu,
  DropdownSeparator,
} from '@/components/ui/dropdown'
import { Switch } from '@/components/ui/switch'
import { Tag } from '@/components/ui/status'
import { useI18n } from '@/i18n'
import {
  TONE_DOT,
  TONE_TEXT,
  targetDistributionCount,
  targetIssueCount,
  targetWorst,
  toneForState,
} from '@/lib/status'
import type { SkillSummary, Target } from '@/lib/types'
import { cn } from '@/lib/utils'
import { TargetIcon } from './TargetIcon'

interface TargetTableProps {
  targets: readonly Target[]
  skills: readonly SkillSummary[]
  pendingId: string | null
  onToggle: (target: Target, enabled: boolean) => void
  onOpenDir: (target: Target) => void
  onEdit: (target: Target) => void
  onDelete: (target: Target) => void
}

const TH =
  'border-b border-line px-3 pb-1.5 font-mono text-[10px] font-normal tracking-[0.12em] whitespace-nowrap text-faint uppercase'

export function TargetTable({
  targets,
  skills,
  pendingId,
  onToggle,
  onOpenDir,
  onEdit,
  onDelete,
}: TargetTableProps) {
  const { t } = useI18n()
  const installs = useMemo(
    () => skills.flatMap((skill) => skill.installs),
    [skills],
  )

  return (
    <div className="min-h-0 flex-1 overflow-auto px-6">
      {/*
        Fixed layout: the directory column takes the leftover width, every
        other column has a width wide enough for its header and its widest
        value, so nothing wraps when the window is narrow. Below 1100px the
        distribution count (derivable from the status column) drops out first.
      */}
      <table className="w-full table-fixed border-collapse text-left text-[12.5px]">
        <thead>
          <tr>
            <th scope="col" className={cn(TH, 'w-[28%] pl-0')}>
              {t('targets.colTarget')}
            </th>
            <th scope="col" className={TH}>
              {t('targets.colDir')}
            </th>
            <th scope="col" className={cn(TH, 'w-20')}>
              {t('targets.colMode')}
            </th>
            <th
              scope="col"
              className={cn(TH, 'w-[96px] max-[1100px]:hidden')}
            >
              {t('targets.colDistributed')}
            </th>
            <th scope="col" className={cn(TH, 'w-[15%]')}>
              {t('targets.colStatus')}
            </th>
            <th scope="col" className={cn(TH, 'w-[84px] text-center')}>
              {t('common.enabled')}
            </th>
            <th scope="col" className={cn(TH, 'w-10 pr-0')}>
              <span className="sr-only">{t('common.actions')}</span>
            </th>
          </tr>
        </thead>
        <tbody>
          {targets.map((target) => {
            const distributed = targetDistributionCount(
              installs,
              target.id,
            )
            const issues = targetIssueCount(installs, target.id)
            const worst = targetWorst(installs, target.id)
            const tone = toneForState(worst)

            let statusText: string
            let statusTone = tone
            if (!target.enabled) {
              statusText = t('targets.statusDisabled')
              statusTone = 'muted'
            } else if (distributed === 0) {
              statusText = t('targets.statusNoLinks')
              statusTone = 'muted'
            } else if (issues > 0) {
              statusText = t('targets.statusIssues', { count: issues })
            } else {
              statusText = t('targets.statusOk')
              statusTone = 'ok'
            }

            return (
              <tr
                key={target.id}
                className="border-b border-line last:border-b-0"
              >
                <td className="py-2.5 pr-3 pl-0 align-middle">
                  <div className="flex min-w-0 items-center gap-2.5">
                    <TargetIcon slug={target.slug} />
                    <div className="min-w-0">
                      <div className="flex min-w-0 items-center gap-2">
                        <b
                          title={target.name}
                          className="truncate text-[13px] font-medium text-ink"
                        >
                          {target.name}
                        </b>
                        <Tag className="h-5 shrink-0 px-1 text-[10px]">
                          {target.detected
                            ? t('targets.detected')
                            : t('targets.manual')}
                        </Tag>
                      </div>
                      <div
                        title={target.slug}
                        className="mt-0.5 truncate font-mono text-[10.5px] text-faint"
                      >
                        {target.slug}
                      </div>
                    </div>
                  </div>
                </td>

                <td className="relative py-2.5 align-middle">
                  {/*
                    The whole cell is the hit target: a path is often wider
                    than its column, so a text-only link would be awkward to
                    hit. The dotted rule marks it as a link without adding
                    an icon that would shift on hover.
                  */}
                  <button
                    type="button"
                    onClick={() => onOpenDir(target)}
                    aria-label={t('targets.openDir', {
                      path: target.skillsDir,
                    })}
                    title={t('targets.openDir', { path: target.skillsDir })}
                    className="group/dir absolute inset-0 flex items-center px-3 text-left font-mono text-[11px] text-muted transition-colors hover:text-accent focus-visible:text-accent"
                  >
                    <span className="min-w-0 truncate underline decoration-line-2 decoration-dotted underline-offset-[3px] transition-colors group-hover/dir:decoration-accent">
                      {target.skillsDir}
                    </span>
                  </button>
                </td>

                <td className="py-2.5 pr-3 pl-3 align-middle">
                  <span className="rounded-[3px] border border-line px-1 py-0.5 font-mono text-[9.5px] tracking-[0.08em] whitespace-nowrap text-faint">
                    {target.linkMode === 'copy'
                      ? t('targets.modeCopyShort')
                      : t('targets.modeSymlinkShort')}
                  </span>
                </td>

                <td className="py-2.5 pr-3 pl-3 align-middle max-[1100px]:hidden">
                  <span className="font-mono text-[11px] whitespace-nowrap text-ink-2">
                    {distributed}
                  </span>
                  <span className="font-mono text-[10.5px] whitespace-nowrap text-faint">
                    {' / '}
                    {skills.length}
                  </span>
                </td>

                <td className="py-2.5 pr-3 pl-3 align-middle">
                  <span
                    className={cn(
                      'inline-flex items-center gap-1.5 text-[11.5px] whitespace-nowrap',
                      TONE_TEXT[statusTone],
                    )}
                  >
                    <span
                      aria-hidden
                      className={cn(
                        'size-1.5 rounded-full',
                        TONE_DOT[statusTone],
                      )}
                    />
                    {statusText}
                  </span>
                </td>

                <td className="py-2.5 pr-3 pl-3 text-center align-middle">
                  <Switch
                    label={t('targets.toggleEnable', { name: target.name })}
                    checked={target.enabled}
                    disabled={pendingId === target.id}
                    onCheckedChange={(checked) => onToggle(target, checked)}
                  />
                </td>

                <td className="py-2.5 pr-0 text-right align-middle">
                  <DropdownMenu
                    trigger={
                      <button
                        type="button"
                        aria-label={`${t('common.more')} · ${target.name}`}
                        className="inline-flex size-7 items-center justify-center rounded-md text-faint transition-colors hover:bg-raise hover:text-ink"
                      >
                        <MoreHorizontal size={14} aria-hidden />
                      </button>
                    }
                  >
                    <DropdownItem
                      icon={<Pencil size={13} aria-hidden />}
                      onSelect={() => onEdit(target)}
                    >
                      {t('targets.edit')}
                    </DropdownItem>
                    <DropdownSeparator />
                    <DropdownItem
                      icon={<Trash2 size={13} aria-hidden />}
                      danger
                      onSelect={() => onDelete(target)}
                    >
                      {t('targets.delete')}
                    </DropdownItem>
                  </DropdownMenu>
                </td>
              </tr>
            )
          })}
        </tbody>
      </table>
    </div>
  )
}
