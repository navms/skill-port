import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { AlertTriangle, RefreshCw } from 'lucide-react'
import { useEffect, useState } from 'react'
import { Button } from '@/components/ui/button'
import { Dialog } from '@/components/ui/dialog'
import { Progress } from '@/components/ui/progress'
import { Spinner } from '@/components/ui/spinner'
import { toast } from '@/components/ui/toast-store'
import { useI18n } from '@/i18n'
import { api } from '@/lib/api'
import { toErrorMessage } from '@/lib/errors'
import { invalidateLibrary, invalidateTargets } from '@/lib/invalidate'
import {
  hasActionable,
  planIsExpired,
  summarizePlan,
} from '@/lib/plan'
import { queryKeys } from '@/lib/queryKeys'
import type { SyncFilter, SyncProgress, SyncResult } from '@/lib/types'
import { PlanGroups, PlanSummaryChips } from './PlanPreview'
import { SyncResultView } from './SyncResultView'

interface SyncPlanDialogProps {
  filter?: SyncFilter
  skillName?: string
  onOpenChange: (open: boolean) => void
}

export function SyncPlanDialog({
  filter,
  skillName,
  onOpenChange,
}: SyncPlanDialogProps) {
  const { t } = useI18n()
  const queryClient = useQueryClient()
  const [result, setResult] = useState<SyncResult | null>(null)
  const [applying, setApplying] = useState(false)
  const [progress, setProgress] = useState<SyncProgress | null>(null)

  const planQuery = useQuery({
    queryKey: queryKeys.syncPlan(filter),
    queryFn: () => api.planSync(filter),
    staleTime: 0,
    gcTime: 0,
    refetchOnMount: 'always',
    retry: false,
  })

  const plan = planQuery.data

  useEffect(() => {
    return api.subscribeSyncProgress((update) => {
      setProgress(update)
    })
  }, [])

  const apply = useMutation({
    mutationFn: (planId: string) => api.applySync(planId),
    onSuccess: (outcome) => {
      setResult(outcome)
      setApplying(false)
      invalidateLibrary(queryClient)
      invalidateTargets(queryClient)

      if (outcome.cancelled) {
        toast(t('toast.syncCancelled'), { tone: 'warn' })
        return
      }
      if (outcome.failed > 0) {
        toast(
          t('toast.syncedPartial', {
            applied: outcome.applied,
            failed: outcome.failed,
          }),
          { tone: 'warn' },
        )
        return
      }
      if (outcome.applied === 0) {
        toast(t('sync.doneNothing'))
        return
      }
      const undoId = outcome.undoId
      toast(t('toast.synced', { count: outcome.applied }), {
        tone: 'ok',
        action: undoId
          ? {
              label: t('sync.undo'),
              onClick: () => {
                void api
                  .undoActivity(undoId)
                  .then((res) => {
                    if (res.ok) {
                      invalidateLibrary(queryClient)
                      toast(t('toast.undone'))
                    } else {
                      toast(
                        t('toast.undoFailed', {
                          message: res.message ?? '',
                        }),
                        { tone: 'err' },
                      )
                    }
                  })
                  .catch((error: unknown) => {
                    toast(
                      t('toast.undoFailed', {
                        message: toErrorMessage(error),
                      }),
                      { tone: 'err' },
                    )
                  })
              },
            }
          : undefined,
      })
    },
    onError: (error) => {
      setApplying(false)
      toast(t('toast.operationFailed', { message: toErrorMessage(error) }), {
        tone: 'err',
      })
    },
  })

  const cancel = useMutation({
    mutationFn: (planId: string) => api.cancelSync(planId),
    onError: (error) => {
      toast(t('toast.operationFailed', { message: toErrorMessage(error) }), {
        tone: 'err',
      })
    },
  })

  const startApply = () => {
    if (!plan) return
    setApplying(true)
    setResult(null)
    setProgress({
      planId: plan.id,
      completed: 0,
      total: summarizePlan(plan.actions).actionable,
      actionId: null,
      status: 'running',
    })
    apply.mutate(plan.id)
  }

  const expired = plan ? planIsExpired(plan) : false
  const summary = plan ? summarizePlan(plan.actions) : null
  const actionable = plan ? hasActionable(plan) : false

  const liveProgress =
    progress && plan && progress.planId === plan.id ? progress : null
  const completed = liveProgress?.completed ?? 0
  const total = liveProgress?.total ?? summary?.actionable ?? 0
  let footer
  if (result) {
    footer = (
      <Button variant="secondary" onClick={() => onOpenChange(false)}>
        {t('common.done')}
      </Button>
    )
  } else if (applying) {
    footer = (
      <>
        <span className="mr-auto font-mono text-[11px] text-muted">
          {t('sync.progress', { done: completed, total })}
        </span>
        <Button
          variant="secondary"
          onClick={() => {
            if (plan) cancel.mutate(plan.id)
          }}
          disabled={cancel.isPending}
        >
          {cancel.isPending ? t('sync.cancelling') : t('sync.cancel')}
        </Button>
      </>
    )
  } else if (plan && expired) {
    footer = (
      <>
        <Button variant="secondary" onClick={() => onOpenChange(false)}>
          {t('common.close')}
        </Button>
        <Button
          variant="primary"
          onClick={() => void planQuery.refetch()}
          disabled={planQuery.isFetching}
        >
          <RefreshCw size={13} aria-hidden />
          {t('sync.retry')}
        </Button>
      </>
    )
  } else {
    footer = (
      <>
        <Button variant="secondary" onClick={() => onOpenChange(false)}>
          {t('common.close')}
        </Button>
        <Button
          variant="primary"
          onClick={startApply}
          disabled={!plan || !actionable || planQuery.isFetching}
        >
          <RefreshCw size={13} aria-hidden />
          {t('sync.apply', { count: summary?.actionable ?? 0 })}
        </Button>
      </>
    )
  }

  return (
    <Dialog
      open
      onOpenChange={(open) => {
        if (!open && !applying) onOpenChange(false)
      }}
      title={
        skillName
          ? t('sync.titleSkill', { slug: skillName })
          : t('sync.titleAll')
      }
      description={t('sync.planTtl')}
      className="w-[min(640px,94vw)]"
      footer={footer}
    >
      {planQuery.isLoading ? (
        <div className="flex items-center justify-center gap-2 py-12 text-[13px] text-muted">
          <Spinner />
          {t('sync.planning')}
        </div>
      ) : planQuery.isError ? (
        <div className="py-8 text-center">
          <p className="text-sm font-medium text-ink-2">
            {t('sync.planError')}
          </p>
          <p className="mt-1 text-xs text-muted">
            {toErrorMessage(planQuery.error)}
          </p>
          <Button
            variant="secondary"
            size="sm"
            className="mt-3"
            onClick={() => void planQuery.refetch()}
          >
            {t('common.retry')}
          </Button>
        </div>
      ) : result ? (
        <SyncResultView result={result} />
      ) : !plan ? null : expired ? (
        <div className="flex items-start gap-2 rounded-md border border-warn-soft bg-warn-soft px-3 py-2 text-xs leading-relaxed text-warn">
          <AlertTriangle size={14} aria-hidden className="mt-0.5 shrink-0" />
          {t('sync.expired')}
        </div>
      ) : summary && summary.total === 0 ? (
        <div className="py-8 text-center">
          <p className="text-sm font-medium text-ink-2">{t('sync.empty')}</p>
          <p className="mt-1 text-xs text-muted">{t('sync.emptyBody')}</p>
        </div>
      ) : (
        <div className="space-y-3">
          {applying ? (
            <div>
              <Progress
                value={completed}
                max={total}
                label={t('sync.applying')}
              />
            </div>
          ) : null}

          <div className="font-mono text-[11px] text-faint">
            {t('sync.summary', {
              total: summary?.total ?? 0,
              actionable: summary?.actionable ?? 0,
            })}
          </div>

          {summary ? <PlanSummaryChips summary={summary} /> : null}

          {plan ? <PlanGroups actions={plan.actions} /> : null}

          {!actionable ? (
            <p className="text-xs leading-relaxed text-muted">
              {t('sync.noActionableBody')}
            </p>
          ) : summary && summary.conflict > 0 ? (
            <p className="text-xs leading-relaxed text-muted">
              {t('sync.conflictHint')}
            </p>
          ) : null}
        </div>
      )}
    </Dialog>
  )
}
