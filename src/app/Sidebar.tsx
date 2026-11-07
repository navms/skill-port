import { Boxes, Target } from 'lucide-react'
import type { ReactNode } from 'react'
import { useLocation, useNavigate } from 'react-router-dom'
import { useI18n } from '@/i18n'
import { categoryCounts } from '@/lib/filter'
import { useSkills, useTargets } from '@/lib/queries'
import { useUiStore } from '@/lib/uiStore'
import { cn } from '@/lib/utils'

const NAV_BASE =
  'relative flex h-8 w-full items-center gap-2 rounded-md px-2 text-left text-[13px] text-ink-2 transition-colors hover:bg-panel-2 hover:text-ink'
const NAV_ACTIVE =
  'bg-raise text-ink before:absolute before:top-1.5 before:bottom-1.5 before:-left-3 before:w-0.5 before:rounded-full before:bg-accent'

function SideLabel({ children }: { children: ReactNode }) {
  return (
    <div className="px-2 pt-3.5 pb-1.5 font-mono text-[10px] tracking-[0.14em] text-faint uppercase">
      {children}
    </div>
  )
}

function Count({ children }: { children: ReactNode }) {
  return (
    <span className="ml-auto font-mono text-[11px] text-faint">{children}</span>
  )
}

export function Sidebar() {
  const { t } = useI18n()
  const navigate = useNavigate()
  const location = useLocation()
  const skillsQuery = useSkills()
  const targetsQuery = useTargets()

  const skills = skillsQuery.data ?? []
  const targets = targetsQuery.data ?? []
  const categories = categoryCounts(skills)

  const category = useUiStore((state) => state.category)
  const selectCategory = useUiStore((state) => state.selectCategory)
  const clearFilters = useUiStore((state) => state.clearFilters)
  const sidebarOpen = useUiStore((state) => state.sidebarOpen)

  const isLibrary =
    location.pathname === '/' || location.pathname.startsWith('/skills/')
  const isTargets = location.pathname === '/targets'

  const goAllSkills = () => {
    clearFilters()
    navigate('/')
  }

  const goCategory = (name: string) => {
    selectCategory(name)
    navigate('/')
  }

  return (
    <aside
      aria-hidden={!sidebarOpen}
      inert={!sidebarOpen}
      className={cn(
        'shrink-0 overflow-hidden transition-[width] duration-200 ease-out max-[900px]:hidden',
        sidebarOpen ? 'w-[244px]' : 'w-0',
      )}
    >
      <div className="flex h-full w-[244px] flex-col border-r border-line bg-panel px-3 pt-3.5 pb-2.5">
      <SideLabel>{t('nav.library')}</SideLabel>
      <button
        type="button"
        onClick={goAllSkills}
        className={cn(NAV_BASE, isLibrary && !category && NAV_ACTIVE)}
      >
        <Boxes size={14} aria-hidden className="shrink-0 text-faint" />
        <span className="truncate">{t('nav.allSkills')}</span>
        <Count>{skills.length}</Count>
      </button>

      <SideLabel>{t('nav.categories')}</SideLabel>
      <div className="space-y-0.5">
        {categories.map((item) => (
          <button
            key={item.name}
            type="button"
            onClick={() => goCategory(item.name)}
            className={cn(
              NAV_BASE,
              isLibrary && category === item.name && NAV_ACTIVE,
            )}
          >
            <span className="size-[14px] shrink-0" aria-hidden />
            <span className="truncate">{item.name}</span>
            <Count>{item.count}</Count>
          </button>
        ))}
      </div>

      <SideLabel>{t('nav.manage')}</SideLabel>
      <button
        type="button"
        onClick={() => navigate('/targets')}
        className={cn(NAV_BASE, isTargets && NAV_ACTIVE)}
      >
        <Target size={14} aria-hidden className="shrink-0 text-faint" />
        <span className="truncate">{t('nav.targets')}</span>
        <Count>{targets.length}</Count>
      </button>
      </div>
    </aside>
  )
}
