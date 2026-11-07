import { useMutation, useQueryClient } from '@tanstack/react-query'
import { FolderOpen, FolderPlus, Plus, RefreshCw } from 'lucide-react'
import { useState } from 'react'
import { Button } from '@/components/ui/button'
import { EmptyState } from '@/components/ui/empty-state'
import { ErrorPanel } from '@/components/ui/error-panel'
import { LoadingBlock } from '@/components/ui/spinner'
import { toast } from '@/components/ui/toast-store'
import { useI18n } from '@/i18n'
import { api } from '@/lib/api'
import { toErrorMessage } from '@/lib/errors'
import { invalidateLibrary, invalidateTargets } from '@/lib/invalidate'
import { useSkills, useTargets } from '@/lib/queries'
import type { Target } from '@/lib/types'
import { useReconcile } from '@/features/sync/useReconcile'
import { TargetDeleteDialog } from './TargetDeleteDialog'
import { TargetFormDialog } from './TargetFormDialog'
import { TargetTable } from './TargetTable'

export function TargetsPage() {
  const { t } = useI18n()
  const queryClient = useQueryClient()
  const skillsQuery = useSkills()
  const targetsQuery = useTargets()
  const reconcile = useReconcile()

  const skills = skillsQuery.data ?? []
  const targets = targetsQuery.data ?? []

  const [createOpen, setCreateOpen] = useState(false)
  const [editing, setEditing] = useState<Target | null>(null)
  const [deleting, setDeleting] = useState<Target | null>(null)

  const toggle = useMutation({
    mutationFn: ({
      target,
      enabled,
    }: {
      target: Target
      enabled: boolean
    }) => api.updateTarget(target.id, { enabled }),
    onSuccess: (_saved, variables) => {
      invalidateTargets(queryClient)
      invalidateLibrary(queryClient)
      toast(
        variables.enabled
          ? t('targets.enabledToast', { name: variables.target.name })
          : t('targets.disabledToast', { name: variables.target.name }),
        { tone: variables.enabled ? 'ok' : 'warn' },
      )
    },
    onError: (error) => {
      toast(t('toast.operationFailed', { message: toErrorMessage(error) }), {
        tone: 'err',
      })
    },
  })

  const openIcons = useMutation({
    mutationFn: () => api.revealIconsDir(),
    onError: (error) => {
      toast(t('toast.operationFailed', { message: toErrorMessage(error) }), {
        tone: 'err',
      })
    },
  })

  const openDir = useMutation({
    mutationFn: (target: Target) => api.revealTarget(target.id),
    onError: (error) => {
      toast(t('toast.operationFailed', { message: toErrorMessage(error) }), {
        tone: 'err',
      })
    },
  })

  const pendingId = toggle.isPending
    ? (toggle.variables?.target.id ?? null)
    : null

  const loading = targetsQuery.isLoading || skillsQuery.isLoading
  const error = targetsQuery.error ?? skillsQuery.error

  return (
    <div className="flex h-full min-h-0 flex-col">
      <header className="shrink-0 px-6 pt-5 pb-3">
        <div className="flex items-start gap-3">
          <div className="min-w-0">
            <h1 className="text-[21px] leading-none font-semibold tracking-tight text-ink">
              {t('targets.title')}
            </h1>
          </div>
          <div className="ml-auto flex shrink-0 items-center gap-2">
            <Button
              variant="secondary"
              size="sm"
              onClick={() => reconcile.mutate()}
              disabled={reconcile.isPending}
            >
              <RefreshCw
                size={13}
                aria-hidden
                className={reconcile.isPending ? 'animate-spin' : undefined}
              />
              {t('targets.reconcile')}
            </Button>
            <Button
              variant="secondary"
              size="icon"
              onClick={() => openIcons.mutate()}
              disabled={openIcons.isPending}
              aria-label={t('targets.openIcons')}
              title={t('targets.openIcons')}
            >
              <FolderOpen size={14} aria-hidden />
            </Button>
            <Button
              variant="primary"
              size="sm"
              onClick={() => setCreateOpen(true)}
            >
              <Plus size={13} aria-hidden />
              {t('targets.add')}
            </Button>
          </div>
        </div>

      </header>

      {loading ? (
        <LoadingBlock label={t('common.loading')} />
      ) : error ? (
        <ErrorPanel
          title={t('error.targets')}
          message={toErrorMessage(error)}
          retryLabel={t('common.retry')}
          onRetry={() => {
            void targetsQuery.refetch()
            void skillsQuery.refetch()
          }}
        />
      ) : targets.length === 0 ? (
        <EmptyState
          icon={<FolderPlus size={17} aria-hidden />}
          title={t('targets.emptyTitle')}
          body={t('targets.emptyBody')}
          actions={
            <Button
              variant="primary"
              size="sm"
              onClick={() => setCreateOpen(true)}
            >
              {t('targets.add')}
            </Button>
          }
        />
      ) : (
        <TargetTable
          targets={targets}
          skills={skills}
          pendingId={pendingId}
          onToggle={(target, value) => toggle.mutate({ target, enabled: value })}
          onOpenDir={(target) => openDir.mutate(target)}
          onEdit={setEditing}
          onDelete={setDeleting}
        />
      )}

      {createOpen ? (
        <TargetFormDialog
          target={null}
          targets={targets}
          onOpenChange={(open) => {
            if (!open) setCreateOpen(false)
          }}
        />
      ) : null}

      {editing ? (
        <TargetFormDialog
          key={editing.id}
          target={editing}
          targets={targets}
          onOpenChange={(open) => {
            if (!open) setEditing(null)
          }}
        />
      ) : null}

      {deleting ? (
        <TargetDeleteDialog
          target={deleting}
          onOpenChange={(open) => {
            if (!open) setDeleting(null)
          }}
        />
      ) : null}
    </div>
  )
}
