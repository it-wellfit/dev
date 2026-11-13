#![forbid(unsafe_code)]

//! `auxbridge-backup` 驗收測試（`SHARED_STANDARDS` §8.1：備份程序＋還原演練前置條件）。
//!
//! 覆蓋：建立備份、SHA-256 校驗檔、JSONL 清單、保留策略、驗證模式（正例與竄改負例）。

use std::path::{Path, PathBuf};
use std::process::Command;

use sqlx::sqlite::SqlitePoolOptions;

const BIN: &str = env!("CARGO_BIN_EXE_auxbridge-backup");

fn temp_dir(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("auxbridge-backup-{tag}-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&dir).expect("temp dir");
    dir
}

async fn seed_db(path: &Path) {
    let url = format!("sqlite://{}?mode=rwc", path.display());
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(platform::db::connect_options(&url).expect("connect options"))
        .await
        .expect("pool");
    platform::db::DatabaseManager::new(pool.clone())
        .initialize()
        .await
        .expect("migrations");
    sqlx::query(
        "INSERT INTO transactions (transaction_id, transaction_code, transaction_date, description, status) \
         VALUES ('t1', 'TX-1', '2026-11-01', 'backup me', 'POSTED')",
    )
    .execute(&pool)
    .await
    .expect("insert");
    pool.close().await;
}

fn run_backup(db: &Path, out: &Path, keep: &str) -> std::process::Output {
    Command::new(BIN)
        .args([
            "--db",
            &db.display().to_string(),
            "--out",
            &out.display().to_string(),
            "--keep",
            keep,
        ])
        .output()
        .expect("執行 auxbridge-backup")
}

fn backups_in(out: &Path) -> Vec<PathBuf> {
    let mut list: Vec<PathBuf> = std::fs::read_dir(out)
        .expect("backup dir")
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with("auxbridge-") && n.ends_with(".db"))
        })
        .collect();
    list.sort();
    list
}

#[tokio::test]
async fn backup_creates_checksum_manifest_and_verifies() {
    let dir = temp_dir("happy");
    let db = dir.join("src.db");
    seed_db(&db).await;
    let out = dir.join("backups");

    let first = run_backup(&db, &out, "14");
    assert!(
        first.status.success(),
        "備份失敗：{}",
        String::from_utf8_lossy(&first.stderr)
    );

    let backups = backups_in(&out);
    assert_eq!(backups.len(), 1, "應產生一份備份");
    let backup = &backups[0];
    let checksum = backup.with_extension("db.sha256");
    assert!(checksum.is_file(), "應產生 SHA-256 校驗檔");

    let manifest = std::fs::read_to_string(out.join("manifest.jsonl")).expect("manifest");
    assert_eq!(manifest.lines().count(), 1, "manifest 應有一筆紀錄");
    assert!(
        manifest.contains("\"integrity\":\"ok\""),
        "manifest 應記錄完整性：{manifest}"
    );
    assert!(manifest.contains("\"sha256\""), "manifest 應記錄 SHA-256");

    // 驗證模式：正例應 PASS。
    let verify = Command::new(BIN)
        .args(["--verify", &backup.display().to_string()])
        .output()
        .expect("執行 --verify");
    assert!(
        verify.status.success(),
        "備份驗證應通過：{}",
        String::from_utf8_lossy(&verify.stderr)
    );

    // 竄改備份位元組 → 驗證必須失敗（校驗碼不符）。
    let mut bytes = std::fs::read(backup).expect("read backup");
    let last = bytes.len() - 1;
    bytes[last] ^= 0xFF;
    std::fs::write(backup, &bytes).expect("tamper");
    let tampered = Command::new(BIN)
        .args(["--verify", &backup.display().to_string()])
        .output()
        .expect("執行 --verify（竄改後）");
    assert!(
        !tampered.status.success(),
        "竄改後之備份驗證必須失敗（不得假裝通過）"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[tokio::test]
async fn backup_retention_keeps_only_latest_n() {
    let dir = temp_dir("retention");
    let db = dir.join("src.db");
    seed_db(&db).await;
    let out = dir.join("backups");

    for keep in ["2", "2", "2"] {
        let output = run_backup(&db, &out, keep);
        assert!(
            output.status.success(),
            "備份失敗：{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let backups = backups_in(&out);
    assert_eq!(
        backups.len(),
        2,
        "--keep 2 應僅保留兩份（實得 {}）",
        backups.len()
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[tokio::test]
async fn backup_rejects_missing_database() {
    let dir = temp_dir("missing");
    let out = dir.join("backups");
    let output = run_backup(&dir.join("nope.db"), &out, "14");
    assert!(!output.status.success(), "不存在之 DB 必須失敗");
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    assert!(
        stderr.contains("找不到資料庫"),
        "錯誤訊息應說明找不到 DB：{stderr}"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[tokio::test]
async fn backup_rejects_zero_keep() {
    let dir = temp_dir("keep0");
    let db = dir.join("src.db");
    seed_db(&db).await;
    let out = dir.join("backups");
    let output = run_backup(&db, &out, "0");
    assert!(
        !output.status.success(),
        "--keep 0 必須被拒絕（否則備份立即被刪）"
    );
    let _ = std::fs::remove_dir_all(&dir);
}
