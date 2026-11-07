# Harness icons (built-in defaults)

One monochrome SVG per harness, named after the target slug the detector uses
(`claude-code`, `codex`, `cursor`, `gemini-cli`, `opencode`, `deepseek`,
`qoder`, `workbuddy`).

These files are **seeds only**: on startup the app copies any missing one into
`<data_dir>/icons/` and then reads icons from there. Editing the runtime copy
(add `<slug>.svg` for a new harness) never requires repackaging, and existing
files are never overwritten by these defaults.

- Marks come from [lobe-icons](https://github.com/lobehub/lobe-icons) (MIT).
  Each logo remains the trademark of its owner and is used here only to
  identify the harness it belongs to.
- Files are 24×24 `viewBox` single-colour SVGs. `TargetIcon` draws them as a
  CSS mask: a hex `fill` on the root `<svg>` becomes the mark colour, while
  `currentColor` follows the theme (use it for black-brand marks).
