//! 领域模型：与 `src/lib/types.ts` 一一对应（IPC 上使用 camelCase）。

use serde::{Deserialize, Serialize};

/// 分发方式。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LinkMode {
    Symlink,
    Copy,
}

impl LinkMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Symlink => "symlink",
            Self::Copy => "copy",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "symlink" => Some(Self::Symlink),
            "copy" => Some(Self::Copy),
            _ => None,
        }
    }
}

/// 某条 Install 的实测状态（DB 中只是缓存，可从文件系统重建）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum InstallStatus {
    Pending,
    Ok,
    Stale,
    Missing,
    Conflict,
    Error,
}

impl InstallStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Ok => "ok",
            Self::Stale => "stale",
            Self::Missing => "missing",
            Self::Conflict => "conflict",
            Self::Error => "error",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "pending" => Some(Self::Pending),
            "ok" => Some(Self::Ok),
            "stale" => Some(Self::Stale),
            "missing" => Some(Self::Missing),
            "conflict" => Some(Self::Conflict),
            "error" => Some(Self::Error),
            _ => None,
        }
    }

    /// 是否是需要用户处理的问题状态。
    pub fn is_issue(self) -> bool {
        !matches!(self, Self::Ok)
    }
}

/// 目标路径上实际存在的东西。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ActualKind {
    Symlink,
    /// v1 不主动创建，仅兼容既有数据。
    Junction,
    Copy,
}

impl ActualKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Symlink => "symlink",
            Self::Junction => "junction",
            Self::Copy => "copy",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "symlink" => Some(Self::Symlink),
            "junction" => Some(Self::Junction),
            "copy" => Some(Self::Copy),
            _ => None,
        }
    }
}

/// 技能来源。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SourceKind {
    Local,
    Import,
    Git,
}

impl SourceKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Local => "local",
            Self::Import => "import",
            Self::Git => "git",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "local" => Some(Self::Local),
            "import" => Some(Self::Import),
            "git" => Some(Self::Git),
            _ => None,
        }
    }
}

/// 一个消费技能的宿主。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Target {
    pub id: String,
    pub slug: String,
    pub name: String,
    pub kind: String,
    pub skills_dir: String,
    pub link_mode: LinkMode,
    pub enabled: bool,
    pub detected: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// 库中的权威技能副本。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Skill {
    pub id: String,
    pub slug: String,
    pub name: String,
    pub description: String,
    /// SkillPort 自己的归类口径，只存 DB；导入时一律留空，仅由编辑对话框修改，从不写回 `SKILL.md`。
    pub category: Option<String>,
    pub source_kind: SourceKind,
    pub source_ref: Option<String>,
    pub content_hash: String,
    pub created_at: String,
    pub updated_at: String,
}

/// 「某个技能应存在于某个目标」的期望记录 + 最近一次核对结果。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Install {
    pub id: i64,
    pub skill_id: String,
    pub target_id: String,
    pub mode: LinkMode,
    pub actual_kind: Option<ActualKind>,
    pub status: InstallStatus,
    pub target_path: String,
    pub linked_hash: Option<String>,
    /// 用户对外部冲突选择「保留并忽略」。
    pub ignored: bool,
    pub last_error: Option<String>,
    pub last_checked: Option<String>,
    pub last_synced: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillSummary {
    pub id: String,
    pub slug: String,
    pub name: String,
    pub description: String,
    /// SkillPort 自有的分类，存在 DB 里，从不写回 `SKILL.md`。
    pub category: Option<String>,
    pub source_kind: SourceKind,
    pub source_ref: Option<String>,
    pub content_hash: String,
    /// 库目录内文件总字节数，扫描时填充。
    pub size_bytes: u64,
    pub created_at: String,
    pub updated_at: String,
    pub installs: Vec<Install>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillDetail {
    #[serde(flatten)]
    pub summary: SkillSummary,
    pub path: String,
    pub skill_md: String,
}

/// 一个 harness 图标：`slug` 与 target slug 同名，`svg` 是文件原文。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TargetIconAsset {
    pub slug: String,
    pub svg: String,
}

