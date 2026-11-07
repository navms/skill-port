# AGENTS.md · SkillPort

> 本文件面向所有 coding agent（Claude Code / Codex / Cursor / Gemini CLI 等）。
> 开始任何改动前先读完本文件。代码与本文件冲突时，以代码为准，并在同一次改动里更新本文件。
>
> 产品名 **SkillPort**（已确认）；app identifier `com.skillport.desktop`。

## 1. 一句话

一个跨平台的桌面端 **Agent Skill 管理器**：技能只存一份，链接/拷贝到多个 harness 各自的技能目录，统一查看、编辑、同步、解决冲突。

## 2. 目标与非目标

### v1 必须有

- **单一真相源**：技能的权威副本只存在于本工具的库目录，不放在任何 harness 的技能目录里。
- **多 harness 分发**：同一技能可分发到 Claude Code、Codex CLI、Cursor、Gemini CLI、OpenCode 等任意数量的目标；支持符号链接与拷贝两种模式。
- **状态可见**：每个技能 × 每个目标的状态（正常 / 副本过期 / 缺失 / 冲突 / 错误 / 未启用）一眼可见。
- **冲突不静默处理**：目标目录已存在同名但不是本工具创建的目录时，标为冲突，交由用户决定，绝不自动覆盖或删除。
- **导入严格校验**：缺 `SKILL.md`、frontmatter、`name` 或 `description` 的技能拒绝导入，并报告路径与缺失字段；不写库、不建链接。
- **CLI 导入**：`skillport-cli import` 让其他 agent 不打开界面也能把技能收进同一个库，输出 JSON，见 §11.1。
- **本地优先**：无账号、无云同步、无遥测；数据是本机 SQLite + 文件目录。
- **macOS 与 Windows** 同等级支持；Linux 尽力而为，不作为发布门槛。

### v1 明确不做

- MCP 管理（v2；见第 13 节，数据模型预留）。
- 项目级技能（`.claude/skills` 之类）的批量管理——v1 只管全局/用户级目录。
- 技能市场、在线安装、版本仓库托管。
- 多设备同步、团队共享、权限体系。
- 修改 harness 自身配置（v1 只写技能目录）。

## 3. 概念与术语

| 术语 | 含义 |
|---|---|
| **Skill** | 一个包含 `SKILL.md` 的目录，可有脚本、资源、子目录。`SKILL.md` 必须有 YAML frontmatter（至少 `name`、`description`）。 |
| **Library（库）** | 本工具管理的唯一真相源目录，存放所有 Skill 的权威副本。 |
| **Target（目标 / Harness）** | 一个消费技能的宿主，例如 Claude Code，对应一个 `skills_dir`。 |
| **Install** | 一条「某个 Skill 应存在于某个 Target」的期望记录。 |
| **Mode** | 分发方式：`symlink`（默认）或 `copy`。 |
| **State** | 某条 Install 的实测状态：`ok` / `stale` / `missing` / `conflict` / `error` / `pending`。 |
| **Reconcile（核对）** | 扫描库与所有目标目录，把实测状态写回 DB。只读 + 更新状态，不修改文件。 |
| **Sync（同步）** | 根据期望状态创建/修复链接或拷贝。写操作，必须先出计划再执行。 |
| **Untracked（未托管）** | 目标目录里存在、但 DB 里没有对应 Install 的条目。默认不动；v1 暂不在 UI 列出，也不自动处理。 |

UI 文案与内部状态的映射必须集中在一处（`src/lib/status.ts`），不要在组件里硬编码中文字符串：

| 内部 | 中文 | 英文 |
|---|---|---|
| `ok` + symlink | 已链接 | Linked |
| `ok` + copy | 副本 | Copy |
| `stale` | 副本过期 | Stale |
| `missing` | 缺失 | Missing |
| `conflict` | 冲突 | Conflict |
| `error` | 异常 | Error |
| `pending` | 待同步 | Pending |

## 4. 存储布局

应用数据目录由 Tauri `app_data_dir()` 提供：

