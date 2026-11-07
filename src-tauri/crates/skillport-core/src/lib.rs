//! 业务核心：与 Tauri 无关，可独立单元测试；桌面端与 CLI 共用。
//!
//! - 文件系统写入全部经过 `paths` 的守卫与 `sync::delete_owned_*` 的归属校验。
//! - `installs` 存期望，`status` 只是可重建缓存。

pub mod db;
pub mod error;
pub mod frontmatter;
pub mod fsops;
pub mod hash;
pub mod icons;
pub mod library;
pub mod logging;
pub mod model;
pub mod paths;
pub mod plan;
pub mod repo;
pub mod settings;
pub mod sync;
pub mod target;
pub mod undo;
pub mod util;
pub mod workspace;
