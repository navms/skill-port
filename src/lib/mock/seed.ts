import type {
  ActivityEntry,
  AppSettings,
  Install,
  InstallState,
  LinkMode,
  SkillDetail,
  Target,
} from '../types'

interface InstallSeed {
  targetId: string
  status: InstallState
  mode?: LinkMode
  error?: string
}

interface SkillSeed {
  slug: string
  name: string
  description: string
  category: string
  updatedMinutesAgo: number
  sizeBytes: number
  body: string
  installs: InstallSeed[]
}

export interface MockSeed {
  targets: Target[]
  skills: SkillDetail[]
  activity: ActivityEntry[]
  settings: AppSettings
  nextActivityId: number
}

const TARGET_SEEDS: readonly {
  id: string
  slug: string
  name: string
  skillsDir: string
  linkMode: LinkMode
  enabled: boolean
}[] = [
  {
    id: 'tgt_claude',
    slug: 'claude-code',
    name: 'Claude Code',
    skillsDir: '~/.claude/skills',
    linkMode: 'symlink',
    enabled: true,
  },
  {
    id: 'tgt_codex',
    slug: 'codex',
    name: 'Codex CLI',
    skillsDir: '~/.codex/skills',
    linkMode: 'copy',
    enabled: true,
  },
  {
    id: 'tgt_cursor',
    slug: 'cursor',
    name: 'Cursor',
    skillsDir: '~/.cursor/skills',
    linkMode: 'symlink',
    enabled: true,
  },
  {
    id: 'tgt_gemini',
    slug: 'gemini-cli',
    name: 'Gemini CLI',
    skillsDir: '~/.gemini/skills',
    linkMode: 'symlink',
    enabled: true,
  },
  {
    id: 'tgt_opencode',
    slug: 'opencode',
    name: 'OpenCode',
    skillsDir: '~/.config/opencode/skills',
    linkMode: 'symlink',
    enabled: false,
  },
]

