import {
  Activity,
  FolderPlus,
  Import,
  Languages,
  ListFilter,
  Moon,
  RefreshCw,
  Sun,
  Target,
} from 'lucide-react'
import type { I18n } from '@/i18n'
import type { FilterChip } from '@/lib/filter'
import type { Language, ThemeMode } from '@/lib/types'
import type { DialogState } from '@/lib/uiStore'
import type { ActionItem } from './paletteTypes'

export interface PaletteActionContext {
  t: I18n['t']
  resolved: 'light' | 'dark'
  openDialog: (dialog: Exclude<DialogState, { kind: 'none' }>) => void
  navigate: (path: string) => void
  setChip: (chip: FilterChip) => void
  setThemeMode: (mode: ThemeMode) => void
  language: Language
  setLanguage: (language: Language) => void
  reconcile: () => void
}

export function buildPaletteActions({
  t,
  resolved,
  openDialog,
  navigate,
  setChip,
  setThemeMode,
  language,
  setLanguage,
  reconcile,
}: PaletteActionContext): ActionItem[] {
  return [
    {
      kind: 'action',
      id: 'import',
      label: t('palette.actionImport'),
      sub: t('palette.actionImportSub'),
      icon: <Import size={14} aria-hidden />,
      run: () => openDialog({ kind: 'import' }),
    },
    {
      kind: 'action',
      id: 'sync-all',
      label: t('palette.actionSyncAll'),
      sub: t('palette.actionSyncAllSub'),
      icon: <RefreshCw size={14} aria-hidden />,
      run: () => openDialog({ kind: 'sync' }),
    },
    {
      kind: 'action',
      id: 'reconcile',
      label: t('palette.actionReconcile'),
      sub: t('palette.actionReconcileSub'),
      icon: <RefreshCw size={14} aria-hidden />,
      run: () => reconcile(),
    },
    {
      kind: 'action',
      id: 'issues',
      label: t('palette.actionIssues'),
      sub: t('palette.actionIssuesSub'),
      icon: <ListFilter size={14} aria-hidden />,
      run: () => {
        setChip('issues')
        navigate('/')
      },
    },
    {
      kind: 'action',
      id: 'targets',
      label: t('palette.actionTargets'),
      sub: t('palette.actionTargetsSub'),
      icon: <Target size={14} aria-hidden />,
      run: () => navigate('/targets'),
    },
    {
      kind: 'action',
      id: 'add-target',
      label: t('palette.actionAddTarget'),
      sub: t('palette.actionAddTargetSub'),
      icon: <FolderPlus size={14} aria-hidden />,
      run: () => openDialog({ kind: 'target-create' }),
    },
    {
      kind: 'action',
      id: 'activity',
      label: t('palette.actionActivity'),
      sub: t('palette.actionActivitySub'),
      icon: <Activity size={14} aria-hidden />,
      run: () => openDialog({ kind: 'activity' }),
    },
    {
      kind: 'action',
      id: 'language',
      label: t('palette.actionLanguage'),
      sub: t('palette.actionLanguageSub', {
        language: language === 'zh-CN' ? 'English' : '中文',
      }),
      icon: <Languages size={14} aria-hidden />,
      run: () => setLanguage(language === 'zh-CN' ? 'en' : 'zh-CN'),
    },
    {
      kind: 'action',
      id: 'theme',
      label: t('palette.actionTheme'),
      sub: t('palette.actionThemeSub', {
        theme:
          resolved === 'dark'
            ? t('common.themeDark')
            : t('common.themeLight'),
      }),
      icon:
        resolved === 'dark' ? (
          <Sun size={14} aria-hidden />
        ) : (
          <Moon size={14} aria-hidden />
        ),
      run: () => setThemeMode(resolved === 'dark' ? 'light' : 'dark'),
    },
  ]
}
