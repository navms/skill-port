//! 小工具：时间、ID、锁。

use std::sync::{Mutex, MutexGuard};

use super::error::{AppError, AppResult};

/// 当前 UTC 时间，ISO-8601（RFC 3339）。
pub fn now_iso() -> String {
    time::OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string())
}

/// `hours` 小时之前的 UTC 时间，ISO-8601（RFC 3339）。
pub fn iso_before_hours(hours: i64) -> String {
    (time::OffsetDateTime::now_utc() - time::Duration::hours(hours))
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string())
}

/// 生成 ULID 字符串。
pub fn new_id() -> String {
    ulid::Ulid::new().to_string()
}

/// 获取互斥锁；中毒时转为可读错误，不 panic。
pub fn lock<T>(mutex: &Mutex<T>) -> AppResult<MutexGuard<'_, T>> {
    mutex
        .lock()
        .map_err(|_| AppError::Internal("内部状态锁已中毒，请重启应用".to_string()))
}

/// 生成适合备份目录名的紧凑时间戳，例如 `20261106T214500Z`。
pub fn compact_stamp() -> String {
    let now = time::OffsetDateTime::now_utc();
    format!(
        "{:04}{:02}{:02}T{:02}{:02}{:02}Z",
        now.year(),
        u8::from(now.month()),
        now.day(),
        now.hour(),
        now.minute(),
        now.second()
    )
}