- macOS：`~/Library/Application Support/com.skillport.desktop/`
- Windows：`%APPDATA%\com.skillport.desktop\`

```
<app_data_dir>/
├─ library/
│  └─ <skill-slug>/          # 唯一真相源，目录名即 slug，稳定不变
│     ├─ SKILL.md
│     └─ ...
├─ icons/                    # harness 图标，<slug>.svg；用户可直接增删，无需重新打包
├─ db.sqlite3                # WAL 模式
├─ backups/                  # 覆盖/删除前的快照（zip 或目录副本）
└─ logs/
```

**硬约束**

- 库目录不得位于任何 target 的 `skills_dir` 内部，启动时校验；否则直接报错并引导用户修改。
- `app_data_dir` 必须可通过环境变量 `SKILLPORT_DATA_DIR` 覆盖——用于测试与便携模式。
- 测试、开发、演示一律使用临时目录，绝不读写真实的 `~/.claude/skills`、`~/.codex` 等。

## 5. 技术栈

| 层 | 选型 |
|---|---|
| 桌面壳 | Tauri 2（Rust） |
| 前端 | React 19 + TypeScript（strict）+ Vite 7 |
| 样式 | Tailwind CSS 4（`@tailwindcss/vite`，CSS-first `@theme` tokens） |
| 组件 | Radix UI primitives + 自建 shadcn 风格组件 + `lucide-react` |
| 图标 | Lucide（界面图标）；harness 图标是外置的 `icons/<slug>.svg`（见 §4），内置默认值来自 `src/assets/targets/` |
| 数据请求 | TanStack Query（IPC 缓存/失效）+ Zustand（纯 UI 状态） |
| 路由 | React Router，hash 模式（桌面端无需 history fallback） |
| 编辑器 | CodeMirror 6（`@uiw/react-codemirror`），v1 可先用 textarea |
| 数据库 | SQLite（`rusqlite` + bundled），迁移用 `rusqlite_migration` |
| 文件系统 | 全部在 Rust 侧完成；前端不直接碰 FS |
| 打包 | macOS `.dmg`（签名 + 公证）；Windows NSIS + MSI |

不在前端引入数据库驱动，不在 Rust 侧实现 UI 文案。

`src-tauri` 是 Cargo workspace 根：桌面应用、`crates/skillport-core`（业务核心，**不得依赖 Tauri**）与
`crates/skillport-cli`（CLI，见 §11.1）三个包。核心只被桌面端与 CLI 消费，任何一端都不复制业务逻辑。

## 6. 目录结构

```
skill-port/
├─ AGENTS.md
├─ CODING_STANDARDS.md        # Rust / React / Tauri 编码规范与 CR 清单
├─ src/
│  ├─ app/                    # 入口、路由、Providers、窗口 chrome
│  ├─ components/ui/          # Button / Dialog / Switch / Badge / Table ...
│  ├─ features/
│  │  ├─ library/             # 技能列表、详情、导入、编辑器
│  │  ├─ targets/             # 目标 CRUD、路径检测、模式设置
│  │  ├─ sync/                # 同步计划、队列、冲突解决
│  │  └─ activity/            # 操作日志
│  ├─ lib/                    # api.ts、types.ts、status.ts、format.ts
│  ├─ i18n/                   # zh-CN.ts（默认）、en.ts
│  └─ styles/                 # tokens.css、globals.css
├─ src-tauri/                 # Cargo workspace 根
│  ├─ src/
│  │  ├─ lib.rs               # builder、数据目录解析、command 注册；`pub use skillport_core as core`
│  │  ├─ state.rs             # AppState：Mutex<Connection> + AppPaths + PlanStore
│  │  ├─ commands/            # 薄封装：参数校验 + spawn_blocking + 事件
│  │  │  └─ library.rs / targets.rs / sync.rs / settings.rs
│  ├─ crates/
│  │  ├─ skillport-core/      # 业务核心（独立 crate，无 Tauri 依赖），桌面端与 CLI 共用
│  │  │  ├─ model.rs          # Skill / Target / Install / Plan 等 IPC 类型
│  │  │  ├─ error.rs          # AppError -> { code, message, detail }
│  │  │  ├─ paths.rs          # app dir、平台默认数据目录、slug 规则、路径守卫
│  │  │  ├─ workspace.rs      # 打开数据目录：布局、DB、摆放校验、临时残留清理
│  │  │  ├─ db.rs             # 连接、PRAGMA、只追加迁移
│  │  │  ├─ repo.rs           # skills / targets / installs / activity / settings 查询
│  │  │  ├─ frontmatter.rs    # SKILL.md 严格解析与重写
│  │  │  ├─ hash.rs           # 稳定树 hash（排除 sidecar 与系统垃圾）
│  │  │  ├─ fsops.rs          # 受控链接、拷贝、sidecar、备份、临时目录清理
│  │  │  ├─ library.rs        # 导入、列表、编辑、删除
│  │  │  ├─ target.rs         # 目标 CRUD、候选目录检测
│  │  │  ├─ logging.rs        # 文件日志（5 MB 轮转，不记技能正文）
│  │  │  ├─ plan.rs           # 同步计划存储（TTL 5 分钟 + 取消标记）
│  │  │  ├─ settings.rs       # 设置读写与校验（默认模式 / 备份保留数）
│  │  │  ├─ undo.rs           # 撤销记录：删除技能、同步操作（基于备份恢复）
│  │  │  └─ sync/             # inspect / planner / apply / owned-delete / reconcile
│  │  └─ skillport-cli/       # CLI（二进制 `skillport-cli`），供其他 agent 调用，见 §11.1
│  ├─ tests/                  # 集成测试（临时目录，覆盖路径守卫/导入/状态机/同步/冲突/撤销）
│  ├─ placeholder-dist/       # 历史占位，不参与构建
│  ├─ capabilities/default.json
│  └─ tauri.conf.json
└─ package.json
```

> `src/` 前端已落地（React 19 + Vite）。`tauri.conf.json` 的 `build.frontendDist` 指向 `../dist`，
> `devUrl` 为 `http://localhost:1420`，`beforeDevCommand` / `beforeBuildCommand` 分别为 `pnpm dev` / `pnpm build`。
> `src-tauri/placeholder-dist` 仅为历史占位，已不参与构建。

约定：Rust 模块 ≤ 400 行；React 单文件 ≤ 300 行；超过就拆。所有 IPC 类型在 `src/lib/types.ts` 与 Rust `model.rs` 双侧一一对应，命名一致（camelCase over IPC）。

