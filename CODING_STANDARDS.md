# CODING_STANDARDS.md · SkillPort 工程规范

> 面向所有改 SkillPort 的人与 agent。开始动手前先读 `AGENTS.md`，再读本文件。
> 本文件把社区规范翻译成这个项目能直接执行的规则；与 `AGENTS.md` 冲突时，以 `AGENTS.md` 为准。
>
> 标记：**MUST** = 阻塞合并；**SHOULD** = 默认遵守，偏离需在 PR 里说明理由；**MAY** = 可选。
> 社区规范是依据，不是教条；项目约束（单一真相源、不静默破坏、路径守卫）优先。
>
> 最后更新：2026-11-06 · 来源见第 8 节。

## 摘要：社区里反复出现的五条

1. **风格自动化，CR 留给设计与风险**：rustfmt / Clippy / ESLint 管格式，人管正确性、边界与可维护性。
2. **小步提交、测试同 PR、重构分离**：小改动更容易审、更容易回滚，也更少引入 bug。
3. **边界要类型化，错误要显式**：Rust 用类型和 `Result`，TS 用 strict + 联合类型，Tauri IPC 用结构化错误。
4. **最小权限与不可信前端**：Tauri 的 Rust 核心权限大，WebView 是不可信一侧；capabilities、CSP、输入校验缺一不可。
5. **状态能派生就派生**：React 里 effect 只用于外部同步；服务端/IPC 状态与纯 UI 状态分开管理。

日常改动先读：第 0 节、2.7（导入与文件系统安全）、3.5（状态）、4.2（capabilities）、5.7（本项目 CR 清单）。新贡献者建议通读。

## 0. 十条不可妥协

1. **库是唯一真相源**。权威副本只在 `library/`；目标目录里只放链接或带 sidecar 的副本。
2. **不对非本工具拥有的路径执行删除或覆盖**。删除只走 `sync::delete_owned_*` 守卫。
3. **写目标前必须过路径守卫**：canonicalize 父目录、`starts_with(skills_dir)`、目录名等于 slug；拒绝 `..`、绝对路径、UNC/驱动器注入。
4. **冲突与未托管不静默处理**。同名外部目录标 `conflict`，只展示、不自动改；未托管条目可手动接管。
5. **写操作先出 plan 再 apply**。plan 有 TTL、可取消、有进度；同步幂等，已完成步骤不回滚。
6. **覆盖前备份**，拷贝用「临时目录 + sidecar + 原子 rename」；启动时清理 `.skillport-tmp-*`。
7. **Rust 不用 `unwrap()` / `expect()` 处理用户输入、路径、IO**；错误统一 `AppError { code, message, detail? }`，前端按 `code` 分支。
8. **前端不直接碰 FS、不直接 `invoke`**。所有 IPC 走 `src/lib/api.ts`；IPC 类型在 `src/lib/types.ts` 与 Rust `model.rs` 双侧一一对应。
9. **Tauri capability 最小化**：只开 `core:default` + `dialog:default`；收紧 CSP；不加载远程内容；敏感命令额外做权限/参数校验。
10. **小 PR、测试同 PR、CR 看每一行**。重构与功能分开；提交信息用 Conventional Commits；合并前跑完自动化 gates。

---

## 1. 通用工程原则

### 1.1 改动方式

- **MUST** 小步、单一目的、可回滚。一个 PR 只解决一件事；重构、格式化、功能、修 bug 不混在一起。
- **MUST** 先读相关模块、测试、`AGENTS.md`，再动手；不确定的破坏性需求先问，不猜着做。
- **SHOULD** 先写/改测试，再实现；核心同步逻辑（plan/apply、状态机、路径守卫）按 TDD 做。
- **SHOULD** 任何涉及真实用户目录的脚本先 dry-run，打印将执行的操作；测试/演示一律用临时目录。
- **SHOULD** 不为「看起来完整」造 UI：没有后端能力的按钮要么禁用并说明，要么不做。
- **MAY** 顺手修明显的小问题（拼写、局部命名），但不要夹带无关重构。

### 1.2 依赖与供应链

- **MUST** 新增依赖前说明理由、体积、维护状态、许可证；能用标准库/现有依赖就不加。
- **SHOULD** 提交 lockfile；CI 跑 `cargo audit`（Rust）与 `pnpm audit`（前端），高危漏洞阻塞发布。
- **SHOULD** 第三方 GitHub Actions 固定到 commit SHA，至少固定到主版本 tag。
- **MAY** 对关键依赖使用 `cargo-vet` / `cargo-crev` 做人工审计；v1 不强制。

### 1.3 可观测性与隐私

- **MUST** 日志记录路径、hash、错误码；**不记录技能正文、token、密钥、用户隐私内容**。
- **MUST** 错误信息可操作：告诉用户发生了什么、可以做什么，不只给堆栈。
- **SHOULD** 结构化日志字段稳定，便于按 `code`、`skill_id`、`target_id` 检索。
- **MUST** 日志轮转，单文件 ≤ 5 MB（见 `AGENTS.md`）。

### 1.4 完成定义（DoD）

一个改动算完成，至少满足：

- [ ] 行为有测试覆盖（Rust 单测/集成测试；前端 Vitest/RTL）。
- [ ] `pnpm typecheck`、`pnpm lint`、`pnpm test` 通过。
- [ ] `cargo fmt --check`、`cargo clippy -- -D warnings`、`cargo test` 通过。
- [ ] 涉及 UI 的改动有 loading / empty / error 三态，键盘可操作，深浅色都可读。
- [ ] 涉及路径/删除/覆盖/迁移的改动，有对应的安全测试与回滚说明。
- [ ] `AGENTS.md` / 本文件中过时的命令、结构、状态机描述已更新。

---

## 2. Rust 规范

### 2.1 工具链与 CI gates

- **MUST** 使用 stable 工具链，edition 2021+；不使用 nightly-only 特性。
- **MUST** `cargo fmt --check` 通过；格式争议交给 rustfmt，不在 CR 里讨论空格。
- **MUST** `cargo clippy -- -D warnings` 通过。Clippy 默认组是 `clippy::all`；需要更严时按需启用 `clippy::pedantic`，不要整组照单全收。
- **SHOULD** 从 `clippy::restriction` 里挑 `unwrap_used`、`expect_used`、`panic` 等与本项目安全目标一致的 lint，用 `#[allow(...)]` 逐处豁免并写理由。
- **SHOULD** 在 `Cargo.toml` 写 `rust-version`（MSRV），与 `AGENTS.md` 的 Rust stable ≥ 1.80 对齐。
- **MAY** 用 Cargo 1.97+ 的 `[build] warnings = "deny"` 或 `CARGO_BUILD_WARNINGS=deny`；MSRV 较低时继续用 `cargo clippy -- -D warnings`。

