import { ActivityDialog } from '@/features/activity/ActivityDialog'
import { ImportDialog } from '@/features/library/ImportDialog'
import { SyncPlanDialog } from '@/features/sync/SyncPlanDialog'
import { TargetFormDialog } from '@/features/targets/TargetFormDialog'
import { useTargets } from '@/lib/queries'
import { useUiStore } from '@/lib/uiStore'

export function DialogHost() {
  const dialog = useUiStore((state) => state.dialog)
  const closeDialog = useUiStore((state) => state.closeDialog)
  const targets = useTargets().data ?? []

  switch (dialog.kind) {
    case 'activity':
      return <ActivityDialog onOpenChange={closeDialog} />
    case 'import':
      return <ImportDialog onOpenChange={closeDialog} />
    case 'target-create':
      return (
        <TargetFormDialog
          target={null}
          targets={targets}
          onOpenChange={closeDialog}
        />
      )
    case 'sync':
      return (
        <SyncPlanDialog
          filter={dialog.skillId ? { skillId: dialog.skillId } : undefined}
          skillName={dialog.skillName}
          onOpenChange={closeDialog}
        />
      )
    case 'none':
      return null
  }
}