## 7. 数据模型（SQLite）

```sql
PRAGMA journal_mode = WAL;
PRAGMA foreign_keys = ON;

CREATE TABLE targets (
  id           TEXT PRIMARY KEY,              -- ULID
  slug         TEXT NOT NULL UNIQUE,          -- claude-code / codex / cursor
  name         TEXT NOT NULL,
  kind         TEXT NOT NULL DEFAULT 'skills_dir', -- v2 预留: config_file
  skills_dir   TEXT NOT NULL UNIQUE,
  link_mode    TEXT NOT NULL DEFAULT 'symlink'
               CHECK (link_mode IN ('symlink', 'copy')),
  enabled      INTEGER NOT NULL DEFAULT 1,
  detected     INTEGER NOT NULL DEFAULT 0,    -- 路径是自动探测到的还是用户手填
  created_at   TEXT NOT NULL,                 -- ISO-8601 UTC
  updated_at   TEXT NOT NULL
);

CREATE TABLE skills (
  id            TEXT PRIMARY KEY,             -- ULID
  slug          TEXT NOT NULL UNIQUE,         -- kebab-case，同时是库目录名
  name          TEXT NOT NULL,                -- frontmatter name
  description   TEXT NOT NULL DEFAULT '',
  category      TEXT,                         -- SkillPort 自有分类
  source_kind   TEXT NOT NULL DEFAULT 'local' -- local | import | git
                CHECK (source_kind IN ('local', 'import', 'git')),
  source_ref    TEXT,
  content_hash  TEXT NOT NULL,                -- blake3(tree)
  created_at    TEXT NOT NULL,
  updated_at    TEXT NOT NULL
);

CREATE TABLE installs (
  id            INTEGER PRIMARY KEY AUTOINCREMENT,
  skill_id      TEXT NOT NULL REFERENCES skills(id) ON DELETE CASCADE,
  target_id     TEXT NOT NULL REFERENCES targets(id) ON DELETE CASCADE,
  mode          TEXT NOT NULL CHECK (mode IN ('symlink', 'copy')),
  actual_kind   TEXT,                         -- symlink | junction | copy
  status        TEXT NOT NULL DEFAULT 'pending'
                CHECK (status IN ('pending','ok','stale','missing','conflict','error')),
  target_path   TEXT NOT NULL,                -- 冗余存储，便于核对与排错
  linked_hash   TEXT,                         -- 建立链接/拷贝时的源 hash
  ignored       INTEGER NOT NULL DEFAULT 0,   -- 冲突的「保留并忽略」
  last_error    TEXT,
  last_checked  TEXT,
  last_synced   TEXT,
  UNIQUE (skill_id, target_id)
);

CREATE TABLE activity (
  id          INTEGER PRIMARY KEY AUTOINCREMENT,
  ts          TEXT NOT NULL,
  action      TEXT NOT NULL,                  -- import / link / copy / unlink / sync / reconcile
                                              -- / conflict / delete / update / target / restore
  skill_id    TEXT,
  skill_name  TEXT,                           -- 冗余显示名，技能删除后仍可读
  target_id   TEXT,
  target_name TEXT,
  result      TEXT NOT NULL,                  -- ok / failed
  detail      TEXT
);

CREATE TABLE undo (
  id       TEXT PRIMARY KEY,                  -- 关联的 activity id
  ts       TEXT NOT NULL,
  kind     TEXT NOT NULL,                     -- skill-delete | sync
  payload  TEXT NOT NULL                      -- JSON（技能行/Install 快照、备份路径）
);

CREATE TABLE settings (
  key   TEXT PRIMARY KEY,
  value TEXT NOT NULL                         -- JSON
);
```

- `installs` 存的是**期望**；`status` 是最近一次核对缓存，任何时候都可从文件系统重建。
- 一个 Skill 可以只分发给部分 Target；没有 Install 行 = 不打算分发，UI 显示「—」。
- 删除 Skill 时先删除所有关联链接（仅限本工具拥有的），再删库目录与 DB 行。
- 删除/同步前的备份与撤销记录配合使用：`undo` 保留 24 小时，撤销时先校验现场再恢复。
- 迁移只追加，不修改历史迁移；启动时自动执行。`installs.ignored`、`activity` 的显示名列、
  `undo` 表都来自追加的 v2 迁移（`SCHEMA_V2`）；`skills.category` 来自 v3（`SCHEMA_V3`）。

## 8. 分发语义

### 8.1 路径

目标路径 = `target.skills_dir / skill.slug`。**slug 是路径名，frontmatter 的 `name` 只是显示名**。重命名 slug 属于迁移操作：需要预览、且同时处理所有 Install。

slug 规则：`^[a-z0-9]+(-[a-z0-9]+)*$`，长度 2–63；保留名 `.`、`..`、`skillport`、`node_modules`。导入时从 frontmatter `name` 生成并清洗（无法生成时退回目录名）；**slug 冲突直接拒绝**（`DuplicateSlug`），不静默加后缀，需用户改名后重新导入。

**导入严格校验（v1 硬性）**

