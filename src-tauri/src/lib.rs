//! SkillPort Tauri 后端入口。
//!
//! 启动时：解析数据目录 → 打开数据库并迁移 → 校验库目录不在任何目标目录内
//! → 清理 `.skillport-tmp-*` 残留 → 核对一次状态。

pub mod commands;
pub mod core;
pub mod state;

use std::path::PathBuf;

use tauri::Manager;

use crate::core::error::{AppError, AppResult};
use crate::core::paths;

/// 构建并运行桌面应用。
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            // 主窗口在配置里是 `create: false`，这里手动创建：
            // macOS 的交通灯位置只能通过 builder 设置，配置文件没有对应字段。
            create_main_window(app)?;
            let data_dir = resolve_data_dir(app.handle())?;
            let state = state::AppState::initialize(data_dir)?;
            app.manage(state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::library::library_list,
            commands::library::library_get,
            commands::library::library_import,
            commands::library::library_update,
            commands::library::library_delete,
            commands::library::library_reveal,
            commands::targets::targets_list,
            commands::targets::target_detect_suggestions,
            commands::targets::target_create,
            commands::targets::target_update,
            commands::targets::target_delete,
            commands::targets::target_pick_dir,
            commands::targets::target_icon_list,
            commands::targets::target_icon_reveal,
            commands::sync::installs_set,
            commands::sync::installs_resolve,
            commands::sync::reconcile,
            commands::sync::sync_plan,
            commands::sync::sync_apply,
            commands::sync::sync_cancel,
            commands::sync::activity_list,
            commands::sync::activity_undo,
            commands::settings::settings_get,
            commands::settings::settings_set,
        ])
        .run(tauri::generate_context!())
        .expect("SkillPort 启动失败：请检查应用数据目录权限与日志");
}

/// 按 `tauri.conf.json` 里的窗口配置创建主窗口。
///
/// macOS 用 Overlay 标题栏时，交通灯默认贴在角落；这里显式放到标题行（44px）的垂直中心，
/// 与 Claude / VS Code 的观感一致。`x` 是左边距，`y` 是交通灯上方的留白（AppKit 按钮高 14px，
/// 按钮上沿 = y - 9），因此 44px 行高居中取 `y = 24`。
fn create_main_window(app: &tauri::App) -> tauri::Result<()> {
    let Some(window_config) = app.config().app.windows.first() else {
        return Ok(());
    };
    let mut builder = tauri::WebviewWindowBuilder::from_config(app.handle(), window_config)?;
    #[cfg(target_os = "macos")]
    {
        builder = builder.traffic_light_position(tauri::LogicalPosition::new(16.0, 24.0));
    }
    builder.build()?;
    Ok(())
}

/// 读取应用数据目录；`SKILLPORT_DATA_DIR` 优先。
fn resolve_data_dir(app: &tauri::AppHandle) -> AppResult<PathBuf> {
    if let Some(dir) = paths::env_data_dir() {
        return Ok(dir);
    }
    app.path()
        .app_data_dir()
        .map_err(|error| AppError::Internal(format!("无法获取应用数据目录：{error}")))
}