```bash
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml
```

### 2.2 命名与 API 设计（Rust API Guidelines）

- **MUST** 遵循 RFC 430 命名：类型/trait `UpperCamelCase`，函数/模块/变量 `snake_case`，常量 `SCREAMING_SNAKE_CASE`。
- **MUST** 转换方法遵循 `as_` / `to_` / `into_` 约定：`as_` 是廉价借用转换，`to_` 产生新值，`into_` 消费 `self`。
- **MUST** 集合迭代方法用 `iter` / `iter_mut` / `into_iter`，不要发明 `get_all` / `items` 之类的同义词。
- **SHOULD** 用 newtype 表达领域 ID 和单位，避免 `String` / `bool` 满天飞：`SkillId(String)`、`TargetId(String)`、`LinkMode`。
- **SHOULD** 参数用 `&str`、`&Path`、`impl AsRef<Path>`，把「调用者决定拷贝」留给调用者；需要存储时再 `to_owned()`。
- **SHOULD** 所有公开类型实现 `Debug`；用户可见的错误实现 `Display`；错误链保留 `source()`。
- **SHOULD** 公开结构体字段默认私有；构造器负责校验不变量；复杂构造用 builder。
- **SHOULD** 对可能扩展的公开 enum/struct 加 `#[non_exhaustive]`；本项目内部类型不必机械添加。
- **MUST** 公开命令/核心 API 写 doc comment，说明副作用、幂等性、错误条件；`unsafe` 额外写 `# Safety`。

### 2.3 错误处理

- **MUST** 核心层用 `thiserror` 定义 `AppError`；命令层把 `AppError` 序列化为 `{ code, message, detail? }`。
- **MUST** 不用 `unwrap()` / `expect()` 处理用户输入、路径、IO、数据库；测试代码和启动期不可恢复的 fail-fast 除外，且要给出可操作信息。
- **MUST** 不用字符串当错误码；前端按 `code` 分支，不匹配 message 文本。
- **SHOULD** 在错误转换时保留上下文（哪个路径、哪个 skill/target、哪个操作），不要把底层错误吞成「失败」。
- **SHOULD** 业务错误用 enum 表达（`PathEscape`、`Conflict`、`PermissionDenied`、`NotFound`），IO 错误用 `#[from]` 包装。
- **MUST** 不让 panic 穿过 IPC 边界；命令层捕获/转换，返回 `Result`。
- **SHOULD** 错误文案面向用户，detail 面向排错；不要把堆栈直接塞进 UI。

```rust
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("路径越界：{0}")]
    PathEscape(String),
    #[error("目标路径已存在且不属于 SkillPort：{0}")]
    Conflict(String),
    #[error("IO 错误：{0}")]
    Io(#[from] std::io::Error),
}
```

### 2.4 所有权、借用与并发

- **SHOULD** 参数优先借用，返回需要所有权时再移动；避免无意义的 `.clone()`，但不要为了零拷贝把代码写得不可读。
- **SHOULD** 用迭代器链表达转换/过滤；复杂循环可以保留，以可读性为准。
- **MUST** 共享状态用 `Mutex<Connection>` / `Mutex<AppState>`；Tauri `State` 已经处理了 `Arc`，不要再套一层 `Arc`。
- **MUST** 不把 `MutexGuard` 跨 `await` 持有；需要跨 await 时改用 Tokio `Mutex` 或缩小临界区。
- **MUST** 文件系统、SQLite 等阻塞操作放到 `spawn_blocking`，不在 async 命令里阻塞 IPC 线程。
- **SHOULD** 锁中毒（poisoned）要转换为 `AppError`，不要 `unwrap()`；锁的粒度尽量小。

### 2.5 模块、可见性与文件大小

- **MUST** Rust 单模块 ≤ 400 行；超过就按职责拆（`commands/` 薄封装，`crates/skillport-core/` 业务）。
- **SHOULD** 默认私有，按需 `pub(crate)`；不要用 `pub use *` 把内部结构泄漏成公开 API。
- **SHOULD** 一个模块一个主题：`paths` 只做路径与守卫，`sync` 只做 plan/apply/reconcile，`db` 只做连接/迁移/查询。
- **MUST** 命令层只做参数校验、状态注入、调用 core、错误转换；业务逻辑不写在 `#[tauri::command]` 里。

### 2.6 unsafe

- **SHOULD** 能不用就不用；标准库/成熟 crate 能解决就不自己写。
- **MUST** 必须用时，最小化 `unsafe` 块，写 `// SAFETY:` 说明前置/后置条件，封装在安全抽象里，并补测试。
- **MUST** 启用 `unsafe_op_in_unsafe_fn`（edition 2024 默认；2021 用 lint），不在 `unsafe fn` 里默认放开所有操作。
- **MUST** 不在 `unsafe` 里做「顺便优化」；性能理由要有测量。

### 2.7 导入校验与文件系统安全（本项目硬约束）

- **MUST** 导入严格校验：必须存在 `SKILL.md`，frontmatter 可解析，`name` 与 `description` 均为非空字符串；缺任一项 → **拒绝导入**，报告路径与缺失字段，不写库、不建链接。
- **MUST** 不自动补 `name`/`description`；未知 frontmatter 字段保留，不因未知字段拒绝（YAML 解析失败除外）。
- **MUST** 批量导入按路径独立报告成功/失败；失败项不产生库目录或 Install 行。
- **MUST** 任何目标写入前：
  1. 校验 slug 匹配 `^[a-z0-9]+(-[a-z0-9]+)*$`，长度 2–63，拒绝保留名；
  2. canonicalize 父目录；
  3. 确认父目录 `starts_with(target.skills_dir)`；
  4. 确认目标目录名等于 `skill.slug`；
  5. 拒绝 `..`、绝对路径、UNC、Windows 驱动器注入。
