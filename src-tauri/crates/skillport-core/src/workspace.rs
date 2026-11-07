//! 打开一个 SkillPort 数据目录：布局、数据库、库目录摆放校验与临时残留清理。
//!
//! 桌面端（`AppState::initialize`）与 CLI 都从这里进入，保证两边看到的库、DB 与
//! 路径守卫完全一致。打开过程幂等：目录不存在就创建，迁移只追加。

use std::path::{Path, PathBuf};

use rusqlite::Connection;

use super::error::AppResult;
use super::paths::AppPaths;
use super::{db, fsops, logging, paths, repo};

/// 打开结果：路径布局 + 已迁移的数据库连接。
pub struct Workspace {
    pub paths: AppPaths,
    pub conn: Connection,
}

/// 创建缺失目录、打开数据库、校验库目录不在任何目标目录内，并清理临时残留。
///
/// 只做只读校验与残留清理，不核对状态、不写分发。
pub fn open(data_dir: impl Into<PathBuf>) -> AppResult<Workspace> {
    let paths = AppPaths::new(data_dir);
    paths.ensure_dirs()?;
    logging::init(&paths.logs_dir)?;
    let conn = db::open(&paths.db_path)?;

    let targets = repo::list_targets(&conn)?;
    paths::validate_library_placement(
        &paths.library_dir,
        targets
            .iter()
            .map(|target| PathBuf::from(&target.skills_dir)),
    )?;

    fsops::cleanup_temp_dirs(&paths.library_dir);
    for target in &targets {
        fsops::cleanup_temp_dirs(Path::new(&target.skills_dir));
    }

    Ok(Workspace { paths, conn })
}
