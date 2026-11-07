import { useInfiniteQuery } from '@tanstack/react-query'
import { Activity } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Dialog } from '@/components/ui/dialog'
import { EmptyState } from '@/components/ui/empty-state'
import { LoadingBlock } from '@/components/ui/spinner'
import { useI18n, type MessageKey } from '@/i18n'
import { api } from '@/lib/api'
import { toErrorMessage } from '@/lib/errors'
import { formatDateTime, formatRelative } from '@/lib/format'
import { queryKeys } from '@/lib/queryKeys'
import type { ActivityAction } from '@/lib/types'
import { cn } from '@/lib/utils'

const ACTION_KEYS: Record<ActivityAction, MessageKey> = {
  import: 'activity.action.import',
  link: 'activity.action.link',
  copy: 'activity.action.copy',
  unlink: 'activity.action.unlink',
  sync: 'activity.action.sync',
  reconcile: 'activity.action.reconcile',
  conflict: 'activity.action.conflict',
  delete: 'activity.action.delete',
  update: 'activity.action.update',
  target: 'activity.action.target',
  restore: 'activity.action.restore',
}

interface ActivityDialogProps {
  onOpenChange: (open: boolean) => void
}

export function ActivityDialog({ onOpenChange }: ActivityDialogProps) {
  const { t, language } = useI18n()

  const query = useInfiniteQuery({
    queryKey: queryKeys.activity,
    queryFn: ({ pageParam }) =>
      api.listActivity({ limit: 20, cursor: pageParam }),
    initialPageParam: null as string | null,
    getNextPageParam: (last) => last.nextCursor,
    staleTime: 10_000,
  })

  const entries = query.data?.pages.flatMap((page) => page.items) ?? []

  return (
    <Dialog
      open
      onOpenChange={(open) => {
        if (!open) onOpenChange(false)
      }}
      title={t('activity.title')}
      description={t('activity.subtitle')}
      className="w-[min(700px,94vw)]"
      footer={
        <>
          {query.hasNextPage ? (
            <Button
              variant="secondary"
              onClick={() => void query.fetchNextPage()}
              disabled={query.isFetchingNextPage}
              className="mr-auto"
            >
              {query.isFetchingNextPage
                ? t('common.loading')
                : t('activity.loadMore')}
            </Button>
          ) : null}
          <Button variant="primary" onClick={() => onOpenChange(false)}>
            {t('common.done')}
          </Button>
        </>
      }
    >
      {query.isLoading ? (
        <LoadingBlock label={t('common.loading')} />
      ) : query.isError ? (
        <div className="py-8 text-center">
          <p className="text-sm font-medium text-ink-2">
            {t('activity.loadError')}
          </p>
          <p className="mt-1 text-xs text-muted">
            {toErrorMessage(query.error)}
          </p>
          <Button
            variant="secondary"
            size="sm"
            className="mt-3"
            onClick={() => void query.refetch()}
          >
            {t('common.retry')}
          </Button>
        </div>
      ) : entries.length === 0 ? (
        <EmptyState
          icon={<Activity size={17} aria-hidden />}
          title={t('activity.empty')}
        />
      ) : (
        <div className="rounded-md border border-line">
          {entries.map((entry) => (
            <div
              key={entry.id}
              className="grid grid-cols-[88px_minmax(0,1fr)_56px] items-start gap-3 border-b border-line px-3 py-2 last:border-b-0"
            >
              <time
                dateTime={entry.ts}
                title={formatDateTime(entry.ts, language)}
                className="pt-0.5 font-mono text-[10.5px] whitespace-nowrap text-faint"
              >
                {formatRelative(entry.ts, language)}
              </time>

              <div className="min-w-0">
                <div className="flex min-w-0 items-center gap-1.5 text-[12px]">
                  <span className="shrink-0 font-medium text-ink-2">
                    {t(ACTION_KEYS[entry.action])}
                  </span>
                  {entry.skillName ? (
                    <>
                      <span aria-hidden className="text-faint">
                        ·
                      </span>
                      <span className="truncate font-mono text-[11px] text-muted">
                        {entry.skillName}
                      </span>
                    </>
                  ) : null}
                  {entry.targetName ? (
                    <>
                      <span aria-hidden className="text-faint">
                        →
                      </span>
                      <span className="truncate text-[11px] text-muted">
                        {entry.targetName}
                      </span>
                    </>
                  ) : null}
                </div>
                {entry.detail ? (
                  <p className="mt-0.5 truncate text-[11px] text-faint">
                    {entry.detail}
                  </p>
                ) : null}
              </div>

              <span
                className={cn(
                  'justify-self-end rounded-[3px] border px-1 py-px font-mono text-[9px] tracking-[0.08em] whitespace-nowrap',
                  entry.result === 'ok'
                    ? 'border-ok-soft bg-ok-soft text-ok'
                    : 'border-danger-soft bg-danger-soft text-danger',
                )}
              >
                {entry.result === 'ok'
                  ? t('activity.resultOk')
                  : t('activity.resultFailed')}
              </span>
            </div>
          ))}
        </div>
      )}
    </Dialog>
  )
}
