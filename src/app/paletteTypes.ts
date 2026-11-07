import type { ReactNode } from 'react'
import type { SkillSummary } from '@/lib/types'

export interface ActionItem {
  kind: 'action'
  id: string
  label: string
  sub: string
  icon: ReactNode
  run: () => void
}

export interface SkillItem {
  kind: 'skill'
  id: string
  skill: SkillSummary
}

export type PaletteItem = ActionItem | SkillItem