- **MUST** 删除/覆盖前用 `symlink_metadata`，不跟随链接；只删除校验通过的、本工具拥有的 symlink/junction/copy。
- **MUST** 不对非本工具拥有的路径执行 `remove_dir_all`；所有删除走 `delete_owned_*` 守卫函数。
- **MUST** 拷贝流程：同卷临时目录 `<target>/.skillport-tmp-<ulid>` → 写 sidecar `.skillport.json` → 原子 rename 到位。
- **MUST** 覆盖非空目录前备份到 `backups/<ts>-<slug>-<target>/`，保留最近 20 份。
- **MUST** 树 hash 排除 `.skillport.json`、`.DS_Store`、`Thumbs.db`，文件顺序不影响结果。
- **MUST** 源目录含指向外部的 symlink 时默认拒绝或要求确认，避免把家目录拷进目标。
- **MUST** Windows：symlink 失败（如 `ERROR_PRIVILEGE_NOT_HELD`）时，不创建任何链接/目录；说明原因并询问是否改用 copy 模式。用户确认 → 以 copy 创建，并把该 install 的 mode 改为 copy；用户拒绝 → 保持 `error`，不做任何文件操作。v1 不自动创建 junction。
- **MUST** 启动时清理 `.skillport-tmp-*` 残留；清理只针对本工具命名模式。

### 2.8 数据库

- **MUST** 启动时 `PRAGMA journal_mode = WAL`、`PRAGMA foreign_keys = ON`。
- **MUST** 迁移只追加，不修改历史迁移；启动时自动执行。
- **MUST** 多步写入用事务；遵循 `installs` 存期望、`status` 是可重建缓存的模型。
- **MUST** 时间统一 UTC ISO-8601；显示时前端本地化。
- **SHOULD** 查询按需取列，不 `SELECT *` 到 UI；大结果分页（activity）。
- **SHOULD** 删除 Skill 时先删除本工具拥有的链接，再删库目录与 DB 行；顺序不可反。

### 2.9 测试

- **MUST** slug 校验/清洗、frontmatter 解析、导入严格校验（缺文件/字段/YAML 错误）、树 hash 稳定性、路径守卫、状态机矩阵、同步幂等、取消链接只删自己拥有的、copy 覆盖与备份，都有单测。
- **MUST** 文件系统测试用临时目录；不读写真实 `~/.claude/skills`、`~/.codex` 等。
- **MUST** 每个 bug fix 带回归测试；修完先让测试红过，再让它绿。
- **SHOULD** 表驱动覆盖状态矩阵（missing / ok / stale / conflict / untracked / error）。
- **SHOULD** Windows 分支在 CI 跑；symlink 失败 → 提示原因 → 用户确认 copy / 拒绝不操作，这条路径要有测试或可验证的 mock。
- **SHOULD** 测试名描述行为（`copy_install_is_stale_after_library_change`），不要 `test_1`。

### 2.10 Tauri 命令层

- **MUST** 命令签名用 owned 类型或 `State<'_, T>`；返回 `Result<T, AppError>`；参数/返回值实现 `serde`。
- **SHOULD** IPC 字段统一 camelCase（`#[serde(rename_all = "camelCase")]`），与 `src/lib/types.ts` 一致。
- **SHOULD** 重活声明 `async` 并 `spawn_blocking`；流式进度用 `tauri::ipc::Channel`，不要用轮询。
- **MUST** 事件只做通知（`sync://progress`、`sync://finished`），不靠事件回传关键结果；事件不类型安全、总是异步、只支持 JSON。
- **MUST** 每个公开命令写 doc comment：副作用、是否幂等、可能返回的错误码。

---

## 3. React + TypeScript 规范

### 3.1 TypeScript

- **MUST** `strict: true`；不用隐式 `any`；不用 `// @ts-ignore`（必要时 `@ts-expect-error` 并写原因）。
- **MUST** 禁止显式 `any`（`@typescript-eslint/no-explicit-any`）；边界数据用 `unknown` + 收窄，或用泛型/联合类型。
- **SHOULD** 用可辨识联合表达状态，不用一堆互相矛盾的布尔值：

```ts
type SyncState =
  | { status: "idle" }
  | { status: "planning" }
  | { status: "ready"; plan: SyncPlan }
  | { status: "running"; progress: number }
  | { status: "error"; error: AppError };
```

- **SHOULD** 开启 `noUncheckedIndexedAccess`、`noImplicitOverride`、`noFallthroughCasesInSwitch`；用 `verbatimModuleSyntax` + `import type`。
- **SHOULD** 用 `satisfies` 保留字面量类型，不用双重 `as` 断言；`as` 只用于确有把握的边界。
- **MUST** IPC 类型在 `src/lib/types.ts` 与 Rust `model.rs` 一一对应，命名一致（camelCase over IPC）。
- **MUST** 运行时不可信的数据在 Rust 侧校验；前端不假设「类型写了就安全」。

### 3.2 组件

- **MUST** 函数组件 + Hooks；不新增 class 组件。
- **MUST** 单文件 ≤ 300 行；超过就拆组件/抽 hook。
- **SHOULD** 一个文件一个主组件；子组件可以同文件，但超过 2 个就考虑拆目录。
- **SHOULD** props 用对象类型（`type` 或 `interface`），不滥用 `React.FC`；直接标注参数类型即可。
- **SHOULD** 组合优先于继承；用 `children` / slot / render prop 解决复用，不写组件继承层级。
- **SHOULD** UI primitives（`components/ui`）保持无业务；业务逻辑放 `features/*` 或 hook。
- **MUST** 列表 `key` 稳定且唯一；可重排列表不用 index 当 key。

### 3.3 Hooks

- **MUST** 只在函数组件或自定义 Hook 的顶层调用 Hooks；不在条件、循环、嵌套函数、`try/catch`、条件 return 之后调用。
- **MUST** 启用 `eslint-plugin-react-hooks` 的 `rules-of-hooks` 与 `exhaustive-deps`；不靠手写依赖数组「看起来对」。
- **MUST** 自定义 Hook 以 `use` 开头；一个 Hook 只做一件事，返回稳定形状。
- **MUST** effect 有清理函数；订阅、定时器、Tauri `listen` 必须 `unlisten`。
- **SHOULD** `useMemo` / `useCallback` 只在有测量或引用稳定性需求时使用；不要默认包一层。
- **SHOULD** 不把服务端数据塞进 `useState` 再手动同步；用 TanStack Query。

### 3.4 Effects：能不用就不用

React 官方《You Might Not Need an Effect》的核心结论：