const SKILL_SEEDS: readonly SkillSeed[] = [
  {
    slug: 'skill-creator',
    name: 'Skill Creator',
    description: '创建、修改并评估 skill —— 让 agent 学会新本事',
    category: '元技能',
    updatedMinutesAgo: 120,
    sizeBytes: 6554,
    body: 'Use this skill when the user wants to create a new skill, modify an existing one, or measure whether a skill triggers.\nStart from the trigger description, not from the implementation.',
    installs: [
      { targetId: 'tgt_claude', status: 'ok' },
      { targetId: 'tgt_codex', status: 'ok', mode: 'copy' },
      { targetId: 'tgt_cursor', status: 'ok' },
      { targetId: 'tgt_gemini', status: 'ok' },
    ],
  },
  {
    slug: 'anysearch',
    name: 'AnySearch',
    description: '实时搜索、并行检索与网页正文抽取',
    category: '工具',
    updatedMinutesAgo: 60 * 72,
    sizeBytes: 3174,
    body: 'Real-time search across the web, vertical domains and URLs.\nPrefer parallel batch queries when the question has several facets.',
    installs: [
      { targetId: 'tgt_claude', status: 'ok' },
      { targetId: 'tgt_codex', status: 'ok', mode: 'copy' },
      { targetId: 'tgt_cursor', status: 'ok' },
      { targetId: 'tgt_gemini', status: 'ok' },
    ],
  },
  {
    slug: 'article-ingest',
    name: 'Article Ingest',
    description: '把文章抓成本地 Markdown 素材，并翻译成中文',
    category: '内容',
    updatedMinutesAgo: 60 * 5,
    sizeBytes: 9011,
    body: 'Fetch a blog post into Markdown with images and front matter.\nTranslate non-Chinese sources, then hand off to the blog publisher.',
    installs: [
      { targetId: 'tgt_claude', status: 'ok' },
      { targetId: 'tgt_codex', status: 'stale', mode: 'copy' },
      { targetId: 'tgt_cursor', status: 'ok' },
    ],
  },
  {
    slug: 'huashu-design',
    name: 'Huashu Design',
    description: 'HTML 高保真原型、幻灯片与动效',
    category: '设计',
    updatedMinutesAgo: 60 * 48,
    sizeBytes: 22937,
    body: 'Produce high-fidelity HTML prototypes, decks and motion pieces.\nAlways start from three distinct directions before building.',
    installs: [
      { targetId: 'tgt_claude', status: 'ok' },
      { targetId: 'tgt_codex', status: 'ok', mode: 'copy' },
      { targetId: 'tgt_cursor', status: 'ok' },
      { targetId: 'tgt_gemini', status: 'ok' },
    ],
  },
  {
    slug: 'ddd-dev-spec',
    name: 'DDD Dev Spec',
    description: 'SpringBoot DDD 分层开发规范与代码审查',
    category: '后端',
    updatedMinutesAgo: 60 * 24,
    sizeBytes: 31948,
    body: 'Layering rules for boot-framework + MyBatis-Plus projects.\nAppService, DomainService, Repository and MapStruct conventions.',
    installs: [
      { targetId: 'tgt_claude', status: 'ok' },
      { targetId: 'tgt_cursor', status: 'conflict' },
    ],
  },
  {
    slug: 'daily-ai-digest',
    name: 'Daily AI Digest',
    description: '每日 AI 前沿简报，检索、筛选、成稿',
    category: '内容',
    updatedMinutesAgo: 60 * 24 * 4,
    sizeBytes: 5837,
    body: "Search the previous day's AI news from a programmer's angle.\nWrite a Chinese digest and publish a new Feishu doc.",
    installs: [
      { targetId: 'tgt_claude', status: 'ok' },
      { targetId: 'tgt_cursor', status: 'ok' },
    ],
  },
  {
    slug: 'organic-blog-publish',
    name: 'Organic Blog Publish',
    description: '把本地 Markdown 文章发布到 navms.cc',
    category: '内容',
    updatedMinutesAgo: 60 * 24 * 6,
    sizeBytes: 13209,
    body: 'Convert SVGs, upload images, then import Markdown into the blog.\nVerify the packaging limit before triggering the import.',
    installs: [
      { targetId: 'tgt_claude', status: 'ok' },
      { targetId: 'tgt_codex', status: 'missing', mode: 'copy' },
    ],
  },
  {
    slug: 'fresh-eyes-review',
    name: 'Fresh Eyes Review',
    description: '用干净上下文的第二个 agent 复审改动',
    category: '元技能',
    updatedMinutesAgo: 60 * 24 * 7,
    sizeBytes: 4608,
    body: 'Spawn a second agent without the working context.\nEvery finding needs a location, a repro command and real output.',
    installs: [
      { targetId: 'tgt_claude', status: 'ok' },
      { targetId: 'tgt_codex', status: 'ok', mode: 'copy' },
      {
        targetId: 'tgt_gemini',
        status: 'error',
        error: '权限不足：无法写入目标目录（EACCES）',
      },
    ],
  },
  {
    slug: 'refactoring-ui',
    name: 'Refactoring UI',
    description: '视觉层级、间距与色彩的系统性审计',
    category: '设计',
    updatedMinutesAgo: 60 * 24 * 14,
    sizeBytes: 18739,
    body: 'Audit hierarchy, spacing, color and depth in a web UI.\nStart in grayscale, then add one accent with a job to do.',
    installs: [
      { targetId: 'tgt_claude', status: 'ok' },
      { targetId: 'tgt_gemini', status: 'pending' },
    ],
  },
]

const ACTIVITY_SEEDS: readonly {
  minutesAgo: number
  action: ActivityEntry['action']
  skillSlug: string | null
  targetId: string | null
  result: ActivityEntry['result']
  detail: string
}[] = [
  {
    minutesAgo: 20,
    action: 'reconcile',
    skillSlug: null,
    targetId: null,
    result: 'ok',
    detail: '核对 26 条链接 · 5 项待处理',
  },
  {
    minutesAgo: 45,
    action: 'conflict',
    skillSlug: 'ddd-dev-spec',
    targetId: 'tgt_cursor',
    result: 'failed',
    detail: 'Cursor 里发现外部同名目录，未改动',
  },
  {
    minutesAgo: 120,
    action: 'link',
    skillSlug: 'skill-creator',
    targetId: 'tgt_gemini',
    result: 'ok',
    detail: 'skill-creator → Gemini CLI 已链接',
  },
  {
    minutesAgo: 60 * 5,
    action: 'update',
    skillSlug: 'huashu-design',
    targetId: null,
    result: 'ok',
    detail: 'huashu-design 的 SKILL.md 已更新',
  },
  {
    minutesAgo: 60 * 24,
    action: 'copy',
    skillSlug: 'fresh-eyes-review',
    targetId: 'tgt_codex',
    result: 'ok',
    detail: 'fresh-eyes-review 拷贝到 Codex CLI',
  },
  {
    minutesAgo: 60 * 48,
    action: 'sync',
    skillSlug: 'organic-blog-publish',
    targetId: 'tgt_codex',
    result: 'failed',
    detail: 'organic-blog-publish 的目标路径不存在',
  },
  {
    minutesAgo: 60 * 24 * 6,
    action: 'import',
    skillSlug: 'organic-blog-publish',
    targetId: null,
    result: 'ok',
    detail: '从 ~/skills/organic-blog-publish 导入',
  },
  {
    minutesAgo: 60 * 24 * 7,
    action: 'target',
    skillSlug: null,
    targetId: 'tgt_opencode',
    result: 'ok',
    detail: 'OpenCode 已停用，已有链接保持不变',
  },
]

