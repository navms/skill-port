import { useEffect, useSyncExternalStore } from 'react'
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

/** Keeps the <html data-theme> attribute in sync with the selected theme. */
export function ThemeSync(): null {
  const resolved = useResolvedTheme()

  useEffect(() => {
    document.documentElement.dataset.theme = resolved
    document.documentElement.style.colorScheme = resolved
  }, [resolved])

  return null
}