- **MUST** 能从 props/state 推导的值，在渲染时算，不放 effect + state。
- **MUST** 由用户事件触发的事情，放事件处理器，不放 effect。
- **MUST** 不写 effect 链（一个 effect 改 state，另一个再改 state）；在事件里一次算完。
- **MUST** props 变化要重置 state 时，优先用 `key` 重建组件，而不是 effect 同步。
- **MUST** 只在「与外部系统同步」时用 effect：Tauri 事件订阅、DOM 订阅、非 React 的 store、网络连接。
- **SHOULD** 数据获取用 TanStack Query（或框架 loader），不用手写 `useEffect` + `fetch`。
- **MUST** effect 在 StrictMode 双调用下保持幂等；订阅要有对称的 cleanup。

Tauri 事件清理的正确写法（来自 Tauri 文档）：

```ts
useEffect(() => {
  const unlisten = listen<number>("sync://progress", (event) => {
    setProgress(event.payload);
  });
  return () => {
    unlisten.then((fn) => fn());
  };
}, []);
```

### 3.5 状态管理

- **MUST** 局部状态优先；只有跨组件共享时才提升/进 store。
- **MUST** 服务端/IPC 状态归 TanStack Query：`useQuery` / `useMutation`，query key 形如 `['skills']`、`['targets']`、`['reconcile']`。
- **MUST** 不把 Query 返回的数据复制进 Zustand；Zustand 只放纯 UI 状态（当前选中、面板宽度、筛选、命令面板）。
- **MUST** 写操作后精确失效相关 query；不要全局 `invalidateQueries()` 了事，除非确实需要全量刷新。
- **SHOULD** 用选择器订阅 Zustand，避免整个 store 变化导致全组件重渲染。
- **SHOULD** 可推导的状态不存：`filteredSkills`、`issueCount`、`coverage` 在渲染时算。
- **SHOULD** 表单/临时输入用局部 state；跨页面草稿才进 store。
- **MAY** 路由状态放 React Router（hash 模式）；不要把路由状态复制进 Zustand。

TanStack Query 的默认值要有意识地配置：缓存默认立即视为 stale，窗口聚焦/重连会后台重取，失败默认重试 3 次，inactive 5 分钟后 GC。桌面本地 IPC 建议明确设置 `staleTime` 与 `retry`，不要依赖「碰巧」。

### 3.6 IPC 与数据流

- **MUST** 组件不直接 `invoke`；所有调用走 `src/lib/api.ts` 的 typed promise。
- **MUST** `mockApi` 只在 `import.meta.env.DEV` 下启用，不泄漏进生产包。
- **MUST** 每个异步操作有 loading / empty / error 三态；错误显示可操作建议。
- **MUST** 批量同步显示进度（`sync://progress`），可取消；已完成的步骤不回滚。
- **SHOULD** mutation 进行中禁用重复提交；乐观更新只在有明确回滚时使用。
- **SHOULD** 不轮询有事件可用的状态；事件监听在组件卸载时清理。

### 3.7 样式与设计 token

- **MUST** 颜色、圆角、字体走 `src/styles/tokens.css` 的 CSS 变量 / Tailwind `@theme`；组件里不写死 hex。
- **MUST** 状态文案与颜色映射集中 `src/lib/status.ts`；不在 JSX 里硬编码「副本过期」。
- **MUST** 状态不能只靠颜色表达；同时有文字/图标（满足色盲与深浅色）。
- **MUST** 正文 ≥ 14px，标签 ≥ 12px；正文对比度 ≥ 4.5:1；可点击区域 ≥ 32×32，主按钮 ≥ 36 高。
- **SHOULD** 用 `cn()`（clsx + tailwind-merge）合并类名；避免超长条件类名。
- **SHOULD** 深浅主题都用 token；新增颜色先加 token，再使用。
- **MUST** 尊重 `prefers-reduced-motion`；动效不是信息载体。
- **SHOULD** 响应式优先用容器/网格，不靠魔法断点；窄窗右栏改为抽屉。

### 3.8 可访问性

- **MUST** 用语义 HTML：`button` 就是 `button`，不拿 `div` 加 `onClick`。
- **MUST** 表单控件有 label；图标按钮有 `aria-label`；对话框有 `role="dialog"`、`aria-modal`、焦点陷阱、Esc 关闭。
- **MUST** 键盘可完成主流程：Tab 顺序、Enter/Space、方向键列表、`⌘/Ctrl+K` 命令面板。
- **MUST** 危险操作按钮不做默认焦点；冲突面板默认选中「保留现状」。
- **SHOULD** toast/异步结果用 `aria-live="polite"`；错误用 `role="alert"`。
- **SHOULD** 用 Testing Library 的 role/label 查询，顺便验证可访问名称。

### 3.9 文案与国际化

- **MUST** 所有 UI 文案进 `src/i18n/zh-CN.ts`（默认）与 `en.ts`；JSX 不散落中文字符串。
- **MUST** 内部状态映射集中 `status.ts`：`stale` → 「副本过期」，`untracked` → 「未托管」。
- **SHOULD** 动词开头、说清对象：「同步到 Claude Code」「在 Finder 中显示」。
- **SHOULD** 不把内部术语直接抛给用户；错误文案给下一步动作。

### 3.10 性能

- **SHOULD** 先测量再优化；不凭感觉加 `memo` / `useMemo`。
- **SHOULD** 长列表用虚拟化（技能很多时）；路由级代码分割；重型编辑器（CodeMirror）按需加载。
- **SHOULD** 避免在渲染路径做 O(n²) 过滤/排序；派生数据用一次遍历。
- **MAY** 用 React DevTools Profiler 验证；没有数据就不做「优化」。

### 3.11 前端安全

- **MUST** 不用 `dangerouslySetInnerHTML`；必须渲染富文本时先消毒，并说明来源。
- **MUST** 不把 token/密钥写进前端、localStorage、日志；敏感信息留在 Rust/keychain。
- **MUST** 不加载远程脚本/iframe；CSP 之外的域名一律不加。
- **SHOULD** 外部链接用 `rel="noreferrer noopener"`；路径展示用文本，不拼 HTML。

### 3.12 前端测试

- **MUST** `status.ts` 映射、筛选/排序、同步计划展示逻辑有 Vitest 单测。
- **MUST** 关键面板（空态/错误态/冲突态）有 Testing Library 测试；测试行为，不测实现细节。
- **SHOULD** 查询优先 `getByRole` / `getByLabelText` / `getByText`；用 `user-event` 模拟真实交互。
- **SHOULD** 在 `api.ts` 边界 mock IPC；组件测试不直接 mock Tauri 内部。
- **MAY** 加 `jest-axe` / `vitest-axe` 做基础可访问性断言。

---

## 4. Tauri 2 规范

