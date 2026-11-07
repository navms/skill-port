//! 应用设置（存储为 settings 表里的 JSON）。

use rusqlite::Connection;

use super::error::{AppError, AppResult};
use super::model::{AppSettings, SettingsPatch, StoredSettings};
use super::paths::AppPaths;
use super::repo;

const SETTINGS_KEY: &str = "app";
const MAX_BACKUP_RETENTION: u32 = 100;

/// 读取持久化设置；缺失或损坏时回退到默认值。
pub fn get(conn: &Connection) -> AppResult<StoredSettings> {
    match repo::get_setting(conn, SETTINGS_KEY)? {
        Some(raw) => Ok(serde_json::from_str(&raw).unwrap_or_default()),
        None => Ok(StoredSettings::default()),
    }
}

/// 合并并保存设置；非法值返回 `InvalidInput`。
pub fn update(
    conn: &Connection,
    paths: &AppPaths,
    patch: &SettingsPatch,
) -> AppResult<StoredSettings> {
    let mut settings = get(conn)?;
    reject_read_only(paths, patch)?;
    if let Some(mode) = patch.default_link_mode {
        settings.default_link_mode = mode;
    }
    if let Some(limit) = patch.backup_retention {
        if limit == 0 || limit > MAX_BACKUP_RETENTION {
            return Err(AppError::InvalidInput(format!(
                "备份保留数量必须在 1–{MAX_BACKUP_RETENTION} 之间"
            )));
        }
        settings.backup_retention = limit;
    }
    let raw = serde_json::to_string(&settings)?;
    repo::set_setting(conn, SETTINGS_KEY, &raw)?;
    Ok(settings)
}

/// 库目录与应用数据目录由启动参数决定，运行期不可修改。
fn reject_read_only(paths: &AppPaths, patch: &SettingsPatch) -> AppResult<()> {
    if let Some(value) = &patch.library_path {
        if value.trim() != paths.library_dir.display().to_string() {
            return Err(AppError::InvalidInput(
                "库目录不能在运行中修改；请通过应用数据目录迁移".to_string(),
            ));
        }
    }
    if let Some(value) = &patch.data_dir {
        if value.trim() != paths.data_dir.display().to_string() {
            return Err(AppError::InvalidInput(
                "应用数据目录不能在运行中修改".to_string(),
            ));
        }
    }
    Ok(())
}

/// 生成暴露给前端的完整设置快照。
pub fn snapshot(conn: &Connection, paths: &AppPaths) -> AppResult<AppSettings> {
    let settings = get(conn)?;
    Ok(AppSettings {
        default_link_mode: settings.default_link_mode,
        library_path: paths.library_dir.display().to_string(),
        data_dir: paths.data_dir.display().to_string(),
        backup_retention: settings.backup_retention,
    })
}