- 目标路径必须存在 `SKILL.md`；frontmatter 必须能被 YAML 解析；`name` 与 `description` 必须是非空字符串。
- 缺任一项 → 拒绝该技能导入。`ImportReport.items` 逐条返回 `{ path, slug, ok, errorCode, detail }`，错误码为
  `MissingSkillFile` / `InvalidFrontmatter` / `MissingName` / `MissingDescription` / `InvalidSlug` /
  `DuplicateSlug` / `PathNotFound` / `IoError`；失败项不写库、不建 Install、不碰目标目录。
- 不自动补 `name` / `description`；未知 frontmatter 字段保留，不因未知字段拒绝。
- frontmatter 可以包含嵌套结构（列表、映射）：校验只读顶层的 `name` / `description`，
  缩进行一律跳过；前端的轻量解析器必须和 serde_yaml 一样接受它们。
- **分类是 SkillPort 自己的归类口径，与技能文件无关**：存在 `skills.category` 里，左栏按它分组。
  导入时一律留空，也不读 frontmatter 里的 `category`（那不是 Skill 的约定字段）；在编辑对话框里改的是 DB，
  `SKILL.md` 一个字节都不动。任何「把分类读写到 frontmatter」的改动都属于越界。
- 批量导入按路径独立处理：失败项不影响成功项，报告逐条列出。

### 8.2 符号链接（默认）

- 创建：`std::os::unix::fs::symlink` / `std::os::windows::fs::symlink_dir`，目标为库目录的**绝对路径**。
- 校验：`symlink_metadata` 是 symlink，且 `read_link` 归一化后等于库目录。
- 删除：只删除校验通过的 symlink；悬空链接若指向库内不存在路径，也视为本工具遗留，可清理，但需在计划里明示。
- Windows 无权限时（`ERROR_PRIVILEGE_NOT_HELD`）：
  1. **不创建任何链接或目录**，记录 `last_error`；
  2. UI 说明原因（缺少符号链接权限，可开启开发者模式或以管理员身份运行），并询问是否改用 **copy 模式**；
  3. 用户确认 → 只把该条 Install 的 `mode` 改为 `copy`，按 8.3 创建，`actual_kind = copy`；不改 target 的默认 `link_mode`；
  4. 用户拒绝 → 保持 `error`，不执行任何文件操作，之后可从详情页重新选择 copy。
- v1 **不自动创建 junction**。`actual_kind` 中的 `junction` 仅保留用于兼容既有数据或未来显式选项，当前流程不产生。

### 8.3 拷贝

- 复制到临时目录（同卷，`<target>/.skillport-tmp-<ulid>`）→ 写入 sidecar `.skillport.json` → 原子 rename 到位。
- sidecar 内容：`{"schema":1,"managed_by":"skillport","skill_id":"...","source_hash":"..."}`。
- 过期判定：重新计算库目录 hash，与 `linked_hash` 比较；不一致 = `stale`。
- 覆盖前：只有 sidecar 存在且 `skill_id` 匹配，才允许替换；否则是冲突。
- 计算树 hash 时排除 `.skillport.json`、`.DS_Store`、`Thumbs.db`。

### 8.4 冲突与未托管

| 现场 | 判定 | 行为 |
|---|---|---|
| 目标路径不存在 | `missing`（有 Install） | 可同步 |
| symlink 指向本库 | `ok` | 无操作 |
| symlink 指向别处 | `conflict` | 不覆盖；提供「接管 / 保留并忽略 / 取消 Install」 |
| 普通目录 + 正确 sidecar | `ok` / `stale` | 可同步/覆盖 |
| 普通目录 + 无 sidecar | `conflict` | 不覆盖、不删除；可「接管为库副本」 |
| 目录里有多余条目、DB 无记录 | `untracked` | v1 的 IPC/UI 暂不列出（前端契约没有该字段）；工具**绝不自动删除**或改动 |
| 无权限 / IO 错误 | `error` | 记录 `last_error`，可重试 |

**接管（adopt）**：外部目录写入 sidecar 并标记为 `copy`；外部 symlink 先把链接目标复制为副本再替换，
原链接目标不被修改。接管后的内容若与库不同，状态为 `stale`，下一次同步会先备份再替换。

**永远不要**对非本工具拥有的路径执行 `remove_dir_all`。删除统一走 `sync::remove_distribution`
或 `undo` 的归属校验（sidecar / 链接指向库内），撤销恢复也先校验现场。

### 8.5 核对与同步

- 核对时机：应用启动、手动刷新、任何写操作之后、目标路径变更后。v1 不做后台文件监听（v0.3 用 `notify` 加，需防抖）。
- 同步必须先返回 **plan**（新增/修复/跳过/冲突），UI 展示后再 `apply(plan_id)`；plan 有 TTL（例如 5 分钟），过期需重新计算。
- 同步幂等；重复执行结果一致。
- 批量操作（全部同步）要显示进度事件 `sync://progress`，可取消；已完成的步骤不回滚。

## 9. 安全与可靠性

