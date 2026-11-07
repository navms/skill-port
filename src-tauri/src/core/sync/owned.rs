//! 只删除本工具拥有的分发条目。

use std::path::Path;

use crate::core::error::AppResult;
use crate::core::fsops;
use crate::core::model::{Skill, Target};
use crate::core::paths::{self, AppPaths};

/// 一次清理的结果；`removed == false` 表示因归属不明而保留。
#[derive(Debug, Clone)]
pub struct RemovalOutcome {
    pub removed: bool,
    pub detail: Option<String>,
}

fn kept(detail: &str) -> RemovalOutcome {
    RemovalOutcome {
        removed: false,
        detail: Some(detail.to_string()),
    }
}

/// 删除某个技能在某个目标下的分发条目。
///
/// - 符号链接：仅在指向本工具库内时删除；`remove_copy` 不影响链接
///   （保留会变成悬空链接）。
/// - 副本：仅当 sidecar 归属匹配且 `remove_copy` 为真时移入备份。
/// - 其余情况一律保留并报告原因。
pub fn remove_distribution(
    app_paths: &AppPaths,
    skill: &Skill,
    target: &Target,
    remove_copy: bool,
    keep: usize,
) -> AppResult<RemovalOutcome> {
    let skills_dir = Path::new(&target.skills_dir);
    if !skills_dir.is_dir() {
        return Ok(RemovalOutcome {
            removed: false,
            detail: Some("目标目录不存在".to_string()),
        });
    }
    let target_path = paths::guarded_target_path(skills_dir, &skill.slug)?;
    let meta = match std::fs::symlink_metadata(&target_path) {
        Ok(meta) => meta,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(RemovalOutcome {
                removed: false,
                detail: Some("目标条目不存在".to_string()),
            })
        }
        Err(error) => return Err(crate::core::error::AppError::io_at(&target_path, error)),
    };

    if meta.file_type().is_symlink() {
        if !fsops::symlink_points_into(&target_path, &app_paths.library_dir)? {
            return Ok(kept("符号链接不属于 SkillPort，已保留"));
        }
        paths::ensure_write_allowed(skills_dir, &target_path, &skill.slug)?;
        fsops::remove_symlink(&target_path)?;
        return Ok(RemovalOutcome {
            removed: true,
            detail: Some("已删除符号链接".to_string()),
        });
    }

    if meta.is_dir() {
        return match fsops::read_sidecar(&target_path) {
            Some(sidecar) if sidecar.skill_id == skill.id => {
                if !remove_copy {
                    return Ok(kept("按请求保留副本"));
                }
                paths::ensure_write_allowed(skills_dir, &target_path, &skill.slug)?;
                fsops::backup_dir(
                    &app_paths.backups_dir,
                    &skill.slug,
                    &target.slug,
                    &target_path,
                    keep,
                )?;
                Ok(RemovalOutcome {
                    removed: true,
                    detail: Some("副本已移入备份".to_string()),
                })
            }
            _ => Ok(kept("目标目录不属于 SkillPort，已保留")),
        };
    }

    Ok(kept("目标路径不是目录，已保留"))
}