export function fakeHash(input: string): string {
  let hash = 2166136261
  let out = ''
  for (let index = 0; index < 64; index += 1) {
    hash ^= input.charCodeAt(index % Math.max(input.length, 1))
    hash = Math.imul(hash, 16777619)
    out += ((hash >>> 0) % 16).toString(16)
  }
  return out
}

/** Category is SkillPort's own field, so it is deliberately absent here. */
export function buildSkillMd(skill: {
  name: string
  description: string
  body: string
}): string {
  return `---\nname: ${skill.name}\ndescription: ${skill.description}\n---\n\n# ${skill.name}\n\n${skill.body}\n`
}

function isoMinutesAgo(minutes: number, now: number): string {
  return new Date(now - minutes * 60_000).toISOString()
}

export function buildSeed(now = Date.now()): MockSeed {
  const targets: Target[] = TARGET_SEEDS.map((target) => ({
    ...target,
    kind: 'skills_dir',
    detected: true,
    createdAt: isoMinutesAgo(60 * 24 * 30, now),
    updatedAt: isoMinutesAgo(60 * 24, now),
  }))

  const targetById = new Map(targets.map((target) => [target.id, target]))
  let installId = 1

  const skills: SkillDetail[] = SKILL_SEEDS.map((seed) => {
    const contentHash = fakeHash(seed.slug)
    const skillId = `skl_${seed.slug}`
    const installs: Install[] = seed.installs.map((installSeed) => {
      const target = targetById.get(installSeed.targetId)
      const mode = installSeed.mode ?? target?.linkMode ?? 'symlink'
      const staleHash = fakeHash(`${seed.slug}-previous`)
      return {
        id: installId++,
        skillId,
        targetId: installSeed.targetId,
        mode,
        actualKind:
          installSeed.status === 'missing' || installSeed.status === 'pending'
            ? null
            : installSeed.status === 'conflict'
              ? null
              : mode,
        status: installSeed.status,
        targetPath: `${target?.skillsDir ?? ''}/${seed.slug}`,
        linkedHash:
          installSeed.status === 'ok'
            ? contentHash
            : installSeed.status === 'stale'
              ? staleHash
              : null,
        ignored: false,
        lastError: installSeed.error ?? null,
        lastChecked: isoMinutesAgo(2, now),
        lastSynced:
          installSeed.status === 'ok' || installSeed.status === 'stale'
            ? isoMinutesAgo(60 * 24, now)
            : null,
      }
    })

    return {
      id: skillId,
      slug: seed.slug,
      name: seed.name,
      description: seed.description,
      category: seed.category,
      sourceKind: 'import',
      sourceRef: null,
      contentHash,
      sizeBytes: seed.sizeBytes,
      createdAt: isoMinutesAgo(60 * 24 * 21, now),
      updatedAt: isoMinutesAgo(seed.updatedMinutesAgo, now),
      installs,
      path: `~/Library/Application Support/com.skillport.desktop/library/${seed.slug}`,
      skillMd: buildSkillMd(seed),
    }
  })

  const skillBySlug = new Map(skills.map((skill) => [skill.slug, skill]))
  const activity: ActivityEntry[] = ACTIVITY_SEEDS.map((seed, index) => {
    const skill = seed.skillSlug ? skillBySlug.get(seed.skillSlug) : undefined
    const target = seed.targetId ? targetById.get(seed.targetId) : undefined
    return {
      id: ACTIVITY_SEEDS.length - index,
      ts: isoMinutesAgo(seed.minutesAgo, now),
      action: seed.action,
      skillId: skill?.id ?? null,
      skillName: skill?.slug ?? null,
      targetId: target?.id ?? null,
      targetName: target?.name ?? null,
      result: seed.result,
      detail: seed.detail,
    }
  })

  return {
    targets,
    skills,
    activity,
    settings: {
      defaultLinkMode: 'symlink',
      libraryPath:
        '~/Library/Application Support/com.skillport.desktop/library',
      dataDir: '~/Library/Application Support/com.skillport.desktop',
      backupRetention: 20,
    },
    nextActivityId: ACTIVITY_SEEDS.length + 1,
  }
}
