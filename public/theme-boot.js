// Resolve the persisted theme before first paint to avoid a flash.
// Kept as an external script so the production CSP can stay `script-src 'self'`.
;(function () {
  try {
    var mode = localStorage.getItem('skillport-theme') || 'dark'
    var resolved =
      mode === 'system'
        ? window.matchMedia('(prefers-color-scheme: light)').matches
          ? 'light'
          : 'dark'
        : mode
    document.documentElement.dataset.theme = resolved
  } catch (error) {
    document.documentElement.dataset.theme = 'dark'
  }
})()
