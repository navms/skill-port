//! 数据库查询（技能、目标、期望记录、活动日志、设置）。

use rusqlite::{params, Connection, OptionalExtension, Row};

use super::error::{AppError, AppResult};
use super::model::*;

fn parse_link_mode(raw: &str) -> LinkMode {
    LinkMode::parse(raw).unwrap_or(LinkMode::Symlink)
}

fn parse_status(raw: &str) -> InstallStatus {
    InstallStatus::parse(raw).unwrap_or(InstallStatus::Error)
}

fn skill_from_row(row: &Row<'_>) -> rusqlite::Result<Skill> {
    let source_kind: String = row.get("source_kind")?;
    Ok(Skill {
        id: row.get("id")?,
        slug: row.get("slug")?,
        name: row.get("name")?,
        description: row.get("description")?,
        category: row.get("category")?,
        source_kind: SourceKind::parse(&source_kind).unwrap_or(SourceKind::Local),
        source_ref: row.get("source_ref")?,
        content_hash: row.get("content_hash")?,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
    })
}

fn target_from_row(row: &Row<'_>) -> rusqlite::Result<Target> {
    let link_mode: String = row.get("link_mode")?;
    Ok(Target {
        id: row.get("id")?,
        slug: row.get("slug")?,
        name: row.get("name")?,
        kind: row.get("kind")?,
        skills_dir: row.get("skills_dir")?,
        link_mode: parse_link_mode(&link_mode),
        enabled: row.get::<_, i64>("enabled")? != 0,
        detected: row.get::<_, i64>("detected")? != 0,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
    })
}

fn install_from_row(row: &Row<'_>) -> rusqlite::Result<Install> {
    let mode: String = row.get("mode")?;
    let status: String = row.get("status")?;
    let actual_kind: Option<String> = row.get("actual_kind")?;
    Ok(Install {
        id: row.get("id")?,
        skill_id: row.get("skill_id")?,
        target_id: row.get("target_id")?,
        mode: parse_link_mode(&mode),
        actual_kind: actual_kind.and_then(|value| ActualKind::parse(&value)),
        status: parse_status(&status),
        target_path: row.get("target_path")?,
        linked_hash: row.get("linked_hash")?,
        ignored: row.get::<_, i64>("ignored")? != 0,
        last_error: row.get("last_error")?,
        last_checked: row.get("last_checked")?,
        last_synced: row.get("last_synced")?,
    })
}

/// 所有 Install 查询共用的列清单。
const INSTALL_COLUMNS: &str = "id, skill_id, target_id, mode, actual_kind, status, target_path,
     linked_hash, ignored, last_error, last_checked, last_synced";

// ---------------------------------------------------------------- skills

