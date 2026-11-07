import { useQuery } from '@tanstack/react-query'
import { api } from './api'
import { queryKeys } from './queryKeys'

export function useSkills() {
  return useQuery({
    queryKey: queryKeys.skills,
    queryFn: () => api.listSkills(),
  })
}

export function useTargets() {
  return useQuery({
    queryKey: queryKeys.targets,
    queryFn: () => api.listTargets(),
  })
}

export function useSettings() {
  return useQuery({
    queryKey: queryKeys.settings,
    queryFn: () => api.getSettings(),
    staleTime: 60_000,
  })
}

export function useSkillDetail(id: string | null) {
  return useQuery({
    queryKey: queryKeys.skill(id ?? ''),
    queryFn: () => api.getSkill(id ?? ''),
    enabled: Boolean(id),
    staleTime: 10_000,
  })
}

/**
 * Icons live in the data directory so they can be added without repackaging.
 * `refetchOnWindowFocus: 'always'` matters: plain `true` only refetches when the
 * query is stale, so a file dropped in during the 30s window would not show up
 * when the user comes back from Finder.
 */
export const targetIconsQueryOptions = {
  staleTime: 30_000,
  refetchOnWindowFocus: 'always',
} as const

export function useTargetIcons() {
  return useQuery({
    queryKey: queryKeys.targetIcons,
    queryFn: () => api.listTargetIcons(),
    ...targetIconsQueryOptions,
  })
}

export function useTargetSuggestions(enabled: boolean) {
  return useQuery({
    queryKey: queryKeys.targetSuggestions,
    queryFn: () => api.detectTargets(),
    enabled,
    staleTime: 60_000,
  })
}
