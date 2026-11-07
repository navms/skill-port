import { useMutation, useQueryClient } from '@tanstack/react-query'
import { Button } from '@/components/ui/button'
import { Dialog } from '@/components/ui/dialog'
import { toast } from '@/components/ui/toast-store'
import { useI18n } from '@/i18n'
import { api } from '@/lib/api'
import { toErrorMessage } from '@/lib/errors'
import { invalidateLibrary } from '@/lib/invalidate'
import type {
  ConflictResolution,
  Install,
  SkillSummary,
  Target,
} from '@/lib/types'

interface ConflictDialogProps {
  skill: SkillSummary
  target: Target
  install: Install
  onOpenChange: (open: boolean) => void
}

export function ConflictDialog({
  skill,
  target,
  install,
  onOpenChange,
}: ConflictDialogProps) {
  const { t } = useI18n()
  const queryClient = useQueryClient()

  const resolve = useMutation({
    mutationFn: (resolution: ConflictResolution) =>
      api.resolveInstall(skill.id, target.id, resolution),
    onSuccess: (_data, resolution) => {
      invalidateLibrary(queryClient)
      const messages: Record<ConflictResolution, string> = {
        adopt: t('toast.conflictAdopted', { target: target.name }),
        ignore: t('toast.conflictIgnored', { target: target.name }),
        unignore: t('toast.conflictUnignored', { target: target.name }),
        cancel: t('toast.installCancelled', { target: target.name }),
      }
      toast(messages[resolution], {
        tone: resolution === 'adopt' ? 'ok' : 'warn',
      })
      onOpenChange(false)
    },
    onError: (error) => {
      toast(t('toast.operationFailed', { message: toErrorMessage(error) }), {
        tone: 'err',
      })
    },
  })

  const pending = resolve.isPending

  return (
    <Dialog
      open
      onOpenChange={(open) => {
        if (!open && !pending) onOpenChange(false)
      }}
      title={t('detail.conflictDialogTitle', { target: target.name })}
      description={t('detail.conflictDialogBody')}
      footer={
        <>
          <Button
            variant="secondary"
            autoFocus
            onClick={() => onOpenChange(false)}
            disabled={pending}
          >
            {t('detail.keepCurrent')}
          </Button>
          {install.ignored ? (
            <Button
              variant="secondary"
              onClick={() => resolve.mutate('unignore')}
              disabled={pending}
            >
              {t('detail.unignore')}
            </Button>
          ) : (
            <Button
              variant="secondary"
              onClick={() => resolve.mutate('ignore')}
              disabled={pending}
            >
              {t('detail.ignore')}
            </Button>
          )}
          <Button
            variant="danger"
            onClick={() => resolve.mutate('cancel')}
            disabled={pending}
          >
            {t('detail.cancelInstall')}
          </Button>
          {!install.ignored ? (
            <Button
              variant="primary"
              onClick={() => resolve.mutate('adopt')}
              disabled={pending}
            >
              {t('detail.adopt')}
            </Button>
          ) : null}
        </>
      }
    >
      <div>
        <p className="mb-1.5 text-xs text-muted">
          {t('detail.conflictPath')}
        </p>
        <div className="rounded-md border border-line bg-panel-2 px-2.5 py-2 font-mono text-[11px] leading-relaxed break-all text-muted">
          {install.targetPath}
        </div>
      </div>
    </Dialog>
  )
}