- **路径守卫**：任何目标写入前，canonicalize 父目录，确认 `starts_with(target.skills_dir)` 且目标名等于 skill.slug；拒绝 `..`、绝对路径 slug、UNC/驱动器路径注入。
- **不跟随恶意链接**：删除/覆盖前用 `symlink_metadata`，不跟随链接；拷贝时若源目录含指向外部的 symlink，默认拒绝或要求确认（避免把整个家目录拷进目标）。
- **备份**：覆盖非空目录前，把旧内容移入 `backups/<ts>-<slug>-<target>/`，保留最近 20 份。
- **权限**：Tauri capability 只开 `core:default` + `core:window:allow-start-dragging` + `dialog:default`（选目录）。
  自定义标题栏拖拽走 Tauri 注入的 `drag.js`，它 invoke 的 `plugin:window|start_dragging` 不在 `core:window:default`
  里，必须单独授权，否则 `data-tauri-drag-region` 完全无效。不启用 `shell`、不启用宽泛 `fs` scope。
- **日志**：记录路径、hash、错误码；不记录技能正文。日志轮转，单文件 ≤ 5 MB。
- **无静默破坏**：删除、覆盖、取消链接、重命名，全部需要确认；确认框显示具体路径。
- **崩溃安全**：先写临时文件/目录再 rename；DB 写事务；启动时清理 `.skillport-tmp-*` 残留。

## 10. 前端约定

### 10.1 数据流

- 所有 IPC 走 `src/lib/api.ts`，对前端暴露 typed promise；组件不直接 `invoke`。
- 读数据用 TanStack Query，query key 形如 `['skills']`、`['targets']`、`['reconcile']`；写操作后精确失效。
- 全局 UI 状态（当前选中、面板宽度、筛选、命令面板）放 Zustand；不复制服务端数据进 store。
- 浏览器里 `pnpm dev` 时自动切到 `mockApi`（内存数据），方便调 UI；不得让 mock 泄漏进生产包（`import.meta.env.DEV` 判断）。

### 10.2 交互要求

- 首屏即主界面，不做启动引导页；首次启动用空状态 +「导入技能 / 添加目标」两个动作。
- 键盘：`⌘/Ctrl+K` 命令面板，`Esc` 关闭浮层，`↑/↓` 仅在命令面板内移动结果（技能列表不做键盘选择），`Enter` 打开。
  核对只在启动、写操作后和目标路径变更后自动执行，不绑定快捷键。
- 危险操作按钮不做默认焦点；冲突面板默认选中「保留现状」。
- 所有异步操作有 loading / empty / error 三态；错误显示可操作建议，不只显示堆栈。
- 删除、同步、冲突解决都要有撤销入口（基于备份，撤销期内可恢复）。
- 目标页的技能目录是链接：整格可点，在系统文件管理器中打开该目录；目录不存在时提示错误，不创建。

### 10.3 视觉系统

设计方向确定前，先实现为 CSS 变量 token，不写死颜色：

```css
@theme {
  --color-bg: ...;
  --color-surface: ...;
  --color-ink: ...;
  --color-muted: ...;
  --color-line: ...;
  --color-accent: ...;
  --color-ok: ...;
  --color-warn: ...;
  --color-danger: ...;
  --radius-panel: ...;
  --font-sans: ...;
  --font-mono: ...;
}
```

- 正文 ≥ 14px，标签 ≥ 12px；正文对比度 ≥ 4.5:1。
- 可点击区域 ≥ 32×32（桌面端），主要按钮 ≥ 36 高。
- 路径、hash、slug 用 mono 字体；状态色不能是唯一信息载体，必须同时有文字/图标。
- harness 图标**外置**在 `<data_dir>/icons/<slug>.svg`，启动时用 `src/assets/targets/` 的内置图标补齐缺失项，
  已存在的文件永远不覆盖（用 `create_new` 打开，悬空链接也不会被跟随）。只读取普通文件、合法 slug、
  且内容确实是 SVG 文档的 `.svg`；其余条目静默跳过。前端通过 `target_icon_list` 取 SVG 原文，由
  `features/targets/TargetIcon` 用 CSS mask 绘制：SVG 根节点的 `fill` 是十六进制就用作品牌色
  （Claude / Gemini / DeepSeek），`currentColor` 则跟随主题；未知 slug 回退到通用图标。
  新增图标 = 往图标目录放一个 `<slug>.svg`，不需要重新打包。图标查询用
  `refetchOnWindowFocus: 'always'`，窗口重新获得焦点时必然重读，不受 staleTime 限制。
- 主题三态：浅色 / 深色 / 跟随系统，从标题行的主题菜单选择并持久化；跟随系统时监听
  `prefers-color-scheme` 实时切换。切换用 View Transitions 做整页交叉淡入（`app/theme.ts`），
  不支持该 API 或 `prefers-reduced-motion` 时直接换色。
- 桌面应用手感：界面文字默认不可选中（`body { user-select: none }`），输入框、CodeMirror、
  `pre`/`code` 与 `.select-text` 例外。
- 视觉方向已定（见 §18），改视觉时以 `src/styles/tokens.css` 的 token 为准，直接在组件里落地；
  不再维护独立的设计稿目录，新的视觉决策直接补进本文件 §18。

### 10.4 窗口 chrome

- macOS 用 `titleBarStyle: "Overlay"` + `hiddenTitle`：交通灯直接叠在应用自己的标题行上（与 VS Code / Claude 相同），
  不额外占一行。
