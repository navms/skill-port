import { useMutation, useQueryClient } from '@tanstack/react-query'
import { toast } from '@/components/ui/toast-store'
import { useI18n } from '@/i18n'
import { api } from '@/lib/api'
import { toErrorMessage } from '@/lib/errors'
import { invalidateLibrary } from '@/lib/invalidate'
import { queryKeys } from '@/lib/queryKeys'
import { useUiStore } from '@/lib/uiStore'

export function useReconcile() {
  const queryClient = useQueryClient()
  const setLastCheckedAt = useUiStore((state) => state.setLastCheckedAt)
  const { t } = useI18n()

  return useMutation({
    mutationFn: () => api.reconcile(),
    onSuccess: (report) => {
      setLastCheckedAt(report.checkedAt)
      invalidateLibrary(queryClient)
      void queryClient.invalidateQueries({ queryKey: queryKeys.targets })
      toast(
        t('toast.reconciled', {
          checked: report.checked,
          issues: report.issues,
        }),
        { tone: report.issues > 0 ? 'warn' : 'ok' },
      )
    },
    onError: (error) => {
      toast(
        t('toast.reconcileFailed', { message: toErrorMessage(error) }),
        { tone: 'err' },
      )
    },
  })
}
