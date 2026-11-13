#![forbid(unsafe_code)]
//! `auxbridge-backup` — 一致性備份與驗證（`SHARED_STANDARDS` §8.1）。
//!
//! 為何需要：資料庫備份此前只有 `db.rs` 內的 VACUUM INTO 機制與測試，沒有可運維的
//! 命令入口（無保留策略、無雜湊清單、無還原前驗證）。本工具補上該層。
//!
//! 設計要點
//! - 以**唯讀**連線對資料庫執行 `VACUUM INTO`，可在伺服器執行中取得一致性快照（WAL 內容一併包含）。
//! - 快照產出後立即做兩道驗證：`PRAGMA integrity_check`（實體完整性）與 `PRAGMA foreign_key_check`（參照完整性）。
//! - 產生 SHA-256 校驗檔與 JSONL 清單（可稽核、可驗證、可用於離線保存）。
//! - 保留策略：僅保留最近 N 份（預設 14），其餘連同 `.sha256` 一併刪除。
//! - `--verify <file>`：驗證既有備份之校驗碼與完整性（還原演練前置條件）。
//!
//! 用法
//! ```text
//! auxbridge-backup [--db <path>] [--out <dir>] [--keep <n>]
//! auxbridge-backup --verify <backup.db>
//! ```
//! DB 來源優先序：`--db` > `AUXBRIDGE_DB_URL`（去 `sqlite://` 前綴與查詢串）> `data/auxbridge.db`。

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use sqlx::sqlite::SqlitePoolOptions;

const DEFAULT_DB: &str = "data/auxbridge.db";
const DEFAULT_OUT: &str = "backups";
const DEFAULT_KEEP: usize = 14;
const PREFIX: &str = "auxbridge-";
const SUFFIX: &str = ".db";

fn main() -> ExitCode {
    // sqlx 需 async runtime；本工具是一次性命令，採 current-thread runtime 即可。
    let runtime = match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(runtime) => runtime,
        Err(err) => {
            eprintln!("錯誤：無法建立 async runtime：{err}");
            return ExitCode::FAILURE;
        }
    };
    match runtime.block_on(run()) {
        Ok(lines) => {
            for line in lines {
                println!("{line}");
            }
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("錯誤：{err}");
            ExitCode::FAILURE
        }
    }
}

async fn run() -> Result<Vec<String>, String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if let Some(target) = arg_value(&args, "--verify") {
        return verify(&PathBuf::from(target)).await;
    }
    let db = arg_value(&args, "--db")
        .map(PathBuf::from)
        .or_else(db_from_env)
        .unwrap_or_else(|| PathBuf::from(DEFAULT_DB));
    let out = PathBuf::from(arg_value(&args, "--out").unwrap_or_else(|| DEFAULT_OUT.to_string()));
    let keep = arg_value(&args, "--keep")
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(DEFAULT_KEEP);
    if keep == 0 {
        return Err("--keep 必須 >= 1（保留 0 份會在備份後立即刪除）".to_string());
    }
    backup(&db, &out, keep).await
}

fn arg_value(args: &[String], flag: &str) -> Option<String> {
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        if arg == flag {
            return iter.next().cloned();
        }
        if let Some(rest) = arg.strip_prefix(&format!("{flag}=")) {
            return Some(rest.to_string());
        }
    }
    None
}

fn db_from_env() -> Option<PathBuf> {
    let url = std::env::var("AUXBRIDGE_DB_URL").ok()?;
    let rest = url.strip_prefix("sqlite://").unwrap_or(&url);
    let path = rest.split(['?', '#']).next().unwrap_or(rest).trim();
    if path.is_empty() || path.starts_with(":memory:") {
        return None;
    }
    Some(PathBuf::from(path))
}