- 主窗口在 `tauri.conf.json` 里是 `create: false`，由 `lib.rs::create_main_window` 用
  `WebviewWindowBuilder::from_config` 创建——只有 builder 才有 `traffic_light_position`。
  当前 `(x = 16, y = 24)`：AppKit 按钮 14px 高、按钮上沿 = `y - 9`，即 44px 标题行正中。
- 标题行高 44px（`h-11`），左侧留 88px 给交通灯（`hasOverlayTitleBar`）；抽屉/遮罩的 `top-11` 必须同步改。
- 标题行是 `data-tauri-drag-region="deep"`：整棵子树可拖，按钮和链接自动排除；裸值只认事件目标本身，
  文字节点会让拖拽失效。
- 标题行不显示品牌名与 logo；左上角是侧边栏显隐按钮（`uiStore.sidebarOpen`，持久化）。侧边栏用
  `width` 过渡 200ms 收起/展开，内层固定 244px 以免收起过程中内容重排；收起时 `inert` + `aria-hidden`。
  `<900px` 时侧边栏与按钮一起隐藏（改由标题行中的「全部技能」导航）。
- Windows 保留系统标题栏；抽屉/遮罩的 `top` 偏移必须与标题行高度一致（当前 `top-11`）。

### 10.5 文案

- 默认 zh-CN，英文兜底；所有 UI 文案进 `src/i18n/zh-CN.ts`，不散落在 JSX。
- 用动词开头、说清楚对象：「同步到 Claude Code」「在 Finder 中显示」。
- 不把内部术语直接抛给用户：`stale` → 「副本过期」，`untracked` → 「未托管」。

## 11. Rust 约定

- edition 2021+；`cargo fmt`、`cargo clippy -- -D warnings` 必须通过。
- 命令层只做参数校验和类型转换；业务逻辑在 `crates/skillport-core`，便于单元测试，CLI 也复用它。
- 错误统一 `AppError`（`thiserror`），序列化为 `{ code, message, detail? }`；前端按 `code` 分支，不匹配 message 文本。
- 不使用 `unwrap()` / `expect()` 处理用户输入、路径、IO；测试代码除外。
- 所有文件操作显式处理 Windows / Unix 分支，`#[cfg(windows)]` 隔离平台代码。
- 时间统一 UTC ISO-8601；显示时前端本地化。
- 并发：单个 `Mutex<Connection>` 足够；文件操作在 `spawn_blocking` 中执行，避免阻塞 IPC 线程。
- 每个公开命令有 doc comment，说明副作用与幂等性。

### IPC 表面（v1）

```text
library_list()                        -> SkillSummary[]        # sizeBytes 扫描填充；category 来自 DB
library_get(id)                       -> SkillDetail { ...summary, path, skillMd }
library_import(paths)                 -> ImportReport { items, imported, skipped }
library_update(id, { skillMd?, name?, description?, category? }) -> SkillDetail
                                      # category 只写 DB（空串清除）；不碰 SKILL.md
library_delete(id, { deleteTargets }) -> { undoId: string | null, removedInstalls: number }
library_reveal(id)                    -> ()

targets_list()                        -> Target[]
target_detect_suggestions()           -> SuggestedTarget[]
target_create(input { slug, name, skillsDir, linkMode, enabled }) -> Target
target_update(id, patch)              -> Target
target_delete(id, { unlink })         -> ()
target_reveal(id)                     -> (); 在文件管理器中打开目标的技能目录（不存在则 NotFound）
target_pick_dir(initialDir?)          -> string | null        # dialog
target_icon_list()                    -> [{ slug, svg }]      # 读 <data_dir>/icons/*.svg
target_icon_reveal()                  -> (); 在文件管理器中打开图标目录

installs_set(skillId, targetId, { enabled, mode? }) -> Install | null
installs_resolve(skillId, targetId, adopt|ignore|unignore|cancel) -> Install | null
reconcile()                           -> ReconcileReport { checked, changed, issues, checkedAt }
sync_plan(filter { skillId?, targetId? }?) -> SyncPlan { id, createdAt, expiresAt, actions }
sync_apply(planId)                    -> SyncResult { planId, applied, failed, cancelled, undoId, items }
sync_cancel(planId)                   -> ()

activity_list({ limit?, cursor? })    -> ActivityPage { items, nextCursor: string | null }
activity_undo(activityId)             -> UndoResult { ok, message }
settings_get() / settings_set(patch)  -> AppSettings { defaultLinkMode, libraryPath, dataDir, backupRetention }
```

- `library_import` 逐项返回 `{ path, slug, ok, errorCode, detail }`；严格校验失败不写库、不建 Install。
- `SyncAction.kind` 只有 `create | repair | skip | conflict`；只有前两者会执行并计入 `applied/failed`。
- `settings_set` 只接受 `defaultLinkMode` / `backupRetention`；`libraryPath` / `dataDir` 为只读，改动会被拒绝。
  当前前端没有设置页：`settings_get` 只用于读 `libraryPath`（表格的来源列与路径守卫），
  外观/语言在标题行切换，分发方式由每个目标自己在创建时决定，因此 `settings_set` 暂无调用方。
- 事件：`sync://progress`（`{ planId, completed, total, actionId, status }`）、
  `sync://finished`、`library://changed`、`reconcile://finished`（携带 `ReconcileReport`）。

### 11.1 CLI（`skillport-cli`，供其他 agent 调用）

