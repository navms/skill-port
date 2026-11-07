import { useMutation, useQueryClient } from '@tanstack/react-query'
import { useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { Button } from '@/components/ui/button'
import { Checkbox } from '@/components/ui/checkbox'
import { Dialog } from '@/components/ui/dialog'
import { toast } from '@/components/ui/toast-store'
import { useI18n } from '@/i18n'
import { api } from '@/lib/api'
import { toErrorMessage } from '@/lib/errors'
import { invalidateLibrary } from '@/lib/invalidate'
import type { SkillSummary } from '@/lib/types'

interface DeleteSkillDialogProps {
  skill: SkillSummary
  onOpenChange: (open: boolean) => void
}

export function DeleteSkillDialog({
  skill,
  onOpenChange,
}: DeleteSkillDialogProps) {
  const { t } = useI18n()
  const queryClient = useQueryClient()
  const navigate = useNavigate()
  const [deleteTargets, setDeleteTargets] = useState(true)

  const remove = useMutation({
    mutationFn: () => api.deleteSkill(skill.id, { deleteTargets }),
    onSuccess: (result) => {
      invalidateLibrary(queryClient)
      navigate('/')
      onOpenChange(false)
      toast(t('delete.done', { slug: skill.slug }), {
        tone: 'ok',
        action: result.undoId
          ? {
              label: t('common.undo'),
              onClick: () => {
                const undoId = result.undoId
                if (!undoId) return
                void api
                  .undoActivity(undoId)
                  .then((outcome) => {
                    if (outcome.ok) {
                      invalidateLibrary(queryClient)
                      toast(t('toast.undone'))
                    } else {
                      toast(
                        t('toast.undoFailed', {
                          message: outcome.message ?? '',
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
      title={t('delete.title')}
      description={t('delete.body', { slug: skill.slug })}
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
            {remove.isPending ? t('common.loading') : t('delete.confirm')}
          </Button>
        </>
      }
    >
      <div className="space-y-3">
        <div className="rounded-md border border-line bg-panel-2 px-2.5 py-2 font-mono text-[11px] break-all text-muted">
          {skill.slug}
        </div>
        <Checkbox
          checked={deleteTargets}
          onChange={(event) => setDeleteTargets(event.target.checked)}
          label={t('delete.cleanTargets')}
        />
        <p className="text-xs leading-relaxed text-faint">
          {t('delete.detail')}
        </p>
        {remove.isError ? (
          <p role="alert" className="text-xs text-danger">
            {toErrorMessage(remove.error)}
          </p>
        ) : null}
      </div>
    </Dialog>
  )
}