/// 執行一次備份；回傳要列印的摘要行。
async fn backup(db: &Path, out: &Path, keep: usize) -> Result<Vec<String>, String> {
    let db = absolutize(db);
    if !db.is_file() {
        return Err(format!(
            "找不到資料庫 {}（可用 --db 或 AUXBRIDGE_DB_URL 指定）",
            db.display()
        ));
    }
    std::fs::create_dir_all(out)
        .map_err(|e| format!("建立備份目錄 {} 失敗：{e}", out.display()))?;

    let stamp = chrono::Utc::now().format("%Y%m%dT%H%M%SZ").to_string();
    // 同一秒內重複執行時避免覆寫（VACUUM INTO 亦會拒絕已存在之目標）：加數列後綴。
    let mut target = absolutize(&out.join(format!("{PREFIX}{stamp}{SUFFIX}")));
    let mut seq = 2u32;
    while target.exists() {
        target = absolutize(&out.join(format!("{PREFIX}{stamp}-{seq}{SUFFIX}")));
        seq += 1;
        if seq > 1000 {
            return Err("同一秒內備份檔名已用盡（請稍後再試）".to_string());
        }
    }
    let url = format!("sqlite://{}?mode=ro", target_url_path(&db));

    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect(&url)
        .await
        .map_err(|e| format!("以唯讀模式開啟 {} 失敗：{e}", db.display()))?;
    let escaped = target.display().to_string().replace('\'', "''");
    sqlx::query(sqlx::AssertSqlSafe(format!("VACUUM INTO '{escaped}'")))
        .execute(&pool)
        .await
        .map_err(|e| format!("VACUUM INTO 失敗：{e}"))?;
    let src_rows = count_rows(&pool).await?;
    pool.close().await;

    let bytes = std::fs::metadata(&target)
        .map_err(|e| format!("讀取快照 {} 失敗：{e}", target.display()))?
        .len();
    let (integrity, fk_violations) = check_snapshot(&target).await?;
    if integrity != "ok" {
        return Err(format!("快照完整性檢查失敗（integrity_check={integrity}）"));
    }
    if fk_violations > 0 {
        return Err(format!(
            "快照有 {fk_violations} 筆外部索引違規（foreign_key_check）"
        ));
    }
    let digest = sha256_file(&target)?;
    std::fs::write(
        target.with_extension("db.sha256"),
        format!("{digest}  {}\n", file_name(&target)),
    )
    .map_err(|e| format!("寫入校驗檔失敗：{e}"))?;

    let entry = serde_json::json!({
        "file": file_name(&target),
        "bytes": bytes,
        "sha256": digest,
        "created_at": chrono::Utc::now().to_rfc3339(),
        "source": db.display().to_string(),
        "source_rows": src_rows,
        "integrity": integrity,
    });
    append_manifest(out, &entry.to_string())?;

    let removed = prune(out, keep)?;
    Ok(vec![
        format!("已備份：{}", target.display()),
        format!("  大小：{bytes} bytes｜SHA-256：{digest}"),
        format!("  來源：{}（{src_rows} 筆 transactions）", db.display()),
        format!("  保留：{keep} 份｜本次刪除舊檔：{removed}"),
    ])
}

/// 驗證既有備份：校驗碼比對＋完整性檢查。
async fn verify(path: &Path) -> Result<Vec<String>, String> {
    let path = absolutize(path);
    if !path.is_file() {
        return Err(format!("找不到備份檔 {}", path.display()));
    }
    let digest = sha256_file(&path)?;
    let checksum_file = path.with_extension("db.sha256");
    let recorded = std::fs::read_to_string(&checksum_file)
        .map_err(|e| format!("讀取校驗檔 {} 失敗：{e}", checksum_file.display()))?;
    let recorded = recorded.split_whitespace().next().unwrap_or("").to_string();
    if recorded.is_empty() {
        return Err("校驗檔為空（備份可能不完整）".to_string());
    }
    let (integrity, fk_violations) = check_snapshot(&path).await?;
    let mut lines = vec![
        format!("備份檔：{}", path.display()),
        format!("  SHA-256（實算）：{digest}"),
        format!("  SHA-256（記錄）：{recorded}"),
        format!("  integrity_check：{integrity}"),
        format!("  foreign_key_check：{fk_violations} 筆違規"),
    ];
    if digest != recorded {
        lines.push("  結果：FAIL（校驗碼不符，檔案可能已損壞或被竄改）".to_string());
        return Err(lines.join("\n"));
    }
    if integrity != "ok" || fk_violations > 0 {
        lines.push("  結果：FAIL（完整性檢查未通過）".to_string());
        return Err(lines.join("\n"));
    }
    lines.push("  結果：PASS（可用於還原演練）".to_string());
    Ok(lines)
}

