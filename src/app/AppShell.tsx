import { useEffect, useRef } from 'react'
import { Navigate, Route, Routes } from 'react-router-dom'
import { LibraryPage } from '@/features/library/LibraryPage'
import { useReconcile } from '@/features/sync/useReconcile'
import { TargetsPage } from '@/features/targets/TargetsPage'
import { useI18n } from '@/i18n'
import { useUiStore } from '@/lib/uiStore'
import { CommandPalette } from './CommandPalette'
import { DetailColumn } from './DetailColumn'
import { DialogHost } from './DialogHost'
import { Sidebar } from './Sidebar'
import { TitleBar } from './TitleBar'

export function AppShell() {
  const { t, language } = useI18n()
  const reconcile = useReconcile()
  const reconcileNow = reconcile.mutate

  const openPalette = useUiStore((state) => state.openPalette)
  const closePalette = useUiStore((state) => state.closePalette)

  const didReconcile = useRef(false)

  useEffect(() => {
    document.documentElement.lang = language
    document.title = t('app.windowTitle')
  }, [language, t])

  useEffect(() => {
    if (didReconcile.current) return
    didReconcile.current = true
    reconcileNow()
  }, [reconcileNow])

  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      const mod = event.metaKey || event.ctrlKey
      if (mod && event.key.toLowerCase() === 'k') {
        event.preventDefault()
        const state = useUiStore.getState()
        if (state.paletteOpen) closePalette()
        else openPalette()
        return
      }

      if (event.key === 'Escape') {
        const state = useUiStore.getState()
        if (
          !state.paletteOpen &&
          state.dialog.kind === 'none' &&
          state.detailDrawerOpen
        ) {
          state.setDetailDrawerOpen(false)
        }
      }
    }

    window.addEventListener('keydown', onKeyDown)
    return () => window.removeEventListener('keydown', onKeyDown)
  }, [openPalette, closePalette, reconcileNow])

  return (
    <div className="flex h-dvh flex-col overflow-hidden bg-bg text-ink">
      <TitleBar />

      <div className="flex min-h-0 flex-1">
        <Sidebar />
        <main className="min-h-0 min-w-0 flex-1 overflow-hidden">
          <Routes>
            <Route path="/" element={<LibraryPage />} />
            <Route path="/skills/:slug" element={<LibraryPage />} />
            <Route path="/targets" element={<TargetsPage />} />
            <Route path="*" element={<Navigate to="/" replace />} />
          </Routes>
        </main>
      </div>

      <DetailColumn />
      <CommandPalette />
      <DialogHost />
    </div>
  )
}