### 4.1 信任模型

Tauri 的边界很清楚：**Rust 核心拥有完整系统权限，WebView 里的前端是不可信的一侧**。IPC 是唯一的桥；所有跨边界的数据都要校验。

- **MUST** 把前端当成可能被 XSS/供应链攻破的一方；敏感操作在 Rust 侧再校验一次。
- **MUST** 不因为「前端已经校验过」就跳过 Rust 校验。
- **SHOULD** 记住 capabilities/CSP 保护的是 WebView 的 API 暴露面，不保护有 bug 的 Rust 代码。

### 4.2 Capabilities：最小权限

- **MUST** `src-tauri/capabilities/default.json` 只开 `core:default` + `dialog:default`；不启用 `shell`、不开宽泛 `fs` scope。
- **MUST** 不添加 `remote.urls`；默认只允许随包发布的前端访问 IPC。
- **MUST** 新增插件权限时在 PR 里写明为什么、暴露了什么、如何限制 scope；capability diff 按安全改动审。
- **SHOULD** 按窗口/平台拆分 capability；桌面与移动不要共用一份大权限。
- **MUST** 注意：**注册的自定义命令默认对所有窗口开放**。敏感命令用 `tauri_build::AppManifest::commands(&[...])` 做构建期白名单，或在命令内做显式权限检查。
- **SHOULD** 定期跑 `pnpm tauri info` / 检查 `gen/schemas`，确认权限没有随插件升级悄悄扩大。

### 4.3 CSP 与远程内容

- **MUST** 在 `tauri.conf.json` 设置 `app.security.csp`；不设等于没有这层保护。
- **MUST** 不加载远程页面/脚本；v1 不需要 remote API 访问。
- **MUST** 不用 `'unsafe-eval'`；`script-src` 尽量只允许 `'self'` 与 Tauri 注入的 nonce/hash。
- **SHOULD** 从最小策略起步，例如：

```json
{
  "app": {
    "security": {
      "csp": {
        "default-src": "'self'",
        "connect-src": "ipc: http://ipc.localhost",
        "img-src": "'self' asset: http://asset.localhost blob: data:",
        "style-src": "'self' 'unsafe-inline'",
        "font-src": "'self'"
      }
    }
  }
}
```

- **SHOULD** `style-src 'unsafe-inline'` 只作为 Tailwind/内联样式的必要例外；能去掉就去掉。若未来接入远程字体/图片，逐域名加白名单，不用 `*`。
- **MAY** 如果未来要渲染不可信远程内容，再引入 isolation pattern；v1 不加载远程内容，不提前加复杂度。

### 4.4 IPC 设计

- **MUST** 命令薄：参数校验 → 调 core → 错误转换；不在命令里写业务分支。
- **MUST** 参数/返回值实现 `Serialize`/`Deserialize`；错误类型也实现 `Serialize`。
- **MUST** 错误返回结构化对象（`code`/`message`/`detail`），不用裸 `String`。
- **SHOULD** 大文件/大目录操作传路径或 ID，不在 IPC 里传整份文件内容；Rust 直接读。
- **SHOULD** 重活 `async` + `spawn_blocking`；流式数据用 `Channel`，通知用事件。
- **MUST** 事件名带命名空间（`sync://progress`），payload 有版本/类型字段；监听器卸载时 `unlisten`。
- **MUST** 不把密钥、token、技能正文放进 IPC 日志。

### 4.5 状态与数据库

- **MUST** 单个 `Mutex<Connection>` 足够；不要在每给命令里新建连接。
- **MUST** 不把 `MutexGuard` 跨 `await`；需要异步锁时用 Tokio `Mutex` 并说明理由。
- **MUST** `State` 已经是共享引用，不要再包 `Arc`；把 `AppHandle` clone 到线程里再取 state 是推荐做法。
- **MUST** 文件/DB 阻塞操作在 `spawn_blocking` 中执行；IPC 线程不阻塞。
- **MUST** 迁移只追加；启动时执行；升级测试覆盖上一版本数据库。
- **SHOULD** 写事务包住「删除链接 + 删 DB 行」等多步操作，失败可回滚。

### 4.6 文件系统与平台

- **MUST** 所有 FS 操作在 Rust；前端不直接碰文件系统。
- **MUST** 应用数据目录用 `app_data_dir()`；支持 `SKILLPORT_DATA_DIR` 覆盖；启动校验库不在任何 target 的 `skills_dir` 内。
- **MUST** macOS 不开启 App Sandbox（会阻止写其他 harness 目录）；这是有意的安全取舍，发布说明里写清。
- **MUST** Windows 处理 symlink 权限、长路径、UNC；symlink 失败时按 `AGENTS.md` 8.2 提示原因并询问是否改用 copy，不自动创建 junction；`#[cfg(windows)]` 隔离平台代码。
- **MUST** 目标默认路径逐个 harness 版本核实，不凭记忆写死；检测不到就让用户手填。
- **SHOULD** 所有路径展示用 canonical/归一化后的形式；日志保留原始输入以便排错。

### 4.7 构建、签名与更新

- **MUST** 发布前签名/公证（macOS Developer ID + notarize + staple；Windows 建议代码签名）。
- **MUST** updater 签名密钥离线保管；不把私钥放进 CI 明文变量。
- **MUST** 不开启未签名的自动更新；更新源不可控时宁可不自动更新。
- **SHOULD** 依赖升级后跑一次真实安装包冒烟；Tauri/插件升级看 changelog 的 security 部分。
- **SHOULD** 开发服务器只在可信网络暴露；不在不可信网络开 `tauri dev` 的 HMR 端口。

### 4.8 测试与验收

- **MUST** Rust core 单测覆盖状态机与路径守卫；命令层有集成测试（临时数据目录）。
- **MUST** 前端用 `mockApi` 调 UI；生产包不能包含 mock 分支。
- **MUST** 发版前 macOS + Windows 各跑一遍真实流程（临时 HOME/测试账号）：
  导入 → 加 3 个目标 → 核对 → 改库文件 → 副本目标显示过期 → 重新同步。
- **MUST** 目标目录预置同名外部目录，确认标冲突且不被修改。
- **MUST** 断网、只读目录、无权限、路径不存在四种异常都有可理解提示。

---

## 5. CR（Code Review / Change Request）规范

### 5.1 目的与标准

CR 的首要目的不是「挑错」，而是让代码库的整体健康度持续变好。采用 Google 的标准：

