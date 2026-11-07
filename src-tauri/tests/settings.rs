mod common;

use common::Fixture;
use skillport_lib::core::model::{LinkMode, SettingsPatch};
use skillport_lib::core::settings;

#[test]
fn defaults_are_sensible() {
    let fixture = Fixture::new();
    let snapshot = settings::snapshot(&fixture.conn, &fixture.paths).expect("get");
    assert_eq!(snapshot.default_link_mode, LinkMode::Symlink);
    assert_eq!(snapshot.backup_retention, 20);
    assert_eq!(
        snapshot.library_path,
        fixture.paths.library_dir.display().to_string()
    );
    assert_eq!(
        snapshot.data_dir,
        fixture.paths.data_dir.display().to_string()
    );
}

#[test]
fn rejects_invalid_values() {
    let fixture = Fixture::new();
    for patch in [
        SettingsPatch {
            backup_retention: Some(0),
            ..Default::default()
        },
        SettingsPatch {
            backup_retention: Some(1000),
            ..Default::default()
        },
        SettingsPatch {
            library_path: Some("/tmp/elsewhere".to_string()),
            ..Default::default()
        },
        SettingsPatch {
            data_dir: Some("/tmp/elsewhere".to_string()),
            ..Default::default()
        },
    ] {
        assert!(settings::update(&fixture.conn, &fixture.paths, &patch).is_err());
    }
}

#[test]
fn persists_valid_values() {
    let fixture = Fixture::new();
    let updated = settings::update(
        &fixture.conn,
        &fixture.paths,
        &SettingsPatch {
            default_link_mode: Some(LinkMode::Copy),
            backup_retention: Some(5),
            ..Default::default()
        },
    )
    .expect("update");
    assert_eq!(updated.default_link_mode, LinkMode::Copy);
    assert_eq!(updated.backup_retention, 5);
    let reloaded = settings::snapshot(&fixture.conn, &fixture.paths).expect("get");
    assert_eq!(reloaded.default_link_mode, LinkMode::Copy);
    assert_eq!(reloaded.backup_retention, 5);
}

#[test]
fn read_only_paths_matching_current_values_are_accepted() {
    let fixture = Fixture::new();
    let patch = SettingsPatch {
        library_path: Some(fixture.paths.library_dir.display().to_string()),
        data_dir: Some(fixture.paths.data_dir.display().to_string()),
        ..Default::default()
    };
    assert!(settings::update(&fixture.conn, &fixture.paths, &patch).is_ok());
}
