import { useMutation, useQueryClient } from '@tanstack/react-query'
import { useState } from 'react'
import { Button } from '@/components/ui/button'
import { Checkbox } from '@/components/ui/checkbox'
import { Dialog } from '@/components/ui/dialog'
import { toast } from '@/components/ui/toast-store'
import { useI18n } from '@/i18n'
import { api } from '@/lib/api'
import { toErrorMessage } from '@/lib/errors'
import { invalidateLibrary, invalidateTargets } from '@/lib/invalidate'
import type { Target } from '@/lib/types'

interface TargetDeleteDialogProps {
  target: Target
  onOpenChange: (open: boolean) => void
}

export function TargetDeleteDialog({
  target,
  onOpenChange,
}: TargetDeleteDialogProps) {
  const { t } = useI18n()
  const queryClient = useQueryClient()
  const [unlink, setUnlink] = useState(true)

  const remove = useMutation({
    mutationFn: () => api.deleteTarget(target.id, { unlink }),
    onSuccess: () => {
      invalidateTargets(queryClient)
      invalidateLibrary(queryClient)
      toast(t('toast.targetDeleted', { name: target.name }), { tone: 'warn' })
      onOpenChange(false)
    },
    onError: (error) => {
      toast(t('toast.operationFailed', { message: toErrorMessage(error) }), {
        tone: 'err',
      })
    },
  })

  return (
    <Dialog
      open
      onOpenChange={(open) => {
        if (!open && !remove.isPending) onOpenChange(false)
      }}
      title={t('targets.deleteTitle')}
      description={t('targets.deleteBody', { name: target.name })}
      footer={
        <>
          <Button
            variant="secondary"
            onClick={() => onOpenChange(false)}
            disabled={remove.isPending}
          >
            {t('common.cancel')}
          </Button>
          <Button
            variant="danger"
            onClick={() => remove.mutate()}
            disabled={remove.isPending}
          >
            {remove.isPending
              ? t('common.loading')
              : t('targets.deleteConfirm')}
          </Button>
        </>
      }
    >
      <div className="space-y-3">
        <div className="rounded-md border border-line bg-panel-2 px-2.5 py-2 font-mono text-[11px] leading-relaxed break-all text-muted">
          {target.skillsDir}
        </div>
        <Checkbox
          checked={unlink}
          onChange={(event) => setUnlink(event.target.checked)}
          label={t('targets.deleteUnlink')}
        />
        {remove.isError ? (
          <p role="alert" className="text-xs text-danger">
            {toErrorMessage(remove.error)}
          </p>
        ) : null}
      </div>
    </Dialog>
  )
}
