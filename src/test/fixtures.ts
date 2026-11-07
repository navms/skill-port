import type {
  Install,
  InstallState,
  SkillSummary,
  Target,
} from '@/lib/types'

export function makeTarget(overrides: Partial<Target> = {}): Target {
  return {
    id: 'tgt_claude',
    slug: 'claude-code',
    name: 'Claude Code',
    kind: 'skills_dir',
    skillsDir: '/tmp/skillport-test/claude/skills',
    linkMode: 'symlink',
    enabled: true,
    detected: false,
    createdAt: '2026-01-01T00:00:00.000Z',
    updatedAt: '2026-01-01T00:00:00.000Z',
    ...overrides,
  }
}

export function makeInstall(overrides: Partial<Install> = {}): Install {
  return {
    id: 1,
    skillId: 'skl_demo',
    targetId: 'tgt_claude',
    mode: 'symlink',
    actualKind: 'symlink',
    status: 'ok',
    targetPath: '/tmp/skillport-test/claude/skills/demo',
    linkedHash: 'hash',
    ignored: false,
    lastError: null,
    lastChecked: '2026-01-01T00:00:00.000Z',
    lastSynced: '2026-01-01T00:00:00.000Z',
    ...overrides,
  }
}

export function installWith(
  targetId: string,
  status: InstallState,
  overrides: Partial<Install> = {},
): Install {
  return makeInstall({
    id: Math.floor(Math.random() * 100000),
    targetId,
    status,
    ...overrides,
  })
}

export function makeSkill(overrides: Partial<SkillSummary> = {}): SkillSummary {
  return {
    id: 'skl_demo',
    slug: 'demo',
    name: 'Demo',
    description: 'A demo skill',
    category: '工具',
    sourceKind: 'import',
    sourceRef: '/tmp/skillport-test/demo',
    contentHash: 'abcdef1234567890',
    sizeBytes: 2048,
    createdAt: '2026-01-01T00:00:00.000Z',
    updatedAt: '2026-01-01T00:00:00.000Z',
    installs: [],
    ...overrides,
  }
}
