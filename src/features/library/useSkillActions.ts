import { useMutation, useQueryClient } from '@tanstack/react-query'
import { toast } from '@/components/ui/toast-store'
import { useI18n } from '@/i18n'
import { api } from '@/lib/api'
import { toErrorMessage } from '@/lib/errors'
import { invalidateLibrary } from '@/lib/invalidate'
import { queryKeys } from '@/lib/queryKeys'
import type { ConflictResolution, SkillSummary, Target } from '@/lib/types'

export function useSkillActions(
  skill: SkillSummary,
  targets: readonly Target[],
) {
  const { t } = useI18n()
  const queryClient = useQueryClient()

  const targetName = (targetId: string) =>
    targets.find((target) => target.id === targetId)?.name ?? targetId

  const refresh = () => {
    invalidateLibrary(queryClient)
    void queryClient.invalidateQueries({ queryKey: queryKeys.skill(skill.id) })
  }

  const fail = (error: unknown) => {
    toast(t('toast.operationFailed', { message: toErrorMessage(error) }), {
      tone: 'err',
    })
  }

  const toggleInstall = useMutation({
    mutationFn: ({
      targetId,
      enabled,
    }: {
      targetId: string
      enabled: boolean
    }) => api.setInstall(skill.id, targetId, { enabled }),
    onSuccess: (_install, variables) => {
      refresh()
      const target = targetName(variables.targetId)
      toast(
        variables.enabled
          ? t('toast.installOn', { slug: skill.slug, target })
          : t('toast.installOff', { slug: skill.slug, target }),
        { tone: variables.enabled ? 'ok' : 'warn' },
      )
    },
    onError: fail,
  })

  const resolve = useMutation({
    mutationFn: ({
      targetId,
      resolution,
    }: {
      targetId: string
      resolution: ConflictResolution
    }) => api.resolveInstall(skill.id, targetId, resolution),
    onSuccess: (_install, variables) => {
      refresh()
      toast(
        t('toast.conflictUnignored', {
          target: targetName(variables.targetId),
        }),
        { tone: 'warn' },
      )
    },
    onError: fail,
  })

  const reveal = useMutation({
    mutationFn: () => api.revealSkill(skill.id),
    onSuccess: () => toast(t('toast.revealed')),
    onError: fail,
  })

  const pendingTargetId = toggleInstall.isPending
    ? (toggleInstall.variables?.targetId ?? null)
    : null

  return { toggleInstall, resolve, reveal, pendingTargetId }
}