> In general, reviewers should favor approving a CL once it is in a state where it definitely improves the overall code health of the system being worked on, even if the CL isn’t perfect.
>
> 一般来说，只要改动明确改善了系统的整体代码健康，reviewer 就应倾向于批准，即使它并不完美。

- **MUST** 不以「完美」为批准门槛；不以个人偏好阻塞。
- **MUST** 技术事实/数据 > 个人意见；风格问题交给 rustfmt/ESLint/Prettier，不在 CR 里争论。
- **MUST** 不批准明确让代码库变差的改动（除非事故应急，且事后补）。
- **SHOULD** 看到做得好的地方要明确说（`praise:`），尤其是作者认真处理了你的意见时。

### 5.2 PR 大小与形状

- **MUST** 一个 PR 只做一件自包含的事；重构、格式化、功能、修 bug 分开。
- **SHOULD** 目标 100–400 行有效 diff；1000 行通常太大。文件数也是大小的一部分。
- **MUST** 行为改动带测试；纯重构也要有测试兜底（没有就先补测试 PR）。
- **MUST** 不破坏构建；多个依赖 PR 要保证每个合并后主干可用。
- **SHOULD** 大改提前和 reviewer 对齐拆分方式（按层/按文件/按垂直功能），不要写完再拆。

### 5.3 作者自查（发起 CR 前）

- [ ] PR 标题符合 Conventional Commits（`feat:` / `fix:` / `refactor:` / `docs:` / `test:` / `chore:`）。
- [ ] 描述写清：改了什么、为什么、怎么验证、风险与回滚。
- [ ] 自动化 gates 全绿（见 5.9）。
- [ ] 新增/修改的行为有测试；bug fix 有回归测试。
- [ ] 涉及 UI 的附截图/录屏；涉及路径/删除/迁移的附 dry-run 或测试输出。
- [ ] 没有提交密钥、真实用户路径、`node_modules`、`target/`、`*.sqlite3`。
- [ ] 相关文档/`AGENTS.md` 已更新。
- [ ] 没有夹带无关格式化/重命名；如果必须有，单独 PR。

### 5.4 Reviewer 检查清单

按 Google 的顺序，从设计到细节：

1. **设计**：改动放在正确的位置吗？应该进 core 还是 command？和现有架构一致吗？现在做合适吗？
2. **功能**：行为符合意图吗？对用户和后续开发者都好吗？边界、并发、错误路径想过了吗？
3. **复杂度**：是否过度设计？是否在解决「未来可能」的问题？函数/模块是否难快速理解？
4. **测试**：测试真的会在代码坏时失败吗？断言有意义吗？覆盖了空/错/冲突/并发吗？
5. **命名**：名字是否准确、够长但不啰嗦？
6. **注释**：解释「为什么」，不是复述「做了什么」；过时 TODO 是否清理？
7. **风格**：是否符合 rustfmt/ESLint/本文件；不因个人偏好阻塞。
8. **文档**：命令、结构、状态机、用户可见行为变了吗？文档同步了吗？
9. **每一行**：人类写的代码逐行看；生成文件/lockfile 可扫描，但要理解变更范围。
10. **上下文**：看整个文件/调用链，不只看 diff 附近；是否让系统整体更复杂？
11. **好事**：明确表扬做得好的设计/测试/命名。

### 5.5 评论约定（Conventional Comments）

格式：

```text
<label> [decorations]: <subject>

[discussion]
```

常用 label：

| label | 用途 |
|---|---|
| `praise:` | 真诚表扬，不滥用 |
| `nitpick:` | 琐碎偏好，天然 non-blocking |
| `suggestion:` | 提出改进，说明「改什么、为什么」 |
| `issue:` | 指明确切问题；尽量配 `suggestion` |
| `todo:` | 小而必要、合并前要做的改动 |
| `question:` | 不确定是否问题，先问清楚 |
| `thought:` | 评审时冒出的想法，non-blocking |
| `chore:` | 流程性事项，附文档链接 |
| `note:` | 提醒，non-blocking |

decoration：

- `(blocking)`：不解决不能合并。
- `(non-blocking)`：不阻塞本次合并，可开 follow-up。
- `(if-minor)`：改动小就顺手修，大了就另开。

示例：

```text
issue (blocking, security): 这里直接 remove_dir_all 了目标目录。
如果该目录不是 SkillPort 创建的，会删除用户数据。请改走 delete_owned_* 守卫，
并补一个「外部同名目录不被删除」的测试。
```

- **MUST** 报 bug 时给「位置 + 复现命令/步骤 + 实际输出 + 期望」；不要只说「这里不对」。
- **MUST** 不把多个无关问题塞进一条评论；一条评论一个主题。
- **SHOULD** 建议用 patch/代码片段，节省作者来回。
- **SHOULD** 对事不对人；用「这段代码」而不是「你」。

### 5.6 严重级别与合并

| 级别 | 含义 | 处理 |
|---|---|---|
| **P0 / blocking** | 数据丢失、安全漏洞、路径逃逸、构建/测试红、硬约束违反 | 必须修，修完重新审 |
| **P1 / should-fix** | 正确性风险、明显可维护性问题、缺关键测试 | 原则上本 PR 修；确实大才开 issue 并注明 |
| **P2 / nit** | 命名、注释、局部风格、可选优化 | 作者可自行决定；不阻塞 |
| **question** | 需要澄清 | 澄清后升级或撤回 |

- **MUST** 至少 1 名 reviewer 批准；作者不能自我批准。
- **MUST** 涉及 capabilities/CSP/路径守卫/删除/迁移/updater 的改动，至少 1 名熟悉 Rust/Tauri 的 reviewer 额外确认。
- **SHOULD** 1 个工作日内给首轮反馈；不能及时审就转给更合适的人，不让 PR 干等。
- **MUST** 冲突先基于规范讨论；讨论不拢就升级（TL/维护者），不把 PR 挂死。

### 5.7 本项目专属 CR 检查清单

**数据安全**

- [ ] 没有对非本工具拥有的路径做删除/覆盖；删除走 `delete_owned_*`。
- [ ] 路径守卫齐全：slug 校验、canonicalize 父目录、`starts_with(skills_dir)`、拒绝 `..`/绝对路径/UNC。
- [ ] 用 `symlink_metadata`，不跟随链接；外部 symlink 有确认/拒绝逻辑。
- [ ] 冲突/未托管不会被静默覆盖；UI 默认「保留现状」。
- [ ] copy 走临时目录 + sidecar + 原子 rename；覆盖前备份；备份保留 20 份。
- [ ] 树 hash 排除 sidecar/.DS_Store/Thumbs.db，顺序无关。
- [ ] 导入严格校验：缺 `SKILL.md`、frontmatter、`name` 或 `description` 被拒绝，且不写库。

