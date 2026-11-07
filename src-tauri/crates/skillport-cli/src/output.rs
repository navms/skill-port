//! 输出：默认是人类可读文本，`--json` 输出稳定结构供其他 agent 解析。

use skillport_core::error::AppError;
use skillport_core::model::ImportReport;
use skillport_core::paths::AppPaths;

/// 打印导入报告。
///
/// JSON 顶层字段：`schema`、`ok`、`dataDir`、`libraryDir`、`imported`、`skipped`、`items`。
/// `items[].errorCode` 与 `AGENTS.md` §8.1 的错误码一致。
pub fn print_report(paths: &AppPaths, report: &ImportReport, json: bool) {
    if json {
        let payload = serde_json::json!({
            "schema": 1,
            "ok": report.skipped == 0,
            "dataDir": paths.data_dir.display().to_string(),
            "libraryDir": paths.library_dir.display().to_string(),
            "imported": report.imported,
            "skipped": report.skipped,
            "items": report.items,
        });
        println!("{}", to_pretty_json(&payload));
        return;
    }

    if report.imported > 0 {
        println!("已导入 {} 个技能：", report.imported);
        for item in report.items.iter().filter(|item| item.ok) {
            let slug = item.slug.as_deref().unwrap_or("-");
            println!("  {slug}  <-  {}", item.path);
        }
    }

    let rejected: Vec<_> = report.items.iter().filter(|item| !item.ok).collect();
    if !rejected.is_empty() {
        println!("拒绝 {} 个：", rejected.len());
        for item in rejected {
            let code = item
                .error_code
                .map(|code| format!("{code:?}"))
                .unwrap_or_else(|| "-".to_string());
            let detail = item.detail.as_deref().unwrap_or("");
            println!("  {}  [{code}] {detail}", item.path);
        }
    }

    println!("数据目录：{}", paths.data_dir.display());
    println!("库目录：{}", paths.library_dir.display());
}

/// 打印致命错误；`--json` 时同样输出 JSON（`ok: false` + `error`）。
pub fn print_error(error: &AppError, json: bool) {
    if json {
        let payload = serde_json::json!({
            "schema": 1,
            "ok": false,
            "error": {
                "code": error.code(),
                "message": error.to_string(),
                "detail": error.detail(),
            },
        });
        println!("{}", to_pretty_json(&payload));
    } else {
        eprintln!("错误：{error}");
    }
}

fn to_pretty_json(value: &serde_json::Value) -> String {
    serde_json::to_string_pretty(value).unwrap_or_else(|_| "{}".to_string())
}
