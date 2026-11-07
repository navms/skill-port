import type { ImportErrorCode } from './types'

export interface ParsedMeta {
  name: string
  description: string
}

export type ParsedSkill =
  | { ok: true; meta: ParsedMeta; body: string }
  | { ok: false; code: ImportErrorCode; detail: string }

const RESERVED_SLUGS = new Set(['.', '..', 'skillport', 'node_modules'])

function unquote(value: string): string {
  const trimmed = value.trim()
  if (trimmed.length >= 2) {
    const first = trimmed[0]
    const last = trimmed[trimmed.length - 1]
    if ((first === '"' && last === '"') || (first === "'" && last === "'")) {
      return trimmed.slice(1, -1)
    }
  }
  return trimmed
}

/**
 * Minimal frontmatter reader for client-side validation and the mock backend.
 * The real parser is serde_yaml; both enforce the same v1 rules: name and
 * description must be non-empty.
 *
 * Only top-level `key: value` lines are read. Nested values (indented maps and
 * sequences, e.g. `authors:` / `credentials:`) are valid YAML and are simply
 * not modelled here \u2014 rejecting them would flag files that serde_yaml accepts.
 */
export function parseSkillMd(source: string): ParsedSkill {
  const normalized = source.replace(/^\uFEFF/, '').replace(/\r\n/g, '\n')
  const match = normalized.match(/^---\n([\s\S]*?)\n---(?:\n|$)/)
  if (!match) {
    return {
      ok: false,
      code: 'InvalidFrontmatter',
      detail: 'Expected a YAML frontmatter block wrapped in --- lines.',
    }
  }

  const block = match[1] ?? ''
  const values = new Map<string, string>()
  for (const line of block.split('\n')) {
    // Indented lines belong to a nested map/sequence we don't model.
    if (!line || /^\s/.test(line) || line.startsWith('#')) continue
    const pair = line.match(/^([A-Za-z0-9_-]+):[ \t]*(.*)$/)
    if (!pair) {
      return {
        ok: false,
        code: 'InvalidFrontmatter',
        detail: `Could not parse frontmatter line: ${line}`,
      }
    }
    values.set(pair[1] ?? '', unquote(pair[2] ?? ''))
  }

  const name = values.get('name')?.trim() ?? ''
  if (!name) {
    return { ok: false, code: 'MissingName', detail: 'name is required.' }
  }
  const description = values.get('description')?.trim() ?? ''
  if (!description) {
    return {
      ok: false,
      code: 'MissingDescription',
      detail: 'description is required.',
    }
  }

  // `category` is SkillPort's own field and lives in the DB; any such key in
  // someone else's frontmatter is treated as an unknown field and ignored.
  return {
    ok: true,
    meta: { name, description },
    body: normalized.slice(match[0].length),
  }
}

export function slugify(input: string): string | null {
  const slug = input
    .toLowerCase()
    .normalize('NFKD')
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/^-+|-+$/g, '')
    .slice(0, 63)
    .replace(/-+$/g, '')

  if (slug.length < 2) return null
  if (RESERVED_SLUGS.has(slug)) return null
  if (!/^[a-z0-9]+(-[a-z0-9]+)*$/.test(slug)) return null
  return slug
}
