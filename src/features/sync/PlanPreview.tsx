import { useI18n } from '@/i18n'
import {
  SYNC_KIND_ORDER,
  groupActions,
  type PlanSummary,
} from '@/lib/plan'
import { TONE_TEXT, stateCopy } from '@/lib/status'
import type { SyncAction } from '@/lib/types'
import { cn } from '@/lib/utils'

const KIND_TONE = {
  create: 'text-ok',
  repair: 'text-warn',
  skip: 'text-faint',
  conflict: 'text-danger',
} as const

function ActionRow({ action }: { action: SyncAction }) {
  const { t, language } = useI18n()
  return (
    <div className="flex items-start gap-3 border-b border-line px-3 py-2 last:border-b-0">
      <div className="min-w-0 flex-1">
        <div className="flex min-w-0 items-center gap-1.5 text-[12px]">
          <b className="truncate font-medium text-ink-2">{action.skillSlug}</b>
          <span aria-hidden className="text-faint">
            →
          </span>
          <span className="truncate text-muted">{action.targetName}</span>
          <span className="shrink-0 rounded-[3px] border border-line px-1 py-px font-mono text-[9px] tracking-[0.06em] text-faint uppercase">
            {action.mode === 'copy' ? 'COPY' : 'LINK'}
          </span>
        </div>
        <div
          title={action.targetPath}
          className="mt-0.5 truncate font-mono text-[10.5px] text-faint"
        >
          {action.targetPath}
        </div>
        {action.reason ? (
          <p className="mt-0.5 text-[11px] leading-relaxed text-muted">
            {action.reason}
          </p>
        ) : null}
      </div>
      <span
        className={cn(
          'shrink-0 pt-0.5 text-[11px]',
          TONE_TEXT[
            action.kind === 'conflict'
              ? 'danger'
              : action.kind === 'skip'
                ? 'muted'
                : action.kind === 'repair'
                  ? 'warn'
                  : 'ok'
          ],
        )}
      >
        {action.kind === 'conflict'
          ? t('sync.kind.conflict')
          : action.kind === 'skip'
            ? t('sync.kind.skip')
            : stateCopy(action.state, action.mode, language)}
      </span>
    </div>
  )
}

export function PlanSummaryChips({ summary }: { summary: PlanSummary }) {
  const { t } = useI18n()
  return (
    <div className="flex flex-wrap gap-1.5">
      {SYNC_KIND_ORDER.map((kind) =>
        summary[kind] > 0 ? (
          <span
            key={kind}
            className={cn(
              'inline-flex h-6 items-center gap-1.5 rounded border border-line px-1.5 text-[11px]',
              KIND_TONE[kind],
            )}
          >
            {t(`sync.group.${kind}`)}
            <span className="font-mono opacity-70">{summary[kind]}</span>
          </span>
        ) : null,
      )}
    </div>
  )
}

export function PlanGroups({ actions }: { actions: readonly SyncAction[] }) {
  const { t } = useI18n()
  const groups = groupActions(actions)

  return (
    <div className="max-h-[42vh] space-y-3 overflow-auto">
      {SYNC_KIND_ORDER.map((kind) => {
        const items = groups[kind]
        if (items.length === 0) return null
        return (
          <section key={kind}>
            <h3
              className={cn(
                'mb-1 font-mono text-[10px] tracking-[0.12em] uppercase',
                KIND_TONE[kind],
              )}
            >
              {t(`sync.group.${kind}`)}
            </h3>
            <div className="rounded-md border border-line">
              {items.map((action) => (
                <ActionRow key={action.id} action={action} />
              ))}
            </div>
          </section>
        )
      })}
    </div>
  )
}
