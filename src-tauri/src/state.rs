//! Tauri 管理的进程内状态：数据库连接、路径、同步计划。

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use rusqlite::Connection;

use crate::core::error::AppResult;
use crate::core::paths::AppPaths;
use crate::core::plan::PlanStore;
use crate::core::{db, fsops, icons, logging, paths, repo, sync};

pub struct AppState {
    pub paths: AppPaths,
    pub db: Mutex<Connection>,
    pub plans: PlanStore,
}

impl AppState {
    /// 初始化数据目录、数据库，校验库目录位置，清理临时残留并核对一次。
    pub fn initialize(data_dir: PathBuf) -> AppResult<Self> {
        let paths = AppPaths::new(data_dir);
        paths.ensure_dirs()?;
        icons::ensure_defaults(&paths)?;
        logging::init(&paths.logs_dir)?;
        logging::write(
            &paths.logs_dir,
            "info",
            "startup",
            Some(&format!("data_dir={}", paths.data_dir.display())),
        );
        let conn = db::open(&paths.db_path)?;
        repo::prune_undo(&conn)?;

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

        sync::reconcile(&conn, &paths)?;

        Ok(Self {
            paths,
            db: Mutex::new(conn),
            plans: PlanStore::new(),
        })
    }
}