async fn count_rows(pool: &sqlx::SqlitePool) -> Result<i64, String> {
    sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM transactions")
        .fetch_one(pool)
        .await
        .map_err(|e| format!("讀取 transactions 筆數失敗：{e}"))
}

/// 對快照做兩道完整性檢查，回傳 (integrity_check 結果, foreign_key 違規數)。
async fn check_snapshot(path: &Path) -> Result<(String, i64), String> {
    let url = format!("sqlite://{}?mode=ro", target_url_path(path));
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect(&url)
        .await
        .map_err(|e| format!("開啟快照 {} 失敗：{e}", path.display()))?;
    let integrity: String = sqlx::query_scalar("PRAGMA integrity_check")
        .fetch_one(&pool)
        .await
        .map_err(|e| format!("integrity_check 失敗：{e}"))?;
    let violations: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM pragma_foreign_key_check")
        .fetch_one(&pool)
        .await
        .map_err(|e| format!("foreign_key_check 失敗：{e}"))?;
    pool.close().await;
    Ok((integrity, violations))
}

fn append_manifest(out: &Path, line: &str) -> Result<(), String> {
    let manifest = out.join("manifest.jsonl");
    let mut text = std::fs::read_to_string(&manifest).unwrap_or_default();
    text.push_str(line);
    text.push('\n');
    std::fs::write(&manifest, text).map_err(|e| format!("寫入 {} 失敗：{e}", manifest.display()))
}

/// 保留最近 `keep` 份備份（依檔名時間排序），刪除其餘含 `.sha256`。
fn prune(out: &Path, keep: usize) -> Result<usize, String> {
    let mut backups: Vec<PathBuf> = std::fs::read_dir(out)
        .map_err(|e| format!("讀取備份目錄 {} 失敗：{e}", out.display()))?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with(PREFIX) && n.ends_with(SUFFIX))
        })
        .collect();
    backups.sort();
    let excess = backups.len().saturating_sub(keep);
    let mut removed = 0usize;
    for path in backups.into_iter().take(excess) {
        let checksum = path.with_extension("db.sha256");
        let _ = std::fs::remove_file(&checksum);
        std::fs::remove_file(&path)
            .map_err(|e| format!("刪除舊備份 {} 失敗：{e}", path.display()))?;
        removed += 1;
    }
    Ok(removed)
}

fn sha256_file(path: &Path) -> Result<String, String> {
    use sha2::{Digest, Sha256};
    let bytes = std::fs::read(path).map_err(|e| format!("讀取 {} 失敗：{e}", path.display()))?;
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    Ok(format!("{:x}", hasher.finalize()))
}

fn absolutize(path: &Path) -> PathBuf {
    if path.is_absolute() {
        return path.to_path_buf();
    }
    std::env::current_dir()
        .map(|cwd| cwd.join(path))
        .unwrap_or_else(|_| path.to_path_buf())
}

fn target_url_path(path: &Path) -> String {
    let text = path.display().to_string();
    text.strip_prefix(r"\\?\").unwrap_or(&text).to_string()
}

fn file_name(path: &Path) -> String {
    path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("backup.db")
        .to_string()
}
