//! Tauri 管理的进程内状态：数据库连接、路径、同步计划。

use std::path::PathBuf;
use std::sync::Mutex;

use rusqlite::Connection;

use crate::core::error::AppResult;
use crate::core::paths::AppPaths;
use crate::core::plan::PlanStore;
use crate::core::{icons, logging, repo, sync, workspace};

pub struct AppState {
    pub paths: AppPaths,
    pub db: Mutex<Connection>,
    pub plans: PlanStore,
}

impl AppState {
    /// 初始化数据目录、数据库，校验库目录位置，清理临时残留并核对一次。
    pub fn initialize(data_dir: PathBuf) -> AppResult<Self> {
        let workspace = workspace::open(data_dir)?;
        let paths = workspace.paths;
        let conn = workspace.conn;

        icons::ensure_defaults(&paths)?;
        logging::write(
            &paths.logs_dir,
            "info",
            "startup",
            Some(&format!("data_dir={}", paths.data_dir.display())),
        );
        repo::prune_undo(&conn)?;
        sync::reconcile(&conn, &paths)?;

        Ok(Self {
            paths,
            db: Mutex::new(conn),
            plans: PlanStore::new(),
        })
    }
}
