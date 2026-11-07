import { isTauri } from './tauri'

const isMacPlatform =
  typeof navigator !== 'undefined' &&
  /Mac|iPhone|iPad|iPod/.test(navigator.platform || navigator.userAgent)

export const isMac = isMacPlatform

/**
 * macOS window with `titleBarStyle: "Overlay"`: the traffic lights float over
 * the app's own header row, which then has to clear them on the left.
 */
export const hasOverlayTitleBar = isMac && isTauri()

/** Command palette shortcut shown in the UI; the handler accepts Ctrl as well. */
export const commandShortcut = isMac ? '⌘K' : 'Ctrl K'

export function isTextInput(target: EventTarget | null): boolean {
  if (!(target instanceof HTMLElement)) return false
  if (target.isContentEditable) return true
  const tag = target.tagName
  return tag === 'INPUT' || tag === 'TEXTAREA' || tag === 'SELECT'
}
