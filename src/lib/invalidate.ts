import type { QueryClient } from '@tanstack/react-query'
import { queryKeys } from './queryKeys'

/** Skills, embedded installs and the activity feed all move together. */
export function invalidateLibrary(queryClient: QueryClient): void {
  void queryClient.invalidateQueries({ queryKey: queryKeys.skills })
  void queryClient.invalidateQueries({ queryKey: ['skill'] })
  void queryClient.invalidateQueries({ queryKey: queryKeys.activity })
}

export function invalidateTargets(queryClient: QueryClient): void {
  void queryClient.invalidateQueries({ queryKey: queryKeys.targets })
  void queryClient.invalidateQueries({ queryKey: queryKeys.targetSuggestions })
  invalidateLibrary(queryClient)
}

export function invalidateSettings(queryClient: QueryClient): void {
  void queryClient.invalidateQueries({ queryKey: queryKeys.settings })
}
