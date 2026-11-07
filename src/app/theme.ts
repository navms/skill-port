import { useEffect, useRef, useSyncExternalStore } from 'react'
import { useUiStore } from '@/lib/uiStore'

type ResolvedTheme = 'light' | 'dark'

const QUERY = '(prefers-color-scheme: light)'

function subscribe(onChange: () => void): () => void {
  const media = window.matchMedia(QUERY)
  media.addEventListener('change', onChange)
  return () => media.removeEventListener('change', onChange)
}

function getSystemTheme(): ResolvedTheme {
  return window.matchMedia(QUERY).matches ? 'light' : 'dark'
}

export function useResolvedTheme(): ResolvedTheme {
  const themeMode = useUiStore((state) => state.themeMode)
  const systemTheme = useSyncExternalStore<ResolvedTheme>(
    subscribe,
    getSystemTheme,
    () => 'dark',
  )
  return themeMode === 'system' ? systemTheme : themeMode
}

function prefersReducedMotion(): boolean {
  return window.matchMedia('(prefers-reduced-motion: reduce)').matches
}

/** Keeps the <html data-theme> attribute in sync with the selected theme. */
export function ThemeSync(): null {
  const resolved = useResolvedTheme()
  const applied = useRef<ResolvedTheme | null>(null)

  useEffect(() => {
    const previous = applied.current
    applied.current = resolved
    if (previous === resolved) return

    const root = document.documentElement
    const apply = () => {
      root.dataset.theme = resolved
      root.style.colorScheme = resolved
    }

    /*
     * The first application is already done by `theme-boot.js` before paint
     * (and must not animate, or the app would fade in from the wrong theme).
     * Later switches cross-fade through the View Transitions API; browsers
     * without it just repaint, which is the pre-existing behaviour.
     */
    if (
      previous === null ||
      prefersReducedMotion() ||
      typeof document.startViewTransition !== 'function'
    ) {
      apply()
      return
    }

    document.startViewTransition(apply)
  }, [resolved])

  return null
}
