#![forbid(unsafe_code)]
//! SBOM 新鮮度閘（P1）：`docs/release/sbom/` 內每個提交的 CycloneDX SBOM 必須與
//! `Cargo.lock` 解析出的「可達依賴集」一致。
//!
//! 動機：SBOM 內含 timestamp／serialNumber，無法用位元組比對做漂移檢查；因此改為
//! **元件版本比對**——以 `cargo metadata` 的 resolve 圖（排除 dev-dependencies）
//! 計算每個 crate 的可達套件集，與 SBOM 的 `metadata.component` ＋ `components[]`
//! 比對。任何未重新產生的 SBOM（相依升級、新增或移除套件）都會在此失敗。
//!
//! 未安裝 `cargo-cyclonedx` 也能執行（本測試只讀提交檔與 `cargo metadata`）。

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::path::{Path, PathBuf};
use std::process::Command;

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("platform crate 應位於 repo 根之下")
        .to_path_buf()
}

/// 執行 `cargo metadata`（offline：不觸發網路，避免 CI 不穩）。
///
/// `--filter-platform` 使 resolve 圖只保留可建置於本目標平台的依賴，與
/// cargo-cyclonedx 的產出範圍一致（SBOM 的 `cdx:rustc:sbom:target:triple`）。
fn load_metadata(manifest: &Path) -> serde_json::Value {
    let output = Command::new("cargo")
        .args([
            "metadata",
            "--format-version",
            "1",
            "--offline",
            "--filter-platform",
            TARGET_TRIPLE,
            "--manifest-path",
        ])
        .arg(manifest)
        .output()
        .unwrap_or_else(|e| panic!("無法執行 cargo metadata（{}）：{e}", manifest.display()));
    assert!(
        output.status.success(),
        "cargo metadata 失敗（{}）：{}",
        manifest.display(),
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("cargo metadata 輸出應為合法 JSON")
}

/// SBOM 的目標平台（與 cargo-cyclonedx 產出的 `cdx:rustc:sbom:target:triple` 一致）。
const TARGET_TRIPLE: &str = "x86_64-pc-windows-msvc";

/// 從 resolve 圖計算 `root_id` 的可達套件集（排除 dev-kind 依賴）。
fn reachable_packages(meta: &serde_json::Value, root_id: &str) -> BTreeSet<String> {
    let mut labels: BTreeMap<&str, String> = BTreeMap::new();
    for pkg in meta["packages"].as_array().expect("packages 應為陣列") {
        let id = pkg["id"].as_str().expect("套件應有 id");
        let name = pkg["name"].as_str().expect("套件應有 name");
        let version = pkg["version"].as_str().expect("套件應有 version");
        labels.insert(id, format!("{name}@{version}"));
    }

    let nodes = meta["resolve"]["nodes"]
        .as_array()
        .expect("resolve.nodes 應為陣列");
    let mut edges: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for node in nodes {
        let id = node["id"].as_str().expect("node 應有 id");
        let mut deps = Vec::new();
        for dep in node["deps"].as_array().into_iter().flatten() {
            let pkg = dep["pkg"].as_str().expect("dep 應有 pkg");
            // dep_kinds 缺省（舊版 cargo）視為一般依賴；否則僅排除 kind == "dev"。
            let kinds = dep["dep_kinds"].as_array();
            let is_dev_only = match kinds {
                Some(kinds) if !kinds.is_empty() => {
                    kinds.iter().all(|k| k["kind"].as_str() == Some("dev"))
                }
                _ => false,
            };
            if !is_dev_only {
                deps.push(pkg);
            }
        }
        edges.insert(id, deps);
    }

    let mut seen: BTreeSet<String> = BTreeSet::new();
    let mut queue: VecDeque<&str> = VecDeque::new();
    queue.push_back(root_id);
    while let Some(id) = queue.pop_front() {
        if let Some(label) = labels.get(id) {
            seen.insert(label.clone());
        }
        if let Some(deps) = edges.get(id) {
            for dep in deps {
                if !seen.contains(labels.get(dep).map(String::as_str).unwrap_or("")) {
                    queue.push_back(dep);
                }
            }
        }
    }
    seen
}

/// 讀取 SBOM 的元件集（根元件 + components）。
fn sbom_components(path: &Path) -> BTreeSet<String> {
    let text = std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("讀取 {} 失敗：{e}", path.display()));
    let value: serde_json::Value =
        serde_json::from_str(&text).unwrap_or_else(|e| panic!("解析 {} 失敗：{e}", path.display()));
    assert_eq!(
        value["bomFormat"],
        "CycloneDX",
        "{} 非 CycloneDX",
        path.display()
    );
    assert_eq!(
        value["specVersion"],
        "1.5",
        "{} 需為 CycloneDX 1.5",
        path.display()
    );

    let mut set = BTreeSet::new();
    let root = &value["metadata"]["component"];
    let root_name = root["name"].as_str().expect("metadata.component 應有 name");
    let root_version = root["version"]
        .as_str()
        .expect("metadata.component 應有 version");
    set.insert(format!("{root_name}@{root_version}"));
    for component in value["components"].as_array().into_iter().flatten() {
        let name = component["name"].as_str().expect("component 應有 name");
        let version = component["version"]
            .as_str()
            .expect("component 應有 version");
        set.insert(format!("{name}@{version}"));
    }
    set
}

