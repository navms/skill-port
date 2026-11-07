import {
  Activity,
  Languages,
  Moon,
  PanelLeftClose,
  PanelLeftOpen,
  Sun,
} from 'lucide-react'
import { NavLink } from 'react-router-dom'
import { Button } from '@/components/ui/button'
import { Kbd } from '@/components/ui/kbd'
import { useI18n } from '@/i18n'
import { formatRelative } from '@/lib/format'
import { commandShortcut, hasOverlayTitleBar } from '@/lib/platform'
import { useUiStore } from '@/lib/uiStore'
import { cn } from '@/lib/utils'
import { useResolvedTheme } from './theme'

const NARROW_NAV =
  'rounded-md px-2.5 py-1.5 text-xs whitespace-nowrap transition-colors text-muted hover:text-ink'
const NARROW_NAV_ACTIVE = 'bg-raise text-ink'

const ICON_NAV =
  'flex size-8 items-center justify-center rounded-md text-muted transition-colors hover:bg-raise hover:text-ink'

export function TitleBar() {
  const { t, language } = useI18n()
  const resolved = useResolvedTheme()
  const lastCheckedAt = useUiStore((state) => state.lastCheckedAt)
  const setThemeMode = useUiStore((state) => state.setThemeMode)
  const openPalette = useUiStore((state) => state.openPalette)
  const openDialog = useUiStore((state) => state.openDialog)
  const sidebarOpen = useUiStore((state) => state.sidebarOpen)
  const toggleSidebar = useUiStore((state) => state.toggleSidebar)
  const setLanguage = useUiStore((state) => state.setLanguage)
  const toggleLanguage = () =>
    setLanguage(language === 'zh-CN' ? 'en' : 'zh-CN')

  return (
    <header
      data-tauri-drag-region="deep"
      className={cn(
        'flex h-11 shrink-0 items-center gap-3 border-b border-line bg-panel pr-3 select-none',
        // macOS overlay title bar: traffic lights are inset to x 16–76 of this row.
        hasOverlayTitleBar ? 'pl-[88px]' : 'pl-3',
      )}
    >
      <button
        type="button"
        onClick={toggleSidebar}
        aria-label={sidebarOpen ? t('titlebar.hideSidebar') : t('titlebar.showSidebar')}
        title={sidebarOpen ? t('titlebar.hideSidebar') : t('titlebar.showSidebar')}
        aria-pressed={sidebarOpen}
        className={cn(ICON_NAV, 'max-[900px]:hidden')}
      >
        {sidebarOpen ? (
          <PanelLeftClose size={15} aria-hidden />
        ) : (
          <PanelLeftOpen size={15} aria-hidden />
        )}
      </button>

      <nav
        aria-label={t('nav.library')}
        className="ml-1 hidden items-center gap-1 max-[900px]:flex"
      >
        <NavLink
          to="/"
          end
          className={({ isActive }) =>
            cn(NARROW_NAV, isActive && NARROW_NAV_ACTIVE)
          }
        >
          {t('nav.allSkills')}
        </NavLink>
      </nav>

      <div className="ml-auto flex shrink-0 items-center gap-2">
        <span className="hidden items-center gap-1.5 text-xs text-muted lg:flex">
          <span
            aria-hidden
            className="size-1.5 rounded-full bg-ok shadow-[0_0_0_3px_var(--sp-ok-soft)]"
          />
          {lastCheckedAt
            ? t('titlebar.checked', {
                time: formatRelative(lastCheckedAt, language),
              })
            : t('titlebar.notChecked')}
        </span>

        <div className="flex items-center gap-0.5">
          <button
            type="button"
            onClick={() => openDialog({ kind: 'activity' })}
            aria-label={t('nav.activity')}
            title={t('nav.activity')}
            className={ICON_NAV}
          >
            <Activity size={14} aria-hidden />
          </button>
          <button
            type="button"
            onClick={toggleLanguage}
            aria-label={t('titlebar.switchLanguage')}
            title={t('titlebar.switchLanguage')}
            className={ICON_NAV}
          >
            <Languages size={14} aria-hidden />
          </button>
        </div>

        <Button
          variant="ghost"
          size="icon"
          onClick={() => setThemeMode(resolved === 'dark' ? 'light' : 'dark')}
          aria-label={
            resolved === 'dark'
              ? t('titlebar.themeToLight')
              : t('titlebar.themeToDark')
          }
          title={
            resolved === 'dark'
              ? t('titlebar.themeToLight')
              : t('titlebar.themeToDark')
          }
        >
          {resolved === 'dark' ? (
            <Sun size={14} aria-hidden />
          ) : (
            <Moon size={14} aria-hidden />
          )}
        </Button>

        <Button
          variant="ghost"
          size="sm"
          onClick={openPalette}
          aria-label={t('titlebar.command')}
          title={t('titlebar.command')}
          className="gap-2 pr-1.5 text-muted hover:text-ink"
        >
          <span className="hidden text-[12px] sm:inline">
            {t('titlebar.command')}
          </span>
          <Kbd>{commandShortcut}</Kbd>
        </Button>
      </div>
    </header>
  )
}
