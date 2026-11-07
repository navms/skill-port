//! 统一错误类型。前端按 `code` 分支，不匹配 message 文本。

use std::path::Path;

use serde::Serialize;

/// 所有核心层与命令层错误的统一类型。
///
/// 序列化后形如 `{ "code": "Conflict", "message": "…", "detail": "…" }`。
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("IO 错误：{message}")]
    Io {
        message: String,
        path: Option<String>,
    },

    #[error("数据库错误：{0}")]
    Db(String),

    #[error("未找到：{0}")]
    NotFound(String),

    #[error("无效的 slug：{0}")]
    InvalidSlug(String),

    #[error("路径越界：{0}")]
    PathEscape(String),

    #[error("目标路径已被占用且不属于 SkillPort：{0}")]
    Conflict(String),

    #[error("导入校验失败：{0}")]
    Validation(String),

    #[error("SKILL.md 不存在：{0}")]
    MissingSkillFile(String),

    #[error("frontmatter 无法解析：{path}")]
    InvalidFrontmatter { path: String, detail: String },

    #[error("frontmatter 缺少非空的 name：{0}")]
    MissingName(String),

    #[error("frontmatter 缺少非空的 description：{0}")]
    MissingDescription(String),

    #[error("缺少创建符号链接的权限：{path}")]
    SymlinkPrivilege { path: String, detail: String },

    #[error("同步计划已过期或不存在：{0}")]
    PlanExpired(String),

    #[error("操作已取消")]
    Cancelled,

    #[error("参数无效：{0}")]
    InvalidInput(String),

    #[error("内部错误：{0}")]
    Internal(String),
}

/// 跨 IPC 返回的结构化错误负载。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorInfo {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

impl AppError {
    /// 稳定错误码；前端按它分支。
    pub fn code(&self) -> &'static str {
        match self {
            Self::Io { .. } => "Io",
            Self::Db(_) => "Db",
            Self::NotFound(_) => "NotFound",
            Self::InvalidSlug(_) => "InvalidSlug",
            Self::PathEscape(_) => "PathEscape",
            Self::Conflict(_) => "Conflict",
            Self::Validation(_) => "Validation",
            Self::MissingSkillFile(_) => "MissingSkillFile",
            Self::InvalidFrontmatter { .. } => "InvalidFrontmatter",
            Self::MissingName(_) => "MissingName",
            Self::MissingDescription(_) => "MissingDescription",
            Self::SymlinkPrivilege { .. } => "SymlinkPrivilege",
            Self::PlanExpired(_) => "PlanExpired",
            Self::Cancelled => "Cancelled",
            Self::InvalidInput(_) => "InvalidInput",
            Self::Internal(_) => "Internal",
        }
    }

    /// 面向排错的补充信息；不会包含技能正文或密钥。
    pub fn detail(&self) -> Option<String> {
        match self {
            Self::Io { path, .. } => path.clone(),
            Self::InvalidFrontmatter { detail, .. } => Some(detail.clone()),
            Self::SymlinkPrivilege { detail, .. } => Some(detail.clone()),
            _ => None,
        }
    }

    /// 转换为可序列化负载。
    pub fn info(&self) -> ErrorInfo {
        ErrorInfo {
            code: self.code().to_string(),
            message: self.to_string(),
            detail: self.detail(),
        }
    }

    /// 为 IO 错误补充路径上下文。
    pub fn io_at(path: &Path, error: std::io::Error) -> Self {
        Self::Io {
            message: error.to_string(),
            path: Some(path.display().to_string()),
        }
    }
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.info().serialize(serializer)
    }
}

impl From<std::io::Error> for AppError {
    fn from(error: std::io::Error) -> Self {
        Self::Io {
            message: error.to_string(),
            path: None,
        }
    }
}

impl From<rusqlite::Error> for AppError {
    fn from(error: rusqlite::Error) -> Self {
        Self::Db(error.to_string())
    }
}

impl From<rusqlite_migration::Error> for AppError {
    fn from(error: rusqlite_migration::Error) -> Self {
        Self::Db(error.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(error: serde_json::Error) -> Self {
        Self::Internal(format!("JSON 解析失败：{error}"))
    }
}

impl From<serde_yaml_ng::Error> for AppError {
    fn from(error: serde_yaml_ng::Error) -> Self {
        Self::Validation(format!("YAML 解析失败：{error}"))
    }
}

/// 核心层统一返回类型。
pub type AppResult<T> = Result<T, AppError>;