入口在 `src-tauri/crates/skillport-cli`，只依赖 `skillport-core`，不链接 Tauri，也不需要先构建前端。
它与桌面端读写同一个数据目录，导入走同一套严格校验、slug 冲突判定与路径守卫。

```bash
skillport-cli import <PATH>... [--data-dir <DIR>] [--json]
```

- `<PATH>` 是技能目录：目录本身含 `SKILL.md` 就导入它，否则导入其下所有含 `SKILL.md` 的直接子目录。
- 数据目录优先级：`--data-dir` > `$SKILLPORT_DATA_DIR` > 平台默认。平台默认按 Tauri `app_data_dir()` 的规则解析
  （macOS `~/Library/Application Support/com.skillport.desktop`、Windows `%APPDATA%\com.skillport.desktop`、
  Linux `$XDG_DATA_HOME` 或 `~/.local/share` 下的同名目录），标识符常量在 `skillport_core::paths::APP_IDENTIFIER`。
- `--json` 输出稳定结构：`{ schema, ok, dataDir, libraryDir, imported, skipped, items[] }`，
  `items[].errorCode` 沿用 §8.1 的错误码；致命错误输出 `{ schema, ok: false, error: { code, message, detail } }`。
- 退出码：`0` 全部成功；`1` 有路径被拒绝（校验失败 / slug 冲突）；`2` 参数错误、数据目录或数据库打不开。
- 导入只写库与 DB，不创建分发；分发仍由桌面端的同步流程负责（应用启动或手动刷新时会核对）。
- 不新增依赖：参数解析是手写的，不为单个子命令引入 clap。

构建与安装：

```bash
cargo build --release -p skillport-cli --manifest-path src-tauri/Cargo.toml
cargo install --path src-tauri/crates/skillport-cli
```

发布时（§15）由 release workflow 产出 macOS Universal `tar.gz` 与 Windows x86_64 `zip`，
不需要另起构建任务。

## 12. 测试与验收

### 单元测试（Rust，必须）

- slug 校验与清洗；frontmatter 解析；树 hash 的稳定性（文件顺序无关）。
- 导入严格校验：缺 `SKILL.md`、缺 frontmatter、缺 `name`、缺 `description`、YAML 解析失败，都必须拒绝且有测试。
- 路径守卫：拒绝 `../x`、绝对路径、符号链接逃逸。
- 状态机：missing / ok / stale / conflict / error 的判定矩阵；忽略标记不计入问题数。
- 同步：首次链接、重复链接幂等、取消链接只删自己拥有的、copy 模式覆盖与备份。
- 冲突处理：接管目录 / 接管外部链接 / 忽略 / 取消忽略 / 取消 Install；外部内容不被删除。
- 撤销：删除技能后恢复库与 Install；同步创建的链接被移除；被替换的副本从备份恢复。
- Windows 分支：symlink 无权限 → 提示原因 → 用户确认 copy / 拒绝不操作；不自动 junction（CI 跑 Windows）。
- CLI 端到端（`src-tauri/crates/skillport-cli/tests/import.rs`）：启动真实二进制，用临时数据目录验证成功导入、
  严格校验拒绝（退出码 1、JSON `errorCode`）、父目录批量导入与参数错误（退出码 2）。

### 前端测试

- Vitest：`status.ts` 映射、筛选/排序、同步计划展示逻辑、图标查询在窗口重新聚焦时刷新。
- Testing Library：关键面板的空态/错误态/冲突态。
- 手工验收（每次发版）：macOS + Windows 各跑一遍真实目录（用临时 HOME 或测试账号）。

### 验收标准（Definition of Done）

1. 在 macOS 与 Windows 上都能完成：导入技能 → 添加到 3 个目标 → 核对 → 修改库文件 → 副本目标显示过期 → 重新同步。
2. 目标目录里预先放一个同名外部目录，工具必须标冲突且不修改它。
3. 断网、只读目录、无权限、路径不存在四种异常都有可理解的提示。
4. `pnpm typecheck`、`pnpm lint`、`pnpm test`、`cargo test`、`cargo clippy` 全绿。
5. 卸载/删除技能后，目标目录无残留（本工具创建的条目全部清理）。

## 13. 后续：MCP（v2 预留）

MCP 与 Skill 的分发机制不同：MCP 是写进 harness 的配置文件（JSON/TOML/YAML），不是目录链接。v2 设计要点：

- `targets.kind` 扩展为 `config_file`，新增 `config_path`、`config_format`。
- 写配置必须是 **结构化 merge**：读取 → 修改指定 key → 校验 → 备份 → 原子写回；保留用户其他配置与注释（TOML 注释需用保留式解析器，不能简单 parse+serialize）。
- 每个 MCP server 一条记录（command/args/env/url），支持「启用 / 停用」，不删除用户手写条目。
- 密钥（env、token）v1 不入库或只存 keychain 引用（`tauri-plugin-keyring`），绝不写日志。

v1 的 schema / UI 不要为 MCP 提前加半成品字段，除 `targets.kind` 外。

## 14. 开发命令

