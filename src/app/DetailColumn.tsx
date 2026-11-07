import { useMatch } from 'react-router-dom'
import { SkillDetailPanel } from '@/features/library/SkillDetailPanel'
import { useI18n } from '@/i18n'
import { useSkills, useTargets } from '@/lib/queries'
import { useUiStore } from '@/lib/uiStore'
import { cn } from '@/lib/utils'

/**
 * Skill detail as a right-hand drawer.
 *
 * There is no permanent detail column: the drawer floats over the list and is
 * only mounted for a selected skill. It stays mounted while sliding out so the
 * close transition is visible; when closed it is inert and off-screen.
 */
export function DetailColumn() {
  const { t } = useI18n()
  const match = useMatch('/skills/:slug')
  const slug = match?.params.slug
  const skills = useSkills().data ?? []
  const targets = useTargets().data ?? []
  const drawerOpen = useUiStore((state) => state.detailDrawerOpen)
  const setDetailDrawerOpen = useUiStore(
    (state) => state.setDetailDrawerOpen,
  )

  const skill = slug
    ? skills.find((item) => item.slug === slug)
    : undefined

  if (!skill) return null

  return (
    <>
      <button
        type="button"
        aria-label={t('detail.close')}
        tabIndex={-1}
        onClick={() => setDetailDrawerOpen(false)}
        className={cn(
          'fixed inset-x-0 top-11 bottom-0 z-30 cursor-default bg-scrim/50 transition-opacity duration-200',
          drawerOpen ? 'opacity-100' : 'pointer-events-none opacity-0',
        )}
      />
      <SkillDetailPanel
        skill={skill}
        targets={targets}
        drawerOpen={drawerOpen}
      />
    </>
  )
}
