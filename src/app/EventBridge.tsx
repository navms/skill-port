import { useQueryClient } from '@tanstack/react-query'
import { useEffect } from 'react'
import { api } from '@/lib/api'
import { invalidateLibrary } from '@/lib/invalidate'
import { queryKeys } from '@/lib/queryKeys'
import { useUiStore } from '@/lib/uiStore'

/** Wires Tauri/mock events into query invalidation. */
export function EventBridge(): null {
  const queryClient = useQueryClient()
  const setLastCheckedAt = useUiStore((state) => state.setLastCheckedAt)

  useEffect(() => {
    const offLibrary = api.subscribeLibraryChanged(() => {
      invalidateLibrary(queryClient)
    })
    const offReconcile = api.subscribeReconcileFinished((report) => {
      setLastCheckedAt(report.checkedAt)
      invalidateLibrary(queryClient)
      void queryClient.invalidateQueries({ queryKey: queryKeys.targets })
    })
    return () => {
      offLibrary()
      offReconcile()
    }
  }, [queryClient, setLastCheckedAt])

  return null
}