```bash
pnpm install
pnpm dev                 # 浏览器 + mock 数据，调 UI
pnpm tauri dev           # 真桌面应用
pnpm typecheck
pnpm lint
pnpm test
pnpm tauri build         # 产出安装包
cargo test   --manifest-path src-tauri/Cargo.toml --workspace
cargo clippy --manifest-path src-tauri/Cargo.toml --workspace --all-targets -- -D warnings
cargo fmt    --manifest-path src-tauri/Cargo.toml --all --check
cargo build  -p skillport-cli --manifest-path src-tauri/Cargo.toml   # CLI 二进制
```

环境：Node ≥ 22、pnpm ≥ 10、Rust stable ≥ 1.80、Xcode CLT（macOS）、WebView2 Runtime（Windows 11 自带；Windows 10 可能需要 bootstrapper，Tauri 打包时会处理）。

> 浏览器里 `pnpm dev` 使用内存 mock，不读写真实 HOME，适合调 UI；真桌面应用请用 `pnpm tauri dev`。
> 手动跑 Rust 端或桌面应用时，仍建议用临时数据目录，避免写真实 HOME：
>
> ```bash
> SKILLPORT_DATA_DIR="$(mktemp -d)/skillport" cargo run --manifest-path src-tauri/Cargo.toml
> ```
>
> CLI 同样用临时数据目录试跑，避免写真实 HOME：
>
> ```bash
> SKILLPORT_DATA_DIR="$(mktemp -d)/skillport" \
>   cargo run -p skillport-cli --manifest-path src-tauri/Cargo.toml -- import ./some-skill --json
> ```

## 15. 打包与发布

- macOS：`universal-apple-darwin`，Developer ID 签名 + 公证 + staple；**不开启 App Sandbox**（会阻止写入其他 harness 目录）。若未来上架 MAS，需要改用 security-scoped bookmarks，属于独立设计。
- Windows：x64 优先，NSIS 安装包 + MSI；代码签名尚未接入 workflow（需要证书与 signtool / Azure Trusted Signing），未签名包会触发 SmartScreen。
- 自动更新用 `tauri-plugin-updater`，签名密钥离线保管；v1 可先手动更新。
- 版本号 SemVer；数据库迁移必须向后兼容至少一个 minor。
- CI 在 `.github/workflows/ci.yml`：前端 typecheck / lint / test / build，加 macOS 与 Windows 的 fmt / clippy / cargo test。
  `cargo` 构建前必须先 `pnpm build`，因为 `tauri::generate_context!()` 在编译期嵌入 `frontendDist`。
- 发布在 `.github/workflows/release.yml`：push `v*` tag 触发，macOS 出 Universal `.dmg`、Windows 出 `.msi` + NSIS `.exe`，
  产物进草稿 Release。tag 必须与 `src-tauri/tauri.conf.json` 的 `version` 一致；签名/公证 secrets 见 workflow 顶部注释。
- 同一个 workflow 还构建 `skillport-cli` 并挂到同一个草稿 Release：macOS 为 Universal `tar.gz`（两个架构 lipo 后
  重签：有 Developer ID 就用它，否则 ad-hoc），Windows 为 x86_64 `zip`。CLI 不随 `.app` 打包，也不参与公证；
  分发时的签名状态写在 releaseBody 里。
- 依赖更新由 `.github/dependabot.yml` 每周提 PR（npm、cargo、github-actions）；PR 模板在
  `.github/pull_request_template.md`。

## 16. Git 与协作

- Conventional Commits：`feat:`, `fix:`, `refactor:`, `docs:`, `test:`, `chore:`。
- 分支：`feat/<scope>`、`fix/<scope>`；PR 描述写清「改了什么 / 怎么验证 / 风险」。
- 不提交：`node_modules`、`target/`、`*.sqlite3`、设计稿截图等大文件、真实用户路径/密钥。
- 提交前跑 `pnpm typecheck && pnpm lint && pnpm test`。
- CR（Code Review）按 `CODING_STANDARDS.md` 第 5 节执行；编码规范细则见该文件。

## 17. 给 agent 的工作规则

1. 先读本文件和相关模块，再动手；不确定的需求先问，不要猜着做破坏性操作。
2. 改动要小步、可验证；先写/改测试，再实现（核心同步逻辑必须 TDD）。
3. 任何涉及真实用户目录的脚本，先 dry-run 并打印将执行的操作；默认使用临时目录。
4. 不新增依赖除非确有必要；新增前说明理由、体积、维护状态。
5. 不为了「看起来完整」造 UI：没有后端能力的按钮要么禁用并说明，要么不做。
6. 设计稿目录已清理，视觉以已确认的方向（§18）和现有 token / 组件为准，不再另起原型目录。
7. 完成后更新本文件中的命令、结构、状态机变化；过时文档比没有文档更糟。

## 18. 待确认

- [x] 产品名：**SkillPort**（2026-11-06 确认）。
- [ ] 品牌标识（logo / 应用图标 / 安装包图标）待设计。
- [ ] v1 首批内置目标与默认路径（需逐个 harness 版本核实，不要凭记忆写死）。
- [ ] 是否需要「项目级技能」作为 v1.5。
- [x] Windows symlink 失败：提示原因并询问是否改用 copy；用户拒绝则不操作；v1 不自动 junction。
- [x] 导入严格校验：缺 `SKILL.md`、frontmatter、`name` 或 `description` 一律拒绝，并报告原因。
