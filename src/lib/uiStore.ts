import { create } from 'zustand'
import { persist } from 'zustand/middleware'
import type { FilterChip } from './filter'
import type { Language, ThemeMode } from './types'

export type DialogState =
  | { kind: 'none' }
  | { kind: 'activity' }
  | { kind: 'import' }
  | { kind: 'sync'; skillId?: string; skillName?: string }
  | { kind: 'target-create' }

interface UiState {
  themeMode: ThemeMode
  language: Language

  query: string
  chip: FilterChip
  category: string | null
  targetFilter: string | null

  paletteOpen: boolean
  dialog: DialogState
  detailDrawerOpen: boolean
  sidebarOpen: boolean
  lastCheckedAt: string | null

  setThemeMode: (mode: ThemeMode) => void
  toggleTheme: (resolved: 'light' | 'dark') => void
  setLanguage: (language: Language) => void

  setQuery: (query: string) => void
  setChip: (chip: FilterChip) => void
  selectCategory: (category: string | null) => void
  setTargetFilter: (targetId: string | null) => void
  clearFilters: () => void

  openPalette: () => void
  closePalette: () => void
  openDialog: (dialog: Exclude<DialogState, { kind: 'none' }>) => void
  closeDialog: () => void
  setDetailDrawerOpen: (open: boolean) => void
  setSidebarOpen: (open: boolean) => void
  toggleSidebar: () => void
  setLastCheckedAt: (iso: string | null) => void
}

function persistTheme(mode: ThemeMode): void {
  try {
    window.localStorage.setItem('skillport-theme', mode)
  } catch {
    // Private mode or a blocked storage backend: the in-memory value still works.
  }
}

export const useUiStore = create<UiState>()(
  persist(
    (set) => ({
      themeMode: 'dark',
      language: 'zh-CN',

      query: '',
      chip: 'all',
      category: null,
      targetFilter: null,

      paletteOpen: false,
      dialog: { kind: 'none' },
      detailDrawerOpen: false,
      sidebarOpen: true,
      lastCheckedAt: null,

      setThemeMode: (themeMode) => {
        persistTheme(themeMode)
        set({ themeMode })
      },
      toggleTheme: (resolved) => {
        const themeMode: ThemeMode = resolved === 'dark' ? 'light' : 'dark'
        persistTheme(themeMode)
        set({ themeMode })
      },
      setLanguage: (language) => set({ language }),

      setQuery: (query) => set({ query }),
      setChip: (chip) =>
        set({ chip, category: null, targetFilter: null }),
      selectCategory: (category) =>
        set({ category, chip: 'all', targetFilter: null }),
      setTargetFilter: (targetFilter) =>
        set({ targetFilter, category: null, chip: 'all' }),
      clearFilters: () =>
        set({ query: '', chip: 'all', category: null, targetFilter: null }),

      openPalette: () => set({ paletteOpen: true }),
      closePalette: () => set({ paletteOpen: false }),
      openDialog: (dialog) => set({ dialog }),
      closeDialog: () => set({ dialog: { kind: 'none' } }),
      setDetailDrawerOpen: (detailDrawerOpen) => set({ detailDrawerOpen }),
      setSidebarOpen: (sidebarOpen) => set({ sidebarOpen }),
      toggleSidebar: () => set((state) => ({ sidebarOpen: !state.sidebarOpen })),
      setLastCheckedAt: (lastCheckedAt) => set({ lastCheckedAt }),
    }),
    {
      name: 'skillport-ui',
      partialize: (state) => ({
        themeMode: state.themeMode,
        language: state.language,
        sidebarOpen: state.sidebarOpen,
      }),
    },
  ),
)
