import * as DialogPrimitive from '@radix-ui/react-dialog'
import { Search } from 'lucide-react'
import {
  useEffect,
  useMemo,
  useRef,
  useState,
  type KeyboardEvent,
} from 'react'
import { useNavigate } from 'react-router-dom'
import { Kbd } from '@/components/ui/kbd'
import { useReconcile } from '@/features/sync/useReconcile'
import { useI18n } from '@/i18n'
import { useSkills, useTargets } from '@/lib/queries'
import { useUiStore } from '@/lib/uiStore'
import { buildPaletteActions } from './paletteActions'
import type { PaletteItem } from './paletteTypes'
import { PaletteList } from './PaletteList'
import { useResolvedTheme } from './theme'

export function CommandPalette() {
  const { t } = useI18n()
  const navigate = useNavigate()
  const resolved = useResolvedTheme()
  const reconcile = useReconcile()

  const paletteOpen = useUiStore((state) => state.paletteOpen)
  const closePalette = useUiStore((state) => state.closePalette)
  const openDialog = useUiStore((state) => state.openDialog)
  const setDetailDrawerOpen = useUiStore(
    (state) => state.setDetailDrawerOpen,
  )
  const setChip = useUiStore((state) => state.setChip)
  const setThemeMode = useUiStore((state) => state.setThemeMode)
  const language = useUiStore((state) => state.language)
  const setLanguage = useUiStore((state) => state.setLanguage)

  const skills = useSkills().data ?? []
  const targets = useTargets().data ?? []

  const [query, setQuery] = useState('')
  const [active, setActive] = useState(0)
  const listRef = useRef<HTMLDivElement>(null)

  useEffect(() => {
    if (paletteOpen) {
      setQuery('')
      setActive(0)
    }
  }, [paletteOpen])

  const actions = useMemo(
    () =>
      buildPaletteActions({
        t,
        resolved,
        openDialog,
        navigate,
        setChip,
        setThemeMode,
        language,
        setLanguage,
        reconcile: reconcile.mutate,
      }),
    [
      t,
      resolved,
      openDialog,
      navigate,
      setChip,
      setThemeMode,
      language,
      setLanguage,
      reconcile,
    ],
  )

  const normalized = query.trim().toLowerCase()

  const filteredActions = actions.filter((action) =>
    `${action.label} ${action.sub}`.toLowerCase().includes(normalized),
  )
  const filteredSkills = skills
    .filter((skill) =>
      `${skill.slug} ${skill.name} ${skill.description}`
        .toLowerCase()
        .includes(normalized),
    )
    .slice(0, 8)

  const items: PaletteItem[] = [
    ...filteredActions,
    ...filteredSkills.map(
      (skill): PaletteItem => ({ kind: 'skill', id: skill.id, skill }),
    ),
  ]

  const clampedActive =
    items.length === 0 ? 0 : Math.min(active, items.length - 1)

  const runItem = (item: PaletteItem) => {
    closePalette()
    if (item.kind === 'action') {
      item.run()
      return
    }
    setDetailDrawerOpen(true)
    navigate(`/skills/${encodeURIComponent(item.skill.slug)}`)
  }

  const onKeyDown = (event: KeyboardEvent<HTMLInputElement>) => {
    if (event.key === 'ArrowDown') {
      event.preventDefault()
      if (items.length > 0) {
        setActive((clampedActive + 1) % items.length)
      }
    } else if (event.key === 'ArrowUp') {
      event.preventDefault()
      if (items.length > 0) {
        setActive((clampedActive - 1 + items.length) % items.length)
      }
    } else if (event.key === 'Enter') {
      event.preventDefault()
      const item = items[clampedActive]
      if (item) runItem(item)
    }
  }

  return (
    <DialogPrimitive.Root
      open={paletteOpen}
      onOpenChange={(open) => {
        if (!open) closePalette()
      }}
    >
      <DialogPrimitive.Portal>
        <DialogPrimitive.Overlay className="sp-overlay fixed inset-0 z-[80] bg-scrim backdrop-blur-[2px]" />
        <DialogPrimitive.Content
          className="sp-palette fixed top-[12vh] left-1/2 z-[80] flex max-h-[70vh] w-[min(620px,92vw)] -translate-x-1/2 flex-col overflow-hidden rounded-[10px] border border-line-2 bg-panel shadow-[var(--sp-panel-shadow)]"
        >
          <DialogPrimitive.Title className="sr-only">
            {t('palette.placeholder')}
          </DialogPrimitive.Title>
          <DialogPrimitive.Description className="sr-only">
            {t('palette.placeholder')}
          </DialogPrimitive.Description>

          <div className="flex h-11 shrink-0 items-center gap-2.5 border-b border-line px-3.5">
            <Search size={14} aria-hidden className="shrink-0 text-faint" />
            <input
              autoFocus
              value={query}
              onChange={(event) => {
                setQuery(event.target.value)
                setActive(0)
              }}
              onKeyDown={onKeyDown}
              placeholder={t('palette.placeholder')}
              aria-label={t('common.search')}
              role="combobox"
              aria-expanded
              aria-controls="palette-list"
              aria-activedescendant={
                items.length > 0 ? `palette-item-${clampedActive}` : undefined
              }
              className="h-full min-w-0 flex-1 border-0 bg-transparent text-[13px] text-ink outline-none placeholder:text-faint"
            />
            <Kbd>ESC</Kbd>
          </div>

          <PaletteList
            actions={filteredActions}
            skills={filteredSkills}
            targets={targets}
            active={clampedActive}
            listRef={listRef}
            onRun={runItem}
            onHover={setActive}
          />

          <div className="flex shrink-0 items-center gap-3 border-t border-line px-3.5 py-1.5 font-mono text-[10px] text-faint">
            <span>{t('palette.move')}</span>
            <span>{t('palette.run')}</span>
            <span className="ml-auto">{t('palette.close')}</span>
          </div>
        </DialogPrimitive.Content>
      </DialogPrimitive.Portal>
    </DialogPrimitive.Root>
  )
}
