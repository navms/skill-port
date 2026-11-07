import { describe, expect, it } from 'vitest'
import { parseSkillMd, slugify } from './frontmatter'

const VALID = `---
name: Demo Skill
description: Does demo things
category: 工具
---

# Demo
`

describe('parseSkillMd', () => {
  it('parses name and description, ignoring other fields', () => {
    const parsed = parseSkillMd(VALID)
    expect(parsed.ok).toBe(true)
    if (!parsed.ok) return
    expect(parsed.meta).toEqual({
      name: 'Demo Skill',
      description: 'Does demo things',
    })
    expect(parsed.body.trim()).toBe('# Demo')
  })

  it('removes a UTF-8 BOM before parsing', () => {
    expect(parseSkillMd(`\uFEFF${VALID}`).ok).toBe(true)
  })

  it('accepts quoted values', () => {
    const parsed = parseSkillMd(
      '---\nname: "Quoted"\ndescription: \'Quoted too\'\n---\n',
    )
    expect(parsed.ok).toBe(true)
    if (!parsed.ok) return
    expect(parsed.meta.name).toBe('Quoted')
    expect(parsed.meta.description).toBe('Quoted too')
  })

  it('rejects a file without frontmatter', () => {
    const parsed = parseSkillMd('# Just a heading\n')
    expect(parsed.ok).toBe(false)
    if (parsed.ok) return
    expect(parsed.code).toBe('InvalidFrontmatter')
  })

  it('rejects malformed frontmatter lines', () => {
    const parsed = parseSkillMd(
      '---\nname: Demo\ndescription: Fine\nnot a yaml line\n---\n',
    )
    expect(parsed.ok).toBe(false)
    if (parsed.ok) return
    expect(parsed.code).toBe('InvalidFrontmatter')
  })

  it('accepts nested frontmatter that serde_yaml also accepts', () => {
    const parsed = parseSkillMd(
      [
        '---',
        'name: anysearch',
        'description: "Real-time search engine."',
        'version: 3.1.1',
        'authors:',
        '  - AnySearch Team',
        'credentials:',
        '  - name: ANYSEARCH_API_KEY',
        '    required: false',
        'category: 工具',
        '---',
        '',
        '## Overview',
        '',
      ].join('\n'),
    )
    expect(parsed.ok).toBe(true)
    if (!parsed.ok) return
    expect(parsed.meta.description).toBe('Real-time search engine.')
    expect(parsed.body.startsWith('\n## Overview')).toBe(true)
  })

  it('rejects a missing name', () => {
    const parsed = parseSkillMd('---\ndescription: Fine\n---\n')
    expect(parsed.ok).toBe(false)
    if (parsed.ok) return
    expect(parsed.code).toBe('MissingName')
  })

  it('rejects a blank name', () => {
    const parsed = parseSkillMd('---\nname: "  "\ndescription: Fine\n---\n')
    expect(parsed.ok).toBe(false)
    if (parsed.ok) return
    expect(parsed.code).toBe('MissingName')
  })

  it('rejects a missing description', () => {
    const parsed = parseSkillMd('---\nname: Demo\n---\n')
    expect(parsed.ok).toBe(false)
    if (parsed.ok) return
    expect(parsed.code).toBe('MissingDescription')
  })

  it('keeps unknown frontmatter fields from blocking the import', () => {
    const parsed = parseSkillMd(
      '---\nname: Demo\ndescription: Fine\nallowed-tools: bash\n---\n',
    )
    expect(parsed.ok).toBe(true)
  })
})

describe('slugify', () => {
  it('builds lower-case kebab slugs', () => {
    expect(slugify('Demo Skill')).toBe('demo-skill')
    expect(slugify('  Spaced   Out  ')).toBe('spaced-out')
    expect(slugify('A/B Testing!')).toBe('a-b-testing')
  })

  it('rejects slugs that are too short or reserved', () => {
    expect(slugify('a')).toBeNull()
    expect(slugify('!')).toBeNull()
    expect(slugify('skillport')).toBeNull()
    // Underscores cannot appear in a slug, so the reserved `node_modules`
    // directory can never collide with a valid slug; it normalizes instead.
    expect(slugify('node_modules')).toBe('node-modules')
    expect(slugify('.')).toBeNull()
  })

  it('caps slug length at 63 characters', () => {
    const slug = slugify('a'.repeat(80))
    expect(slug).not.toBeNull()
    expect((slug ?? '').length).toBeLessThanOrEqual(63)
  })
})
