import { isTauri } from './tauri'
import { createTauriApi } from './tauriApi'
import type { Api } from './types'

/**
 * Browser dev uses the in-memory mock; the desktop webview always talks to Rust.
 * The dynamic import keeps the mock out of the production bundle.
 */
async function resolveApi(): Promise<Api> {
  if (import.meta.env.DEV && !isTauri()) {
    const { mockApi } = await import('./mockApi')
    return mockApi
  }
  return createTauriApi()
}

export const api: Api = await resolveApi()
export { isTauri }
