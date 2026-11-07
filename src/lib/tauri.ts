/** True when the page is running inside a Tauri webview rather than a browser. */
export function isTauri(): boolean {
  return (
    typeof window !== 'undefined' &&
    Object.prototype.hasOwnProperty.call(window, '__TAURI_INTERNALS__')
  )
}
