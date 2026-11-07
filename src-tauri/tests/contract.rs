//! IPC 形状守卫：确保 Rust 序列化与 `src/lib/types.ts` 的字段名一致。

use skillport_lib::core::model::{
    ActualKind, AppSettings, ImportErrorCode, ImportReport, ImportResultItem, Install,
    InstallStatus, LinkMode, PlanAction, ReconcileReport, SkillDetail, SkillSummary, SourceKind,
    SyncActionKind, SyncPlan, SyncProgress, SyncProgressStatus, SyncResult, SyncResultItem,
};

fn install() -> Install {
    Install {
        id: 1,
        skill_id: "skl".to_string(),
        target_id: "tgt".to_string(),
        mode: LinkMode::Symlink,
        actual_kind: Some(ActualKind::Symlink),
        status: InstallStatus::Ok,
        target_path: "/tmp/skill".to_string(),
        linked_hash: Some("hash".to_string()),
        ignored: false,
        last_error: None,
        last_checked: None,
        last_synced: None,
    }
}

fn summary() -> SkillSummary {
    SkillSummary {
        id: "skl".to_string(),
        slug: "demo".to_string(),
        name: "Demo".to_string(),
        description: "d".to_string(),
        category: Some("工具".to_string()),
        source_kind: SourceKind::Import,
        source_ref: None,
        content_hash: "hash".to_string(),
        size_bytes: 12,
        created_at: "2026-01-01T00:00:00Z".to_string(),
        updated_at: "2026-01-01T00:00:00Z".to_string(),
        installs: vec![install()],
    }
}

#[test]
fn install_uses_camel_case_and_ignored_flag() {
    let value = serde_json::to_value(install()).expect("json");
    assert_eq!(value["skillId"], "skl");
    assert_eq!(value["targetId"], "tgt");
    assert_eq!(value["actualKind"], "symlink");
    assert_eq!(value["targetPath"], "/tmp/skill");
    assert_eq!(value["linkedHash"], "hash");
    assert_eq!(value["ignored"], false);
    assert!(value.get("skill_id").is_none());
}

#[test]
fn skill_detail_flattens_summary() {
    let detail = SkillDetail {
        summary: summary(),
        path: "/tmp/demo".to_string(),
        skill_md: "---\nname: Demo\n---\n".to_string(),
    };
    let value = serde_json::to_value(detail).expect("json");
    assert_eq!(value["slug"], "demo");
    assert_eq!(value["category"], "工具");
    assert_eq!(value["sizeBytes"], 12);
    assert_eq!(value["skillMd"], "---\nname: Demo\n---\n");
    assert_eq!(value["path"], "/tmp/demo");
    assert!(value.get("summary").is_none());
}

#[test]
fn sync_shapes_match_frontend_contract() {
    let action = PlanAction {
        id: "skl:tgt".to_string(),
        skill_id: "skl".to_string(),
        skill_slug: "demo".to_string(),
        skill_name: "Demo".to_string(),
        target_id: "tgt".to_string(),
        target_name: "Claude Code".to_string(),
        target_path: "/tmp/skill".to_string(),
        mode: LinkMode::Copy,
        kind: SyncActionKind::Repair,
        state: InstallStatus::Stale,
        reason: Some("更新过期副本".to_string()),
    };
    let action_json = serde_json::to_value(&action).expect("action");
    assert_eq!(action_json["kind"], "repair");
    assert_eq!(action_json["state"], "stale");
    assert_eq!(action_json["skillSlug"], "demo");
    assert_eq!(action_json["targetName"], "Claude Code");

    let plan = SyncPlan {
        id: "plan".to_string(),
        created_at: "2026-01-01T00:00:00Z".to_string(),
        expires_at: "2026-01-01T00:05:00Z".to_string(),
        actions: vec![action],
    };
    let plan_json = serde_json::to_value(plan).expect("plan");
    assert_eq!(plan_json["createdAt"], "2026-01-01T00:00:00Z");
    assert_eq!(plan_json["expiresAt"], "2026-01-01T00:05:00Z");

    let progress = SyncProgress {
        plan_id: "plan".to_string(),
        completed: 1,
        total: 2,
        action_id: Some("skl:tgt".to_string()),
        status: SyncProgressStatus::Ok,
    };
    let progress_json = serde_json::to_value(progress).expect("progress");
    assert_eq!(progress_json["planId"], "plan");
    assert_eq!(progress_json["actionId"], "skl:tgt");
    assert_eq!(progress_json["status"], "ok");

    let result = SyncResult {
        plan_id: "plan".to_string(),
        applied: 1,
        failed: 0,
        cancelled: false,
        undo_id: Some("7".to_string()),
        items: vec![SyncResultItem {
            action_id: "skl:tgt".to_string(),
            ok: true,
            error_code: None,
            message: None,
        }],
    };
    let result_json = serde_json::to_value(result).expect("result");
    assert_eq!(result_json["undoId"], "7");
    assert_eq!(result_json["items"][0]["actionId"], "skl:tgt");
    assert_eq!(result_json["failed"], 0);
}

#[test]
fn report_and_settings_shapes_match_frontend_contract() {
    let report = ReconcileReport {
        checked: 3,
        changed: 1,
        issues: 2,
        checked_at: "2026-01-01T00:00:00Z".to_string(),
    };
    let report_json = serde_json::to_value(report).expect("report");
    assert_eq!(report_json["checked"], 3);
    assert_eq!(report_json["changed"], 1);
    assert_eq!(report_json["issues"], 2);
    assert_eq!(report_json["checkedAt"], "2026-01-01T00:00:00Z");

    let settings = AppSettings {
        default_link_mode: LinkMode::Copy,
        library_path: "/tmp/library".to_string(),
        data_dir: "/tmp/data".to_string(),
        backup_retention: 20,
    };
    let settings_json = serde_json::to_value(settings).expect("settings");
    assert_eq!(settings_json["defaultLinkMode"], "copy");
    assert_eq!(settings_json["libraryPath"], "/tmp/library");
    assert_eq!(settings_json["dataDir"], "/tmp/data");
    assert_eq!(settings_json["backupRetention"], 20);
}

#[test]
fn import_shapes_use_frontend_error_codes() {
    let report = ImportReport {
        items: vec![ImportResultItem {
            path: "/tmp/x".to_string(),
            slug: None,
            ok: false,
            error_code: Some(ImportErrorCode::DuplicateSlug),
            detail: Some("slug 已存在".to_string()),
        }],
        imported: 0,
        skipped: 1,
    };
    let value = serde_json::to_value(report).expect("import");
    assert_eq!(value["items"][0]["errorCode"], "DuplicateSlug");
    assert_eq!(value["items"][0]["ok"], false);
    assert_eq!(value["imported"], 0);
    assert_eq!(value["skipped"], 1);
}
