//! SQLite 连接、PRAGMA 与只追加迁移。

use std::path::Path;
use std::time::Duration;

use rusqlite::Connection;
use rusqlite_migration::{Migrations, M};

use super::error::AppResult;

/// v1 结构。迁移只追加，历史条目永不修改。
const SCHEMA_V1: &str = r#"
CREATE TABLE targets (
  id           TEXT PRIMARY KEY,
  slug         TEXT NOT NULL UNIQUE,
  name         TEXT NOT NULL,
  kind         TEXT NOT NULL DEFAULT 'skills_dir',
  skills_dir   TEXT NOT NULL UNIQUE,
  link_mode    TEXT NOT NULL DEFAULT 'symlink'
               CHECK (link_mode IN ('symlink', 'copy')),
  enabled      INTEGER NOT NULL DEFAULT 1,
  detected     INTEGER NOT NULL DEFAULT 0,
  created_at   TEXT NOT NULL,
  updated_at   TEXT NOT NULL
);

CREATE TABLE skills (
  id            TEXT PRIMARY KEY,
  slug          TEXT NOT NULL UNIQUE,
  name          TEXT NOT NULL,
  description   TEXT NOT NULL DEFAULT '',
  source_kind   TEXT NOT NULL DEFAULT 'local'
                CHECK (source_kind IN ('local', 'import', 'git')),
  source_ref    TEXT,
  content_hash  TEXT NOT NULL,
  created_at    TEXT NOT NULL,
  updated_at    TEXT NOT NULL
);

CREATE TABLE installs (
  id            INTEGER PRIMARY KEY AUTOINCREMENT,
  skill_id      TEXT NOT NULL REFERENCES skills(id) ON DELETE CASCADE,
  target_id     TEXT NOT NULL REFERENCES targets(id) ON DELETE CASCADE,
  mode          TEXT NOT NULL CHECK (mode IN ('symlink', 'copy')),
  actual_kind   TEXT,
  status        TEXT NOT NULL DEFAULT 'pending'
                CHECK (status IN ('pending','ok','stale','missing','conflict','error')),
  target_path   TEXT NOT NULL,
  linked_hash   TEXT,
  last_error    TEXT,
  last_checked  TEXT,
  last_synced   TEXT,
  UNIQUE (skill_id, target_id)
);

CREATE TABLE activity (
  id         INTEGER PRIMARY KEY AUTOINCREMENT,
  ts         TEXT NOT NULL,
  action     TEXT NOT NULL,
  skill_id   TEXT,
  target_id  TEXT,
  result     TEXT NOT NULL,
  detail     TEXT
);

CREATE TABLE settings (
  key   TEXT PRIMARY KEY,
  value TEXT NOT NULL
);

CREATE INDEX idx_installs_skill ON installs(skill_id);
CREATE INDEX idx_installs_target ON installs(target_id);
CREATE INDEX idx_activity_ts ON activity(ts);
"#;

/// v2：冲突忽略标记、活动日志中的显示名、撤销记录。
const SCHEMA_V2: &str = r#"
ALTER TABLE installs ADD COLUMN ignored INTEGER NOT NULL DEFAULT 0;
ALTER TABLE activity ADD COLUMN skill_name TEXT;
ALTER TABLE activity ADD COLUMN target_name TEXT;

CREATE TABLE undo (
  id       TEXT PRIMARY KEY,       -- 关联的 activity id
  ts       TEXT NOT NULL,
  kind     TEXT NOT NULL,          -- skill-delete | sync
  payload  TEXT NOT NULL
);

CREATE INDEX idx_undo_ts ON undo(ts);
"#;

/// v3：技能分类。这是 SkillPort 自己的归类口径，只存 DB，绝不写回 `SKILL.md`。
const SCHEMA_V3: &str = r#"
ALTER TABLE skills ADD COLUMN category TEXT;
"#;

fn migrations() -> Migrations<'static> {
    Migrations::new(vec![M::up(SCHEMA_V1), M::up(SCHEMA_V2), M::up(SCHEMA_V3)])
}

fn configure(conn: &Connection) -> AppResult<()> {
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.pragma_update(None, "foreign_keys", "ON")?;
    conn.busy_timeout(Duration::from_secs(5))?;
    Ok(())
}

/// 打开数据库文件并执行迁移。
pub fn open(path: &Path) -> AppResult<Connection> {
    let mut conn = Connection::open(path)?;
    configure(&conn)?;
    migrations().to_latest(&mut conn)?;
    Ok(conn)
}

/// 内存数据库，仅测试使用。
pub fn open_in_memory() -> AppResult<Connection> {
    let mut conn = Connection::open_in_memory()?;
    conn.pragma_update(None, "foreign_keys", "ON")?;
    migrations().to_latest(&mut conn)?;
    Ok(conn)
}