pub fn insert_skill(conn: &Connection, skill: &Skill) -> AppResult<()> {
    conn.execute(
        "INSERT INTO skills
           (id, slug, name, description, category, source_kind, source_ref, content_hash,
            created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![
            skill.id,
            skill.slug,
            skill.name,
            skill.description,
            skill.category,
            skill.source_kind.as_str(),
            skill.source_ref,
            skill.content_hash,
            skill.created_at,
            skill.updated_at,
        ],
    )?;
    Ok(())
}

pub fn update_skill(conn: &Connection, skill: &Skill) -> AppResult<()> {
    conn.execute(
        "UPDATE skills
            SET name = ?2, description = ?3, category = ?4, content_hash = ?5,
                updated_at = ?6
          WHERE id = ?1",
        params![
            skill.id,
            skill.name,
            skill.description,
            skill.category,
            skill.content_hash,
            skill.updated_at,
        ],
    )?;
    Ok(())
}

pub fn get_skill(conn: &Connection, id: &str) -> AppResult<Skill> {
    conn.query_row(
        "SELECT id, slug, name, description, category, source_kind, source_ref,
                content_hash, created_at, updated_at
           FROM skills WHERE id = ?1",
        params![id],
        skill_from_row,
    )
    .optional()?
    .ok_or_else(|| AppError::NotFound(format!("技能 {id}")))
}

pub fn find_skill_by_slug(conn: &Connection, slug: &str) -> AppResult<Option<Skill>> {
    Ok(conn
        .query_row(
            "SELECT id, slug, name, description, category, source_kind, source_ref,
                    content_hash, created_at, updated_at
               FROM skills WHERE slug = ?1",
            params![slug],
            skill_from_row,
        )
        .optional()?)
}

pub fn list_skills(conn: &Connection) -> AppResult<Vec<Skill>> {
    let mut stmt = conn.prepare(
        "SELECT id, slug, name, description, category, source_kind, source_ref,
                content_hash, created_at, updated_at
           FROM skills ORDER BY name COLLATE NOCASE ASC",
    )?;
    let rows = stmt.query_map([], skill_from_row)?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

pub fn all_slugs(conn: &Connection) -> AppResult<Vec<String>> {
    let mut stmt = conn.prepare("SELECT slug FROM skills")?;
    let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

pub fn delete_skill_row(conn: &Connection, id: &str) -> AppResult<()> {
    conn.execute("DELETE FROM skills WHERE id = ?1", params![id])?;
    Ok(())
}

// --------------------------------------------------------------- targets

pub fn insert_target(conn: &Connection, target: &Target) -> AppResult<()> {
    conn.execute(
        "INSERT INTO targets
           (id, slug, name, kind, skills_dir, link_mode, enabled, detected, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![
            target.id,
            target.slug,
            target.name,
            target.kind,
            target.skills_dir,
            target.link_mode.as_str(),
            i64::from(target.enabled),
            i64::from(target.detected),
            target.created_at,
            target.updated_at,
        ],
    )?;
    Ok(())
}

pub fn update_target_row(conn: &Connection, target: &Target) -> AppResult<()> {
    conn.execute(
        "UPDATE targets
            SET name = ?2, skills_dir = ?3, link_mode = ?4, enabled = ?5, updated_at = ?6
          WHERE id = ?1",
        params![
            target.id,
            target.name,
            target.skills_dir,
            target.link_mode.as_str(),
            i64::from(target.enabled),
            target.updated_at,
        ],
    )?;
    Ok(())
}

pub fn get_target(conn: &Connection, id: &str) -> AppResult<Target> {
    conn.query_row(
        "SELECT id, slug, name, kind, skills_dir, link_mode, enabled, detected,
                created_at, updated_at
           FROM targets WHERE id = ?1",
        params![id],
        target_from_row,
    )
    .optional()?
    .ok_or_else(|| AppError::NotFound(format!("目标 {id}")))
}

pub fn find_target_by_slug(conn: &Connection, slug: &str) -> AppResult<Option<Target>> {
    Ok(conn
        .query_row(
            "SELECT id, slug, name, kind, skills_dir, link_mode, enabled, detected,
                    created_at, updated_at
               FROM targets WHERE slug = ?1",
            params![slug],
            target_from_row,
        )
        .optional()?)
}

pub fn list_targets(conn: &Connection) -> AppResult<Vec<Target>> {
    let mut stmt = conn.prepare(
        "SELECT id, slug, name, kind, skills_dir, link_mode, enabled, detected,
                created_at, updated_at
           FROM targets ORDER BY name COLLATE NOCASE ASC",
    )?;
    let rows = stmt.query_map([], target_from_row)?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

pub fn delete_target_row(conn: &Connection, id: &str) -> AppResult<()> {
    conn.execute("DELETE FROM targets WHERE id = ?1", params![id])?;
    Ok(())
}

// -------------------------------------------------------------- installs

pub fn upsert_install(
    conn: &Connection,
    skill_id: &str,
    target_id: &str,
    mode: LinkMode,
    target_path: &str,
) -> AppResult<Install> {
    set_expectation(
        conn,
        skill_id,
        target_id,
        mode,
        target_path,
        &super::util::now_iso(),
    )
}

/// 写入期望（新建或更新模式/路径），状态置为 `pending` 等待同步。
pub fn set_expectation(
    conn: &Connection,
    skill_id: &str,
    target_id: &str,
    mode: LinkMode,
    target_path: &str,
    at: &str,
) -> AppResult<Install> {
    conn.execute(
        "INSERT INTO installs (skill_id, target_id, mode, status, target_path, last_checked)
         VALUES (?1, ?2, ?3, 'pending', ?4, ?5)
         ON CONFLICT(skill_id, target_id) DO UPDATE SET
           mode = excluded.mode,
           target_path = excluded.target_path,
           status = 'pending',
           actual_kind = NULL,
           linked_hash = NULL,
           last_error = NULL,
           last_checked = excluded.last_checked",
        params![skill_id, target_id, mode.as_str(), target_path, at],
    )?;
    get_install(conn, skill_id, target_id)?
        .ok_or_else(|| AppError::Internal("Install 写入后未能读回".to_string()))
}

pub fn set_ignored(
    conn: &Connection,
    skill_id: &str,
    target_id: &str,
    ignored: bool,
) -> AppResult<()> {
    conn.execute(
        "UPDATE installs SET ignored = ?3 WHERE skill_id = ?1 AND target_id = ?2",
        params![skill_id, target_id, i64::from(ignored)],
    )?;
    Ok(())
}

pub fn get_install(
    conn: &Connection,
    skill_id: &str,
    target_id: &str,
) -> AppResult<Option<Install>> {
    let sql =
        format!("SELECT {INSTALL_COLUMNS} FROM installs WHERE skill_id = ?1 AND target_id = ?2");
    Ok(conn
        .query_row(&sql, params![skill_id, target_id], install_from_row)
        .optional()?)
}

pub fn list_installs(conn: &Connection) -> AppResult<Vec<Install>> {
    let sql = format!("SELECT {INSTALL_COLUMNS} FROM installs ORDER BY skill_id, target_id");
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map([], install_from_row)?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

pub fn list_installs_for_skill(conn: &Connection, skill_id: &str) -> AppResult<Vec<Install>> {
    let sql =
        format!("SELECT {INSTALL_COLUMNS} FROM installs WHERE skill_id = ?1 ORDER BY target_id");
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(params![skill_id], install_from_row)?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

pub fn list_installs_for_target(conn: &Connection, target_id: &str) -> AppResult<Vec<Install>> {
    let sql =
        format!("SELECT {INSTALL_COLUMNS} FROM installs WHERE target_id = ?1 ORDER BY skill_id");
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(params![target_id], install_from_row)?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

/// 完整写回一条 Install（撤销时使用）；已存在则覆盖。
pub fn put_install(conn: &Connection, install: &Install) -> AppResult<()> {
    conn.execute(
        "INSERT INTO installs
           (id, skill_id, target_id, mode, actual_kind, status, target_path,
            linked_hash, ignored, last_error, last_checked, last_synced)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
         ON CONFLICT(skill_id, target_id) DO UPDATE SET
           mode = excluded.mode,
           actual_kind = excluded.actual_kind,
           status = excluded.status,
           target_path = excluded.target_path,
           linked_hash = excluded.linked_hash,
           ignored = excluded.ignored,
           last_error = excluded.last_error,
           last_checked = excluded.last_checked,
           last_synced = excluded.last_synced",
        params![
            install.id,
            install.skill_id,
            install.target_id,
            install.mode.as_str(),
            install.actual_kind.map(|kind| kind.as_str()),
            install.status.as_str(),
            install.target_path,
            install.linked_hash,
            i64::from(install.ignored),
            install.last_error,
            install.last_checked,
            install.last_synced,
        ],
    )?;
    Ok(())
}

pub fn delete_install(conn: &Connection, skill_id: &str, target_id: &str) -> AppResult<()> {
    conn.execute(
        "DELETE FROM installs WHERE skill_id = ?1 AND target_id = ?2",
        params![skill_id, target_id],
    )?;
    Ok(())
}

pub fn record_observation(
    conn: &Connection,
    install_id: i64,
    status: InstallStatus,
    actual_kind: Option<ActualKind>,
    linked_hash: Option<&str>,
    last_error: Option<&str>,
    checked_at: &str,
) -> AppResult<()> {
    conn.execute(
        "UPDATE installs
            SET status = ?2, actual_kind = ?3, linked_hash = ?4, last_error = ?5, last_checked = ?6
          WHERE id = ?1",
        params![
            install_id,
            status.as_str(),
            actual_kind.map(|kind| kind.as_str()),
            linked_hash,
            last_error,
            checked_at,
        ],
    )?;
    Ok(())
}

pub fn mark_synced(
    conn: &Connection,
    skill_id: &str,
    target_id: &str,
    actual_kind: ActualKind,
    linked_hash: &str,
    at: &str,
) -> AppResult<()> {
    conn.execute(
        "UPDATE installs
            SET status = 'ok', actual_kind = ?3, linked_hash = ?4, ignored = 0,
                last_error = NULL, last_checked = ?5, last_synced = ?5
          WHERE skill_id = ?1 AND target_id = ?2",
        params![skill_id, target_id, actual_kind.as_str(), linked_hash, at],
    )?;
    Ok(())
}

pub fn mark_error(
    conn: &Connection,
    skill_id: &str,
    target_id: &str,
    message: &str,
    at: &str,
) -> AppResult<()> {
    conn.execute(
        "UPDATE installs
            SET status = 'error', ignored = 0, last_error = ?3, last_checked = ?4
          WHERE skill_id = ?1 AND target_id = ?2",
        params![skill_id, target_id, message, at],
    )?;
    Ok(())
}

/// 库目录内容变化后更新权威 hash。
pub fn update_skill_hash(
    conn: &Connection,
    skill_id: &str,
    content_hash: &str,
    at: &str,
) -> AppResult<()> {
    conn.execute(
        "UPDATE skills SET content_hash = ?2, updated_at = ?3 WHERE id = ?1",
        params![skill_id, content_hash, at],
    )?;
    Ok(())
}

/// 目标路径变化后，刷新该目标下所有期望记录的 `target_path`。
pub fn retarget_installs(
    conn: &Connection,
    target_id: &str,
    skills_dir: &str,
    at: &str,
) -> AppResult<()> {
    let installs = list_installs_for_target(conn, target_id)?;
    for install in installs {
        let skill = get_skill(conn, &install.skill_id)?;
        let path = std::path::Path::new(skills_dir).join(&skill.slug);
        conn.execute(
            "UPDATE installs SET target_path = ?2, status = 'pending', last_checked = ?3
              WHERE id = ?1",
            params![install.id, path.to_string_lossy(), at],
        )?;
    }
    Ok(())
}

pub fn installed_slugs_for_target(conn: &Connection, target_id: &str) -> AppResult<Vec<String>> {
    let mut stmt = conn.prepare(
        "SELECT s.slug FROM installs i JOIN skills s ON s.id = i.skill_id
          WHERE i.target_id = ?1",
    )?;
    let rows = stmt.query_map(params![target_id], |row| row.get::<_, String>(0))?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

// -------------------------------------------------------------- activity

/// 一条待写入的活动日志。
pub struct ActivityRecord {
    pub action: String,
    pub skill_id: Option<String>,
    pub skill_name: Option<String>,
    pub target_id: Option<String>,
    pub target_name: Option<String>,
    pub result: String,
    pub detail: Option<String>,
}

impl ActivityRecord {
    pub fn ok(action: &str) -> Self {
        Self {
            action: action.to_string(),
            skill_id: None,
            skill_name: None,
            target_id: None,
            target_name: None,
            result: "ok".to_string(),
            detail: None,
        }
    }

    pub fn skill(mut self, skill: &Skill) -> Self {
        self.skill_id = Some(skill.id.clone());
        self.skill_name = Some(skill.slug.clone());
        self
    }

    pub fn target(mut self, target: &Target) -> Self {
        self.target_id = Some(target.id.clone());
        self.target_name = Some(target.name.clone());
        self
    }

    pub fn detail(mut self, detail: &str) -> Self {
        self.detail = Some(detail.to_string());
        self
    }

    pub fn result(mut self, result: &str) -> Self {
        self.result = result.to_string();
        self
    }
}

/// 写入活动日志并返回自增 id（撤销记录用它作为键）。
pub fn log_activity(conn: &Connection, record: &ActivityRecord) -> AppResult<i64> {
    conn.execute(
        "INSERT INTO activity
           (ts, action, skill_id, skill_name, target_id, target_name, result, detail)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            super::util::now_iso(),
            record.action.as_str(),
            record.skill_id.as_deref(),
            record.skill_name.as_deref(),
            record.target_id.as_deref(),
            record.target_name.as_deref(),
            record.result.as_str(),
            record.detail.as_deref()
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn list_activity(
    conn: &Connection,
    limit: u32,
    cursor: Option<i64>,
) -> AppResult<ActivityPage> {
    let limit = i64::from(limit.clamp(1, 200));
    let mut stmt = conn.prepare(
        "SELECT id, ts, action, skill_id, skill_name, target_id, target_name, result, detail
           FROM activity
          WHERE (?1 IS NULL OR id < ?1)
          ORDER BY id DESC
          LIMIT ?2",
    )?;
    let rows = stmt.query_map(params![cursor, limit], |row| {
        Ok(ActivityEntry {
            id: row.get(0)?,
            ts: row.get(1)?,
            action: row.get(2)?,
            skill_id: row.get(3)?,
            skill_name: row.get(4)?,
            target_id: row.get(5)?,
            target_name: row.get(6)?,
            result: row.get(7)?,
            detail: row.get(8)?,
        })
    })?;
    let items = rows.collect::<rusqlite::Result<Vec<_>>>()?;
    let next_cursor = if items.len() as i64 == limit {
        items.last().map(|entry| entry.id.to_string())
    } else {
        None
    };
    Ok(ActivityPage { items, next_cursor })
}

// -------------------------------------------------------------- undo

pub fn put_undo(conn: &Connection, id: &str, kind: &str, payload: &str) -> AppResult<()> {
    conn.execute(
        "INSERT INTO undo (id, ts, kind, payload) VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT(id) DO UPDATE SET ts = excluded.ts, kind = excluded.kind,
                                       payload = excluded.payload",
        params![id, super::util::now_iso(), kind, payload],
    )?;
    Ok(())
}

pub fn take_undo(conn: &Connection, id: &str) -> AppResult<Option<(String, String)>> {
    let row: Option<(String, String)> = conn
        .query_row(
            "SELECT kind, payload FROM undo WHERE id = ?1",
            params![id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()?;
    if row.is_some() {
        conn.execute("DELETE FROM undo WHERE id = ?1", params![id])?;
    }
    Ok(row)
}

/// 清理超过 24 小时的撤销记录（文件系统可能已变化，过期不再可靠）。
pub fn prune_undo(conn: &Connection) -> AppResult<()> {
    conn.execute(
        "DELETE FROM undo WHERE ts < ?1",
        params![super::util::iso_before_hours(24)],
    )?;
    Ok(())
}

// -------------------------------------------------------------- settings

pub fn get_setting(conn: &Connection, key: &str) -> AppResult<Option<String>> {
    Ok(conn
        .query_row(
            "SELECT value FROM settings WHERE key = ?1",
            params![key],
            |row| row.get(0),
        )
        .optional()?)
}

pub fn set_setting(conn: &Connection, key: &str, value: &str) -> AppResult<()> {
    conn.execute(
        "INSERT INTO settings (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![key, value],
    )?;
    Ok(())
}
