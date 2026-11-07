use std::fs;

use skillport_lib::core::model::{LinkMode, Target};
use skillport_lib::core::paths::AppPaths;
use skillport_lib::core::{db, repo, util};
use skillport_lib::state::AppState;
use tempfile::TempDir;

#[test]
fn initialize_creates_layout_cleans_temps_and_reconciles() {
    let temp = TempDir::new().expect("temp");
    let data = temp.path().join("data");
    let paths = AppPaths::new(&data);
    paths.ensure_dirs().expect("ensure");
    let leftover = paths.library_dir.join(".skillport-tmp-leftover");
    fs::create_dir_all(&leftover).expect("leftover");

    let state = AppState::initialize(data).expect("initialize");

    assert!(state.paths.db_path.is_file());
    assert!(!leftover.exists(), "启动时必须清理临时残留");
    assert!(state.paths.logs_dir.join("skillport.log").is_file());
    let conn = state.db.lock().expect("db");
    assert!(repo::list_skills(&conn).expect("skills").is_empty());
}

#[test]
fn initialize_rejects_library_inside_target_dir() {
    let temp = TempDir::new().expect("temp");
    let data = temp.path().join("data");
    let paths = AppPaths::new(&data);
    paths.ensure_dirs().expect("ensure");
    let conn = db::open(&paths.db_path).expect("db");
    let now = util::now_iso();
    let target = Target {
        id: util::new_id(),
        slug: "bad-target".to_string(),
        name: "Bad".to_string(),
        kind: "skills_dir".to_string(),
        skills_dir: data.display().to_string(),
        link_mode: LinkMode::Symlink,
        enabled: true,
        detected: false,
        created_at: now.clone(),
        updated_at: now,
    };
    repo::insert_target(&conn, &target).expect("target");
    drop(conn);

    let error = match AppState::initialize(data) {
        Ok(_) => panic!("库目录不能位于目标目录内"),
        Err(error) => error,
    };
    assert_eq!(error.code(), "InvalidInput");
}