**同步状态机**

- [ ] 状态矩阵（missing/ok/stale/conflict/untracked/error）有测试。
- [ ] plan 有 TTL（5 分钟），过期重算；apply 幂等；重复执行结果一致。
- [ ] 批量同步有进度、可取消；已完成步骤不回滚；取消后状态可重建。
- [ ] 不把 `installs.status` 当唯一真相；能从文件系统重建。

**Rust**

- [ ] 没有 `unwrap`/`expect` 处理用户输入/路径/IO；错误有 `code`。
- [ ] 没把 `MutexGuard` 跨 `await`；阻塞操作在 `spawn_blocking`。
- [ ] 迁移只追加；事务边界正确；WAL/foreign_keys 已开。
- [ ] 没记录技能正文/密钥；日志有轮转。

**React**

- [ ] IPC 走 `api.ts`；组件不直接 `invoke`。
- [ ] 服务端状态在 TanStack Query，不复制进 Zustand；写后失效精确。
- [ ] 没有不必要的 effect/effect 链；Tauri 事件有 cleanup。
- [ ] loading/empty/error/conflict 四态齐全；键盘可操作；深浅色可读。
- [ ] 文案在 i18n；状态映射在 `status.ts`。

**Tauri/安全**

- [ ] capabilities 没有扩大；新增权限有理由和 scope 限制。
- [ ] CSP 没有放 `unsafe-eval`/`*`；没有新增远程内容。
- [ ] 敏感自定义命令有权限/参数校验；没有把前端输入直接当路径用。
- [ ] updater/签名/密钥处理没有退化。

**跨平台**

- [ ] Windows symlink 失败 → 提示原因并询问 copy；拒绝 → 无操作；没有自动 junction。
- [ ] macOS 不开沙盒的前提下路径可写；没有硬编码平台路径。
- [ ] 测试用临时目录，不碰真实 `~/.claude`、`~/.codex`。

**文档**

- [ ] `AGENTS.md` 的命令、目录、状态机、IPC 表面同步更新。
- [ ] 用户可见行为变化有说明；破坏性变更写迁移/回滚。

### 5.8 对 agent 生成改动的额外要求

- **MUST** 把 agent 的输出当作作者提交的代码，不因为它「说通过了」就批准。
- **MUST** 作者提供实际运行的命令与输出（测试、lint、手动验证），不是总结。
- **MUST** reviewer 至少独立跑一次关键验证（路径守卫/同步/迁移相关必须）。
- **SHOULD** 让 agent 先给改动计划/影响面，再动手；大改拆小 PR。
- **SHOULD** 对「看起来合理但没验证」的结论保持怀疑；要求位置 + 复现 + 实际输出。
- **MAY** 用干净上下文的第二个 agent 做复审（fresh-eyes review），但结论仍要人来确认。

### 5.9 自动化 gates

合并前必须全绿：

```bash
pnpm typecheck
pnpm lint
pnpm test
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml
```

- **SHOULD** CI 在 macOS + Windows 双平台跑；Linux 尽力而为。
- **SHOULD** 前端构建 + `tauri build` 在发布分支跑一次。
- **MAY** 加 `cargo audit`、`pnpm audit`、依赖许可证检查为定时任务。

### 5.10 PR 模板

```markdown
## 改了什么
<!-- 一两句话；关联 issue -->

## 为什么
<!-- 用户问题/设计约束；为什么现在做 -->

## 怎么验证
- [ ] pnpm typecheck / lint / test
- [ ] cargo fmt / clippy / test
- [ ] macOS 手动验证（临时 HOME）
- [ ] Windows 手动验证（临时目录）
- [ ] UI 截图/录屏（如适用）

## 风险与回滚
<!-- 数据/路径/迁移/权限风险；如何回滚 -->

## 安全与兼容
- [ ] 未扩大 Tauri capabilities / CSP
- [ ] 未对非本工具拥有的路径做删除/覆盖
- [ ] 迁移向后兼容
```

---

## 6. 建议的工具配置

### 6.1 Rust

`rustfmt.toml`（只用 stable 选项）：

```toml
edition = "2021"
max_width = 100
```

`clippy.toml`：

```toml
msrv = "1.80"
```

如果要把 `unwrap` 变成硬错误，按文件/模块逐个开，不要全仓库一刀切：

```rust
#![deny(clippy::unwrap_used, clippy::expect_used)]
```

测试模块里用 `#[allow(clippy::unwrap_used)]`，并保持豁免范围最小。

### 6.2 TypeScript / ESLint

`tsconfig.json` 关键项：

```json
{
  "compilerOptions": {
    "strict": true,
    "noUncheckedIndexedAccess": true,
    "noImplicitOverride": true,
    "noFallthroughCasesInSwitch": true,
    "verbatimModuleSyntax": true,
    "jsx": "react-jsx",
    "moduleResolution": "bundler",
    "noEmit": true
  }
}
```

ESLint 目标规则（按实际安装版本调整插件配置）：

- `typescript-eslint` 的 `strictTypeChecked` + `stylisticTypeChecked`。
- `@typescript-eslint/no-explicit-any: error`
- `@typescript-eslint/no-floating-promises: error`
- `@typescript-eslint/no-misused-promises: error`
- `@typescript-eslint/consistent-type-imports: error`
- `@typescript-eslint/switch-exhaustiveness-check: error`
- `react-hooks/rules-of-hooks: error`
- `react-hooks/exhaustive-deps: warn`（CI 可升 error）
- `jsx-a11y` recommended
- `react-refresh/only-export-components: warn`

原则：**格式化交给工具，CR 讨论设计和风险**。不要用大量 `eslint-disable` 换安静；每条豁免写原因。

### 6.3 CI 矩阵

建议：

- `macos-latest` + `windows-latest`。
- Node 22、pnpm 10、Rust stable。
- 缓存 pnpm store 与 Cargo registry/target。
- 依次跑前端三件套、Rust 三件套。
- 发布分支额外跑 `pnpm tauri build` 与签名/公证步骤（密钥只在受保护环境）。

---

## 7. 一分钟合并前检查

