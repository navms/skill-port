import type { SyncFilter } from './types'

export const queryKeys = {
  skills: ['skills'] as const,
  skill: (id: string) => ['skill', id] as const,
  targets: ['targets'] as const,
  targetSuggestions: ['target-suggestions'] as const,
  targetIcons: ['target-icons'] as const,
  settings: ['settings'] as const,
  activity: ['activity'] as const,
  syncPlan: (filter?: SyncFilter) =>
    [
      'sync-plan',
      filter?.skillId ?? null,
      filter?.targetId ?? null,
    ] as const,
}