fn assert_sbom_matches(meta: &serde_json::Value, root_id: &str, sbom_path: &Path) {
    let expected = reachable_packages(meta, root_id);
    let actual = sbom_components(sbom_path);
    let missing: Vec<&String> = expected.difference(&actual).collect();
    let extra: Vec<&String> = actual.difference(&expected).collect();
    assert!(
        missing.is_empty() && extra.is_empty(),
        "{} 與解析圖不符：\n  缺少（SBOM 未含）：{:?}\n  多餘（不在解析圖）：{:?}\n（請重新產出 SBOM：見 docs/release/UPDATER_RUNBOOK.md §4）",
        sbom_path.display(),
        missing,
        extra
    );
}

fn find_package_id(meta: &serde_json::Value, manifest: &Path) -> String {
    let manifest = manifest
        .canonicalize()
        .unwrap_or_else(|_| manifest.to_path_buf());
    for pkg in meta["packages"].as_array().expect("packages 應為陣列") {
        let path = pkg["manifest_path"].as_str().unwrap_or_default();
        let path = Path::new(path);
        let path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
        if path == manifest {
            return pkg["id"].as_str().expect("套件應有 id").to_string();
        }
    }
    panic!("找不到套件：{}", manifest.display());
}

/// 根 workspace 的六份 SBOM 必須與解析圖一致。
#[test]
fn root_sboms_match_resolved_graph() {
    let root = repo_root();
    let meta = load_metadata(&root.join("Cargo.toml"));
    let cases = [
        ("platform", "platform/Cargo.toml", "platform.cdx.json"),
        (
            "auxbridge_core",
            "core/Cargo.toml",
            "auxbridge_core.cdx.json",
        ),
        ("kem", "crates/kem/Cargo.toml", "kem.cdx.json"),
        (
            "console_manifest",
            "crates/console_manifest/Cargo.toml",
            "console_manifest.cdx.json",
        ),
        (
            "flaps_bridge",
            "crates/flaps_bridge/Cargo.toml",
            "flaps_bridge.cdx.json",
        ),
        ("web", "web/Cargo.toml", "web.cdx.json"),
    ];
    for (package, manifest, sbom) in cases {
        let manifest_path = root.join(manifest);
        let root_id = find_package_id(&meta, &manifest_path);
        let sbom_path = root.join("docs/release/sbom").join(sbom);
        assert!(
            sbom_path.is_file(),
            "找不到 SBOM：{}（package {package}）",
            sbom_path.display()
        );
        assert_sbom_matches(&meta, &root_id, &sbom_path);
    }
}

/// 桌面套件（獨立 workspace）的 SBOM 必須與其解析圖一致。
#[test]
fn desktop_sbom_matches_resolved_graph() {
    let root = repo_root();
    let manifest = root.join("web/src-tauri/Cargo.toml");
    let meta = load_metadata(&manifest);
    let root_id = find_package_id(&meta, &manifest);
    let sbom_path = root.join("docs/release/sbom/auxbridge-web.cdx.json");
    assert!(sbom_path.is_file(), "找不到 SBOM：{}", sbom_path.display());
    assert_sbom_matches(&meta, &root_id, &sbom_path);
}

/// SBOM 必須標示 CycloneDX 1.5（與 CI 產出格式一致）。
#[test]
fn sboms_declare_spec_version_1_5() {
    let dir = repo_root().join("docs/release/sbom");
    let mut count = 0;
    for entry in std::fs::read_dir(&dir).expect("可讀取 SBOM 目錄") {
        let path = entry.expect("可讀取目錄項目").path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let text = std::fs::read_to_string(&path).expect("可讀取 SBOM");
        let value: serde_json::Value = serde_json::from_str(&text).expect("SBOM 應為 JSON");
        assert_eq!(value["specVersion"], "1.5", "{} 非 1.5", path.display());
        count += 1;
    }
    assert!(count >= 7, "SBOM 至少應有 7 份（實得 {count}）");
}