/// 导入失败原因（与前端 `ImportErrorCode` 一致）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum ImportErrorCode {
    MissingSkillFile,
    InvalidFrontmatter,
    MissingName,
    MissingDescription,
    InvalidSlug,
    DuplicateSlug,
    PathNotFound,
    IoError,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportResultItem {
    pub path: String,
    pub slug: Option<String>,
    pub ok: bool,
    pub error_code: Option<ImportErrorCode>,
    pub detail: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportReport {
    pub items: Vec<ImportResultItem>,
    pub imported: usize,
    pub skipped: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SuggestedTarget {
    pub slug: String,
    pub name: String,
    pub skills_dir: String,
    pub exists: bool,
    pub already_added: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UntrackedEntry {
    pub target_id: String,
    pub target_slug: String,
    pub target_name: String,
    pub name: String,
    pub path: String,
}

/// 核对汇总。`changed` 是本次状态发生变化的 Install 数。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReconcileReport {
    pub checked: usize,
    pub changed: usize,
    pub issues: usize,
    pub checked_at: String,
}

/// 同步计划中的动作类型（前端据此分组）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum SyncActionKind {
    Create,
    Repair,
    Skip,
    Conflict,
}

impl SyncActionKind {
    /// 只有 create / repair 会被执行。
    pub fn is_actionable(self) -> bool {
        matches!(self, Self::Create | Self::Repair)
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanAction {
    pub id: String,
    pub skill_id: String,
    pub skill_slug: String,
    pub skill_name: String,
    pub target_id: String,
    pub target_name: String,
    pub target_path: String,
    pub mode: LinkMode,
    pub kind: SyncActionKind,
    pub state: InstallStatus,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncPlan {
    pub id: String,
    pub created_at: String,
    pub expires_at: String,
    pub actions: Vec<PlanAction>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum SyncProgressStatus {
    Running,
    Ok,
    Failed,
    Conflict,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncProgress {
    pub plan_id: String,
    pub completed: usize,
    pub total: usize,
    pub action_id: Option<String>,
    pub status: SyncProgressStatus,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncResultItem {
    pub action_id: String,
    pub ok: bool,
    pub error_code: Option<String>,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncResult {
    pub plan_id: String,
    pub applied: usize,
    /// 失败的动作数（不是列表）。
    pub failed: usize,
    pub cancelled: bool,
    /// 可用于 `activity_undo` 的活动 id。
    pub undo_id: Option<String>,
    pub items: Vec<SyncResultItem>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivityEntry {
    pub id: i64,
    pub ts: String,
    pub action: String,
    pub skill_id: Option<String>,
    pub skill_name: Option<String>,
    pub target_id: Option<String>,
    pub target_name: Option<String>,
    pub result: String,
    pub detail: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivityPage {
    pub items: Vec<ActivityEntry>,
    pub next_cursor: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivityQuery {
    pub limit: Option<u32>,
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UndoResult {
    pub ok: bool,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TargetCreateInput {
    pub slug: String,
    pub name: String,
    pub skills_dir: String,
    #[serde(default = "default_link_mode")]
    pub link_mode: LinkMode,
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_link_mode() -> LinkMode {
    LinkMode::Symlink
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TargetUpdatePatch {
    pub slug: Option<String>,
    pub name: Option<String>,
    pub skills_dir: Option<String>,
    pub link_mode: Option<LinkMode>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillPatch {
    pub name: Option<String>,
    pub description: Option<String>,
    /// 新的完整 `SKILL.md` 内容；给出时按严格校验后整篇写入。
    pub skill_md: Option<String>,
    /// 分类（SkillPort 自有字段，只写 DB）。`None` 表示不改，空串表示清除。
    pub category: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteSkillOptions {
    /// 是否删除目标目录中本工具拥有的副本；符号链接无论该值如何都会清理，
    /// 否则会留下悬空链接。
    #[serde(default = "default_true")]
    pub delete_targets: bool,
}

impl Default for DeleteSkillOptions {
    fn default() -> Self {
        Self {
            delete_targets: true,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteSkillResult {
    pub undo_id: Option<String>,
    pub removed_installs: usize,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteTargetOptions {
    /// 是否删除目标目录中本工具拥有的链接/副本。
    #[serde(default = "default_true")]
    pub unlink: bool,
}

impl Default for DeleteTargetOptions {
    fn default() -> Self {
        Self { unlink: true }
    }
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallSetInput {
    pub enabled: bool,
    /// 省略时沿用已有 Install 的模式，否则用目标的默认模式。
    pub mode: Option<LinkMode>,
}

/// 冲突处理方式。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConflictResolution {
    Adopt,
    Ignore,
    Unignore,
    Cancel,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncFilter {
    pub skill_id: Option<String>,
    pub target_id: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsPatch {
    pub default_link_mode: Option<LinkMode>,
    pub backup_retention: Option<u32>,
    /// 只读字段；传入且与当前值不同会被拒绝。
    pub library_path: Option<String>,
    pub data_dir: Option<String>,
}

/// 持久化设置（库/数据目录是派生值，不入库）。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoredSettings {
    pub default_link_mode: LinkMode,
    pub backup_retention: u32,
}

impl Default for StoredSettings {
    fn default() -> Self {
        Self {
            default_link_mode: LinkMode::Symlink,
            backup_retention: 20,
        }
    }
}

/// 暴露给前端的设置快照。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub default_link_mode: LinkMode,
    pub library_path: String,
    pub data_dir: String,
    pub backup_retention: u32,
}
