//! 单条分发的实测状态判定（只读）。

use std::path::Path;

use crate::core::model::{ActualKind, InstallStatus};
use crate::core::{fsops, paths};

/// 一次核对的实测结果。
#[derive(Debug, Clone)]
pub struct Inspection {
    pub status: InstallStatus,
    pub actual_kind: Option<ActualKind>,
    pub linked_hash: Option<String>,
    pub last_error: Option<String>,
    /// 目标路径是否为本工具拥有的条目。
    pub owned: bool,
}

impl Inspection {
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            status: InstallStatus::Error,
            actual_kind: None,
            linked_hash: None,
            last_error: Some(message.into()),
            owned: false,
        }
    }

    fn conflict(message: impl Into<String>, actual_kind: Option<ActualKind>, owned: bool) -> Self {
        Self {
            status: InstallStatus::Conflict,
            actual_kind,
            linked_hash: None,
            last_error: Some(message.into()),
            owned,
        }
    }
}

fn same_path(left: &Path, right: &Path) -> bool {
    if paths::normalize_lexically(left) == paths::normalize_lexically(right) {
        return true;
    }
    paths::canonical_or_normalized(left) == paths::canonical_or_normalized(right)
}

/// 判定 `target_path` 相对于库中技能目录的状态。
pub fn inspect_target(
    library_root: &Path,
    skill_id: &str,
    skill_dir: &Path,
    skill_hash: &str,
    target_path: &Path,
) -> Inspection {
    let meta = match std::fs::symlink_metadata(target_path) {
        Ok(meta) => meta,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Inspection {
                status: InstallStatus::Missing,
                actual_kind: None,
                linked_hash: None,
                last_error: None,
                owned: false,
            }
        }
        Err(error) => return Inspection::error(error.to_string()),
    };

    if meta.file_type().is_symlink() {
        return inspect_symlink(library_root, skill_dir, skill_hash, target_path);
    }

    if meta.is_dir() {
        return match fsops::read_sidecar(target_path) {
            Some(sidecar) if sidecar.skill_id == skill_id => {
                let status = if sidecar.source_hash == skill_hash {
                    InstallStatus::Ok
                } else {
                    InstallStatus::Stale
                };
                Inspection {
                    status,
                    actual_kind: Some(ActualKind::Copy),
                    linked_hash: Some(sidecar.source_hash),
                    last_error: None,
                    owned: true,
                }
            }
            Some(_) => Inspection::conflict(
                "目录由 SkillPort 管理，但属于其他技能",
                Some(ActualKind::Copy),
                false,
            ),
            None => Inspection::conflict(
                "目标目录已存在且不属于 SkillPort",
                Some(ActualKind::Copy),
                false,
            ),
        };
    }

    Inspection::conflict("目标路径已存在，且不是目录链接或受管副本", None, false)
}

fn inspect_symlink(
    library_root: &Path,
    skill_dir: &Path,
    skill_hash: &str,
    target_path: &Path,
) -> Inspection {
    let resolved = match fsops::resolve_link(target_path) {
        Ok(resolved) => resolved,
        Err(error) => return Inspection::error(error.to_string()),
    };
    if same_path(&resolved, skill_dir) {
        return Inspection {
            status: InstallStatus::Ok,
            actual_kind: Some(ActualKind::Symlink),
            linked_hash: Some(skill_hash.to_string()),
            last_error: None,
            owned: true,
        };
    }
    match fsops::symlink_points_into(target_path, library_root) {
        Ok(true) => {
            if std::fs::metadata(&resolved).is_ok() {
                Inspection::conflict("符号链接指向库内其他位置", Some(ActualKind::Symlink), true)
            } else {
                // 指向库内但目标已不存在的悬空链接：本工具遗留，可重新创建。
                Inspection {
                    status: InstallStatus::Missing,
                    actual_kind: Some(ActualKind::Symlink),
                    linked_hash: None,
                    last_error: None,
                    owned: true,
                }
            }
        }
        Ok(false) => Inspection::conflict(
            "符号链接指向 SkillPort 库以外的位置",
            Some(ActualKind::Symlink),
            false,
        ),
        Err(error) => Inspection::error(error.to_string()),
    }
}