- [ ] 这个 PR 只做一件事，描述能让我不看代码就懂？
- [ ] 测试会在代码坏的时候红吗？
- [ ] 有没有可能删掉/覆盖不属于我们的东西？
- [ ] 路径、slug、symlink 的边界都守住了吗？
- [ ] 错误有 `code`，前端能分支处理吗？
- [ ] 有没有把服务端状态复制进 Zustand / 手写 effect 同步？
- [ ] Tauri 权限、CSP、IPC 输入校验有没有退化？
- [ ] 深浅色、键盘、空/错/冲突态都看过了吗？
- [ ] 文档和 `AGENTS.md` 跟上了吗？
- [ ] 有没有该表扬的地方？

---

## 8. 来源

访问日期：2026-11-06。以下为本文件的主要依据；社区文章会更新，冲突时以官方文档与项目 `AGENTS.md` 为准。

### Rust

- [Rust API Guidelines · Checklist](https://rust-lang.github.io/api-guidelines/checklist.html) — 命名、trait、错误、文档、可预测性、类型安全、semver。
- [Rust API Guidelines · Naming](https://rust-lang.github.io/api-guidelines/naming.html) — RFC 430 与 `as_`/`to_`/`into_`。
- [Clippy Usage](https://doc.rust-lang.org/clippy/usage.html) — lint 组、`-D warnings`、`pedantic`/`restriction`。
- [Clippy Lint List](https://rust-lang.github.io/rust-clippy/master/index.html) — 逐条 lint 说明。
- [rustfmt](https://rust-lang.github.io/rustfmt/) — 格式化配置。
- [The Rust Book · Error Handling](https://doc.rust-lang.org/book/ch09-00-error-handling.html) — `Result`、`?`、panic 边界。
- [The Rustonomicon](https://doc.rust-lang.org/nomicon/) — unsafe 的前置条件与安全抽象。
- [Rust Design Patterns](https://rust-unofficial.github.io/patterns/) — 社区模式库（newtype、RAII 等）。
- [Effective Rust](https://effective-rust.com/) — 社区实践建议。
- [Cargo · build.warnings](https://doc.rust-lang.org/cargo/reference/config.html#buildwarnings) — CI 把 warning 升 error 的推荐方式。

### React / TypeScript

- [React · Rules of Hooks](https://react.dev/reference/rules/rules-of-hooks) — Hook 调用位置与 eslint 插件。
- [React · You Might Not Need an Effect](https://react.dev/learn/you-might-not-need-an-effect) — 派生状态、事件、订阅、数据获取。
- [React · Choosing the State Structure](https://react.dev/learn/choosing-the-state-structure) — 避免冗余/矛盾/重复状态。
- [React · Using TypeScript](https://react.dev/learn/typescript) — props、Hooks、联合类型。
- [React · Reusing Logic with Custom Hooks](https://react.dev/learn/reusing-logic-with-custom-hooks) — 自定义 Hook 边界。
- [TanStack Query · Overview](https://tanstack.com/query/latest/docs/framework/react/overview) — 服务端状态与客户端状态的区别。
- [TanStack Query · Important Defaults](https://tanstack.com/query/latest/docs/framework/react/guides/important-defaults) — stale/retry/gc/refetch 默认值。
- [Zustand · Docs](https://zustand.docs.pmnd.rs/) — store、action、选择器；本项目只把它用于 UI 状态。
- [typescript-eslint · no-explicit-any](https://typescript-eslint.io/rules/no-explicit-any/) — `unknown`、索引签名、联合类型替代。
- [typescript-eslint · Typed Linting](https://typescript-eslint.io/getting-started/typed-linting/) — 类型感知规则配置。
- [eslint-plugin-react-hooks](https://www.npmjs.com/package/eslint-plugin-react-hooks) — `rules-of-hooks` / `exhaustive-deps`。
- [eslint-plugin-jsx-a11y](https://github.com/jsx-eslint/eslint-plugin-jsx-a11y) — 可访问性 lint。
- [Testing Library · Guiding Principles](https://testing-library.com/docs/guiding-principles) — 测试贴近用户使用方式。

### Tauri 2

- [Tauri · Security](https://v2.tauri.app/security/) — 信任边界、CSP、capabilities、isolation。
- [Tauri · Capabilities](https://v2.tauri.app/security/capabilities/) — 权限、平台、remote、自定义命令白名单。
- [Tauri · CSP](https://v2.tauri.app/security/csp/) — nonce/hash、最小策略。
- [Tauri · Application Lifecycle Threats](https://v2.tauri.app/security/lifecycle/) — 依赖、开发服务器、构建、分发、运行时威胁。
- [Tauri · Calling Rust from the Frontend](https://v2.tauri.app/develop/calling-rust/) — 命令、错误、async、channels、events。
- [Tauri · Calling the Frontend from Rust](https://v2.tauri.app/develop/calling-frontend/) — 事件、channel、监听清理。
- [Tauri · State Management](https://v2.tauri.app/develop/state-management/) — `State`、`Mutex`、async mutex、`Arc`。
- [Tauri · Architecture](https://v2.tauri.app/concept/architecture/) — WebView/Rust 进程与消息传递。
- [Tauri · Config Reference](https://v2.tauri.app/reference/config/) — `app.security`、bundle、updater。

### Code Review

- [Google · The Standard of Code Review](https://google.github.io/eng-practices/review/reviewer/standard.html) — 以代码健康为标准，不追求完美。
- [Google · What to Look for in a Code Review](https://google.github.io/eng-practices/review/reviewer/looking-for.html) — 设计/功能/复杂度/测试/命名/注释/风格/文档/每一行。
- [Google · Small CLs](https://google.github.io/eng-practices/review/developer/small-cls.html) — 小改动、拆分、测试同 CL、重构分离。
- [Google · Speed of Code Reviews](https://google.github.io/eng-practices/review/reviewer/speed.html) — 及时响应，不阻塞作者。
- [Conventional Comments](https://conventionalcomments.org/) — `praise`/`nitpick`/`suggestion`/`issue`/`todo`/`question` 与 blocking 装饰。
- [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/) — 提交类型与语义。

---

## 9. 维护

- 本文件与 `AGENTS.md` 同步更新；任何状态机、目录结构、IPC 表面、安全约束变化，都在同一个 PR 里改。
- 每季度复查一次来源链接与工具版本；失效链接替换，不删除历史依据。
- 发生一次真实事故/差点出事，补一条**可执行**的规则（带检查点），不要只写「要小心」。
- 规则有例外时，在 PR 里写清楚：为什么例外、影响面、如何回滚；不要把例外变成默认。
