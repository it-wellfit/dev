#![forbid(unsafe_code)]
//! Zero-JS 多頁操作主控台（knowledge/console/）— 系統與流程操作介面。
//!
//! 設計原則（憲章 §2.3＋AGENTS §4＋KEM §8）：
//! - 純靜態 HTML＋共享 CSS；`<script>` 計數恆為 0；互動僅用 :target/:has()
//! - 深色模式（prefers-color-scheme）、列印樣式、:focus-visible 全覆蓋
//! - 頁數 = f(claims, workflows, roles, ledger chunks)——有機擴展律：
//!   pages ≈ 21 + claims + ceil(ledger/200)；主控台規模隨知識庫成長
//! - 角色 × 工作流 × 情境全覆蓋（developer/reviewer/operator/pm/auditor）

use crate::filing::status_counts;
use crate::schema::{Claim, ClaimStatus, KemError};
use crate::{embed, ledger};
use chrono::Utc;
use std::fmt::Write as _;
use std::path::Path;

pub const LEDGER_PAGE_SIZE: usize = 200;

/// 工作流參考資料（13 命令全覆蓋；靜態表——命令契約不隨資料變動）。
///
/// 元素：(name, actor, syntax, purpose, input, output, failure)。
const WORKFLOWS: [(&str, &str, &str, &str, &str, &str, &str); 13] = [
    ("capture", "開發者", "kem capture --topic T --name N --file F [--ext md]",
     "L0 擷取：來源檔不可變入庫 raw/（SHA-256 鎖定、同內容冪等、異內容拒絕）",
     "來源檔案", "raw/<topic>/<date>-<name>.<ext>＋帳本 Capture 條目",
     "路徑穿越拒絕（ext 限英數）；覆寫拒絕"),
    ("ingest", "開發者", "kem ingest --file batch.json",
     "L1 原子化＋T1 裁決：逐字引證/重複/機密三閘；批次內可見性（Fix #2）",
     "claims JSON 陣列（BOM 容忍；id 可省略自動編號）",
     "claims/<id>.json＋帳本 Vet/File＋佇列（reject/review）",
     "捏造引證 fail-closed；批次內重複→review"),
    ("ask", "開發者/PM", "kem ask --query Q [--top N] [--alpha A] [--bm25-only] [--refusal-floor F]",
     "P2b 混合檢索：α·BM25＋(1−α)·向量；語義拒答地板 0.55；誠實拒答",
     "自然語言查詢（中英皆可——跨語言實證）",
     "排序結果＋via 標記（hybrid/bm25）＋模式聲明",
     "嵌入器不可用→BM25-only 如實降級；金標 20/20"),
    ("eval", "審查者", "kem eval [--set golden.json]",
     "V1 金標迴歸閘：top-1 精準＋4 拒答案例（hybrid 模式含地板）",
     "golden-20.json（JSONL）", "pass/total＋失敗明細（stderr）",
     "非 20/20 → 退出碼非零（阻斷）"),
    ("embed", "維運者", "kem embed",
     "P2b 向量重建：單批嵌入全部可檢索主張（OVH bge-m3→Ollama 備援）",
     "claims/ SSOT", ".kem/vectors.json（含 body-sha 陳舊防護＋模型歸因）",
     "429 限流→下一池；全敗→誠實 Err"),
    ("sync", "維運者", "kem sync",
     "M6 演進：引證逐字重驗證＋信心衰減/再確認＋問題入佇列",
     "claims/＋證據檔現況", "信心更新＋佇列（quote drifted 等）＋帳本 Sync",
     "漂移可見不靜默；取代不刪除"),
    ("supersede", "審查者（Track A）", "kem supersede --old ID --file successor.json",
     "M6 結構取代：繼任者 published、前任 superseded 降級可檢索",
     "successor JSON（Track A 必需）", "雙向連結＋帳本 Supersede/File",
     "Track B 拒絕；非 published/deprecated 拒絕"),
    ("vet2", "審查者", "kem vet2 --claim ID [--votes K] [--local-only]",
     "T2 五模型聯合裁決（SCS 輪換＋配額護欄＋每票歸因）",
     "claim ID", "帳本 Vet（models_used/agreement/逐票）",
     "agreement<0.60→入隊；全端點不可達→誠實 UNKNOWN"),
    ("review", "審查者", "kem review --claim ID --disposition accept|reject|supersede|defer [--reviewer R] [--note N]",
     "M7 審查处置：佇列 append-only（resolved 行不計 pending）",
     "佇列 pending 項", "佇列处置行（歷史保留）",
     "处置後 pending 計數即降（doctor 對帳）"),
    ("lint", "維運者", "kem lint",
     "M7 規則 L1–L5：無證據/取代斷鏈/陳舊/標題近碰撞",
     "claims/ SSOT", "逐項 findings（rule+claim+detail）",
     "findings>0 → 退出碼非零"),
    ("doctor", "維運者", "kem doctor",
     "深健康：帳本鏈驗證＋claims 計數＋pending 佇列（gov::pending_queue）",
     "帳本＋claims＋佇列", "Health 報告（ledger_ok/findings/queue）",
     "chain 斷裂→明確錯誤（竄改偵測）"),
    ("status", "稽核者", "kem status",
     "狀態摘要：六態計數＋帳本條目數",
     "claims/＋帳本", "狀態行", "—"),
    ("finalize", "維運者", "kem finalize",
     "M4 視圖重生成：index.md＋log.md＋dashboard.html（零 JS 自檢）",
     "claims/＋帳本", "三視圖＋findings>0 時非零退出",
     "script 計數必為 0（自檢）"),
];

/// 角色視角 → 相關工作流與檔案。
const ROLES: [(&str, &str, &str, &str); 5] = [
    (
        "developer",
        "開發者",
        "capture, ingest, ask, sync, supersede, embed",
        "日常知識生產：擷取來源、裁決入庫、混合檢索、演化維護",
    ),
    (
        "reviewer",
        "審查者",
        "eval, vet2, review, supersede",
        "品質關卡：金標迴歸、聯合裁決分歧複核、处置佇列、結構取代",
    ),
    (
        "operator",
        "維運者",
        "sync, embed, lint, doctor, status, finalize",
        "自維運迴圈：排程守護下的 sync/lint/finalize＋健康檢查",
    ),
    (
        "pm",
        "產品經理",
        "ask, eval, status, finalize",
        "產品可見面：儀表板階段卡片、覆蓋狀態、Phase 門證據（A 未核准如實呈現）",
    ),
    (
        "auditor",
        "稽核者",
        "doctor, status, review",
        "稽核面：帳本鏈驗證、證據直達（claims JSON＋來源 rev）、处置歷史",
    ),
];

fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn href_fix(key: &str, active: &str) -> String {
    let base = if active == "index" { "" } else { "../" };
    match key {
        "index" => format!("{base}index.html"),
        "roles" => format!("{base}index.html#roles"),
        "workflows" => format!("{base}workflows/capture.html"),
        "claims" => format!("{base}claims/index.html"),
        "ledger" => format!("{base}ledger-1.html"),
        "gov" => format!("{base}queue.html"),
        "ops" => format!("{base}maintenance.html"),
        _ => format!("{base}index.html"),
    }
}

fn workflow_href(active: &str, name: &str) -> String {
    let base = if active == "index" {
        "workflows/"
    } else {
        "../workflows/"
    };
    format!("{base}{name}.html")
}

/// 共用頁框：頭部導航＋內容＋頁腳（CSS 由 assets/console.css 提供）。
fn page(title: &str, active: &str, body: &str) -> String {
    let mut h = String::with_capacity(8 * 1024);
    let css = if active == "index" {
        "assets/console.css"
    } else {
        "../assets/console.css"
    };
    let _ = write!(
        h,
        "<!DOCTYPE html>\n<html lang=\"zh-Hant\">\n<head>\n<meta charset=\"utf-8\">\n\
         <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n\
         <title>{t} — KEM 操作主控台</title>\n\
         <link rel=\"stylesheet\" href=\"{c}\">\n</head>\n<body>\n",
        t = esc(title),
        c = css
    );
    h.push_str("<header><h1>KEM 操作主控台</h1><p class=\"sub\">零 JavaScript｜系統與流程操作介面｜知識演進模組 v");
    h.push_str(crate::SCHEMA_VERSION);
    h.push_str("</p><nav aria-label=\"主導航\">");
    let nav = [
        ("index", "首頁"),
        ("roles", "角色"),
        ("workflows", "工作流"),
        ("claims", "主張"),
        ("ledger", "帳本"),
        ("gov", "治理"),
        ("ops", "維運"),
    ];
    for (key, label) in &nav {
        let cls = if *key == active {
            " class=\"active\""
        } else {
            ""
        };
        let _ = write!(
            h,
            "<a{} href=\"{}\">{}</a>",
            cls,
            href_fix(key, active),
            label
        );
    }
    h.push_str("</nav></header>\n<main>\n");
    h.push_str(body);
    h.push_str("</main>\n<footer>由 <code>kem console</code> 生成｜");
    let _ = write!(h, "{}", esc(&Utc::now().to_rfc3339()));
    h.push_str(
        "｜純靜態零 JavaScript｜資料來源：claims SSOT＋帳本＋佇列</footer>\n</body>\n</html>\n",
    );
    h
}

pub const CONSOLE_CSS: &str = r#":root{
  --bg:#f4f6f9;--surface:#fff;--ink:#1b2733;--muted:#5c6b7a;--accent:#0b5cab;
  --ok:#177245;--warn:#9a5b00;--err:#b3261e;--border:#d9dfe7;--chip:#eef2f7;
  --s1:4px;--s2:8px;--s3:16px;--s4:24px;--radius:10px;
  --font:"Microsoft JhengHei","PingFang TC","Noto Sans TC",system-ui,sans-serif;
  --mono:Consolas,"Cascadia Mono",monospace}
@media (prefers-color-scheme:dark){:root{--bg:#10161d;--surface:#182029;--ink:#e8edf3;
  --muted:#9db0c2;--accent:#6db3f2;--ok:#5cd6a2;--warn:#f0b45c;--err:#ff8f86;
  --border:#2c3947;--chip:#212c38}}
*{box-sizing:border-box}
body{margin:0;background:var(--bg);color:var(--ink);font-family:var(--font);line-height:1.65}
header{background:var(--surface);border-bottom:1px solid var(--border);padding:var(--s2) var(--s3);position:sticky;top:0;z-index:2}
h1{margin:0;font-size:1.25rem}.sub{margin:2px 0 0;color:var(--muted);font-size:.8rem}
nav{display:flex;gap:var(--s1);margin-top:var(--s2);flex-wrap:wrap}
nav a{padding:5px 12px;border-radius:999px;text-decoration:none;color:var(--ink);border:1px solid var(--border);font-size:.85rem}
nav a:hover{background:var(--chip)}
nav a.active,nav a:focus-visible{background:var(--chip);border-color:var(--accent);color:var(--accent);font-weight:600}
nav a:focus-visible{outline:3px solid var(--accent);outline-offset:2px}
main{max-width:1100px;margin:0 auto;padding:var(--s3)}
.panel{background:var(--surface);border:1px solid var(--border);border-radius:var(--radius);padding:var(--s3);margin-bottom:var(--s2)}
h2{font-size:1rem;margin:0 0 var(--s2)}
h2 small{font-weight:400;color:var(--muted)}
table{width:100%;border-collapse:collapse;font-size:.86rem}
th{text-align:left;color:var(--muted);border-bottom:2px solid var(--border);padding:6px var(--s2);white-space:nowrap}
td{padding:8px var(--s2);border-bottom:1px solid var(--border);vertical-align:top}
tbody tr:hover{background:color-mix(in srgb,var(--accent) 5%,transparent)}
code,.mono{font-family:var(--mono);font-size:.82rem}
code.cmd{background:var(--chip);padding:2px 8px;border-radius:6px;display:inline-block}
a{color:var(--accent)}
a:focus-visible{outline:3px solid var(--accent);outline-offset:2px;border-radius:3px}
.pill{display:inline-block;border-radius:999px;padding:1px 10px;font-size:.72rem;font-weight:600;background:var(--chip);color:var(--muted)}
.pill.published{background:color-mix(in srgb,var(--ok) 15%,var(--surface));color:var(--ok)}
.pill.superseded,.pill.deprecated{background:color-mix(in srgb,var(--warn) 16%,var(--surface));color:var(--warn)}
.pill.draft,.pill.pending_review{background:var(--chip);color:var(--muted)}
.pill.retired{background:color-mix(in srgb,var(--err) 12%,var(--surface));color:var(--err)}
.bar{position:relative;height:8px;background:var(--chip);border-radius:99px;min-width:80px;display:inline-block;vertical-align:middle}
.bar i{position:absolute;inset:0 auto 0 0;border-radius:99px;background:var(--accent)}
.bar.hi i{background:var(--ok)}.bar.lo i{background:var(--warn)}
.banner{border-left:4px solid var(--warn);background:color-mix(in srgb,var(--warn) 8%,var(--surface));padding:var(--s2) var(--s3);border-radius:var(--radius);margin-bottom:var(--s2)}
.empty{color:var(--muted);text-align:center;padding:var(--s4)}
.grid{display:grid;grid-template-columns:repeat(auto-fit,minmax(280px,1fr));gap:var(--s2)}
.type{color:var(--muted);font-size:.78rem}
footer{color:var(--muted);font-size:.74rem;padding:var(--s3);max-width:1100px;margin:0 auto}
@media print{header{position:static}nav{display:none}}
@media (prefers-reduced-motion:reduce){*{transition:none!important}}
"#;

/// 工作流詳情頁本體。
fn workflow_body(name: &str, active: &str) -> String {
    let mut b = String::new();
    let _ = write!(b, "<section class=\"panel\"><h2>工作流：{}</h2>", esc(name));
    if let Some(w) = WORKFLOWS.iter().find(|w| w.0 == name) {
        let (role, syntax, purpose, input, output, failure) = (w.1, w.2, w.3, w.4, w.5, w.6);
        let _ = write!(
            b,
            "<table><tbody>\
             <tr><th scope=\"row\">命令</th><td><code class=\"cmd\">{}</code></td></tr>\
             <tr><th scope=\"row\">主要角色</th><td>{}</td></tr>\
             <tr><th scope=\"row\">目的</th><td>{}</td></tr>\
             <tr><th scope=\"row\">輸入</th><td>{}</td></tr>\
             <tr><th scope=\"row\">產出</th><td>{}</td></tr>\
             <tr><th scope=\"row\">失敗模式／防護</th><td>{}</td></tr>\
             <tr><th scope=\"row\">其他工作流</th><td>{}</td></tr>\
             </tbody></table>",
            esc(syntax),
            esc(role),
            esc(purpose),
            esc(input),
            esc(output),
            esc(failure),
            WORKFLOWS
                .iter()
                .filter(|w| w.0 != name)
                .map(|w| {
                    format!(
                        "<a href=\"{}\">{}</a>",
                        workflow_href(active, w.0),
                        esc(w.0)
                    )
                })
                .collect::<Vec<_>>()
                .join(" · "),
        );
    }
    b.push_str("</section>");
    b
}

/// 單一主張詳情頁本體。
fn claim_body(c: &Claim, vec_present: bool) -> String {
    let mut b = String::new();
    let _ = write!(
        b,
        "<section class=\"panel\"><h2>{} <span class=\"pill {:?}\">{:?}</span></h2>",
        esc(&c.title),
        c.status,
        c.status
    );
    let _ = write!(
        b,
        "<table><tbody>\
         <tr><th scope=\"row\">ID</th><td><code>{}</code></td></tr>\
         <tr><th scope=\"row\">類型</th><td>{:?}</td></tr>\
         <tr><th scope=\"row\">信心</th><td>{:.2}</td></tr>\
         <tr><th scope=\"row\">軌道</th><td>Track {:?}</td></tr>\
         <tr><th scope=\"row\">有效起訖</th><td>{} → {}</td></tr>\
         <tr><th scope=\"row\">主題</th><td>{}</td></tr>\
         <tr><th scope=\"row\">取代</th><td>{}</td></tr>\
         <tr><th scope=\"row\">被取代</th><td>{}</td></tr>\
         <tr><th scope=\"row\">向量</th><td>{}</td></tr>\
         </tbody></table>",
        esc(&c.id),
        c.claim_type,
        c.confidence,
        c.track,
        c.valid_from,
        c.valid_to
            .map(|d| d.to_string())
            .unwrap_or_else(|| "現行有效".into()),
        esc(&c.topics.join("、")),
        c.supersedes.join("、"),
        c.superseded_by.clone().unwrap_or_else(|| "—".into()),
        if vec_present {
            "已建立（1024 維，陳舊防護啟用）"
        } else {
            "未建立／已陳舊（BM25-only 路徑）"
        }
    );
    b.push_str("<h2 style=\"margin-top:16px\">主張內容</h2><p>");
    b.push_str(&esc(&c.body));
    b.push_str("</p><h2 style=\"margin-top:16px\">證據</h2><table><thead><tr><th>來源檔</th><th>修訂版</th><th>逐字引證</th></tr></thead><tbody>");
    for ev in &c.evidence {
        let _ = write!(
            b,
            "<tr><td><a href=\"../../../{}\"><code>{}</code></a></td><td><code>{}</code></td><td>{}</td></tr>",
            esc(&ev.path),
            esc(&ev.path),
            esc(&ev.rev),
            esc(&ev.quote)
        );
    }
    b.push_str("</tbody></table>");
    let _ = write!(
        b,
        "<p class=\"mono\" style=\"margin-top:12px\"><a href=\"{}.json\">檢視機器 SSOT JSON →</a></p>",
        c.id
    );
    b.push_str("</section>");
    b
}

/// 生成全部主控台頁面（回傳生成頁數——擴展律可審計）。
pub fn generate(
    root: &Path,
    claims: &[Claim],
    ledger_entries: &[ledger::LedgerEntry],
    queue_pending: &[(String, Vec<String>)],
    vectors: Option<&embed::VectorIndexFile>,
) -> Result<usize, KemError> {
    let base = root.join("console");
    std::fs::create_dir_all(base.join("assets"))?;
    std::fs::create_dir_all(base.join("roles"))?;
    std::fs::create_dir_all(base.join("workflows"))?;
    std::fs::create_dir_all(base.join("claims"))?;
    let mut pages = 0usize;

    std::fs::write(base.join("assets/console.css"), CONSOLE_CSS)?;
    pages += 1;

    let counts = status_counts(claims);
    let retrievable = claims.iter().filter(|c| c.status.retrievable()).count();
    let now = Utc::now().to_rfc3339();

    // ── index（sitemap＋角色入口＋狀態卡）──
    {
        let mut b = String::new();
        b.push_str("<section class=\"panel\"><h2>生命週期狀態</h2><div class=\"grid\">");
        let labels = [
            (ClaimStatus::Draft, "草稿"),
            (ClaimStatus::PendingReview, "待審"),
            (ClaimStatus::Published, "已發布"),
            (ClaimStatus::Superseded, "已取代"),
            (ClaimStatus::Deprecated, "已棄用"),
            (ClaimStatus::Retired, "已退役"),
        ];
        for ((status, count), (_, label)) in counts.iter().zip(labels.iter()) {
            let _ = (status, count);
            let _ = write!(
                b,
                "<div class=\"panel\"><div class=\"mono\" style=\"font-size:1.5rem;font-weight:700\">{count}</div><div class=\"type\">{label}</div></div>"
            );
        }
        b.push_str("</div>");
        let _ = write!(
            b,
            "<p class=\"type\">可檢索 {retrievable} 筆｜帳本 {} 條｜待審 {} 筆｜向量 {} 條｜生成 {}</p></section>",
            ledger_entries.len(),
            queue_pending.len(),
            vectors.map(|v| v.entries.len()).unwrap_or(0),
            esc(&now)
        );
        b.push_str("<section class=\"panel\" id=\"roles\"><h2>角色入口</h2><div class=\"grid\">");
        for (key, label, flows, desc) in &ROLES {
            let _ = write!(
                b,
                "<div class=\"panel\"><h2>{}</h2><p>{}</p><p class=\"mono\">{}</p><p><a href=\"roles/{}.html\">進入 {} 視角 →</a></p></div>",
                esc(label),
                esc(desc),
                esc(flows),
                key,
                esc(label)
            );
        }
        b.push_str("</div></section>");
        b.push_str("<section class=\"panel\"><h2>工作流目錄（13）</h2><p class=\"mono\">");
        b.push_str(
            &WORKFLOWS
                .iter()
                .map(|w| {
                    format!(
                        "<a href=\"{}\">{}</a>",
                        workflow_href("index", w.0),
                        esc(w.0)
                    )
                })
                .collect::<Vec<_>>()
                .join(" · "),
        );
        b.push_str(
            "</p><p class=\"type\">擴展律：頁數 ≈ 21 + 主張數 + ⌈帳本/200⌉——隨知識庫成長有機擴展。</p></section>",
        );
        std::fs::write(base.join("index.html"), page("首頁", "index", &b))?;
        pages += 1;
    }

    // ── 角色頁（5）──
    for (key, label, flows, desc) in &ROLES {
        let mut b = String::new();
        let _ = write!(
            b,
            "<section class=\"panel\"><h2>{}視角</h2><p>{}</p><p class=\"mono\">相關工作流：{}</p></section>",
            esc(label),
            esc(desc),
            esc(flows)
        );
        b.push_str(
            "<section class=\"panel\"><h2>此角色的工作流</h2><table><thead><tr><th>命令</th><th>目的</th></tr></thead><tbody>",
        );
        for flow in flows.split(", ") {
            if let Some(w) = WORKFLOWS.iter().find(|w| w.0 == flow) {
                let _ = write!(
                    b,
                    "<tr><td><a href=\"{}\"><code>{}</code></a></td><td>{}</td></tr>",
                    workflow_href(key, w.0),
                    esc(w.0),
                    esc(w.3)
                );
            }
        }
        b.push_str("</tbody></table></section>");
        b.push_str(
            "<section class=\"panel\"><h2>快捷</h2><p><a href=\"../index.html\">回首頁</a>｜<a href=\"../queue.html\">審查佇列</a>｜<a href=\"../ledger-1.html\">帳本</a>｜<a href=\"../maintenance.html\">維運排程</a></p></section>",
        );
        std::fs::write(
            base.join("roles").join(format!("{key}.html")),
            page(&format!("{label}視角"), key, &b),
        )?;
        pages += 1;
    }

    // ── 工作流頁（13）──
    for w in WORKFLOWS.iter() {
        let name = w.0;
        std::fs::write(
            base.join("workflows").join(format!("{name}.html")),
            page(
                &format!("工作流 {name}"),
                "workflows",
                &workflow_body(name, "workflows"),
            ),
        )?;
        pages += 1;
    }

    // ── 主張索引＋詳情頁（1 + claims）──
    {
        let mut b = String::new();
        b.push_str(
            "<section class=\"panel\"><h2>主張索引</h2><table><thead><tr><th>ID</th><th>標題</th><th>狀態</th><th>信心</th><th>主題</th></tr></thead><tbody>",
        );
        let mut sorted: Vec<&Claim> = claims.iter().collect();
        sorted.sort_by(|a, b2| a.id.cmp(&b2.id));
        for c in &sorted {
            let _ = write!(
                b,
                "<tr><td><a href=\"{}.html\"><code>{}</code></a></td><td>{}</td><td><span class=\"pill {:?}\">{:?}</span></td><td>{:.2}</td><td>{}</td></tr>",
                c.id,
                c.id,
                esc(&c.title),
                c.status,
                c.status,
                c.confidence,
                esc(&c.topics.join("、"))
            );
        }
        b.push_str("</tbody></table></section>");
        std::fs::write(
            base.join("claims").join("index.html"),
            page("主張索引", "claims", &b),
        )?;
        pages += 1;

        let vmap = vectors.as_ref().map(|v| {
            v.entries
                .iter()
                .map(|e| (e.id.as_str(), e))
                .collect::<std::collections::HashMap<_, _>>()
        });
        for c in &sorted {
            let vec_present = vmap
                .as_ref()
                .and_then(|m| m.get(c.id.as_str()))
                .map(|e| e.body_sha == crate::capture::sha256_hex(c.body.as_bytes()))
                .unwrap_or(false);
            std::fs::write(
                base.join("claims").join(format!("{}.html", c.id)),
                page(&c.title, "claims", &claim_body(c, vec_present)),
            )?;
            std::fs::write(
                base.join("claims").join(format!("{}.json", c.id)),
                serde_json::to_string_pretty(c)?,
            )?;
            pages += 1;
        }
    }

    // ── 帳本分頁（ceil(entries/200)）──
    {
        let chunks = ledger_entries.len().div_ceil(LEDGER_PAGE_SIZE).max(1);
        for p in 0..chunks {
            let start = p * LEDGER_PAGE_SIZE;
            let end = ((p + 1) * LEDGER_PAGE_SIZE).min(ledger_entries.len());
            let mut b = String::new();
            let _ = write!(
                b,
                "<section class=\"panel\"><h2>帳本第 {} / {} 頁（SHA-256 雜湊鏈）</h2><table><thead><tr><th>Seq</th><th>動作</th><th>主張</th><th>時間 (UTC)</th><th>行為者</th></tr></thead><tbody>",
                p + 1,
                chunks
            );
            for e in &ledger_entries[start..end] {
                let _ = write!(
                    b,
                    "<tr><td class=\"mono\">{}</td><td>{}</td><td><code>{}</code></td><td class=\"mono\">{}</td><td>{}</td></tr>",
                    e.seq,
                    esc(&format!("{:?}", e.action).to_lowercase()),
                    esc(&if e.claim_id.is_empty() {
                        "—".into()
                    } else {
                        e.claim_id.clone()
                    }),
                    esc(&e.ts.to_rfc3339()),
                    esc(&e.actor)
                );
            }
            b.push_str("</tbody></table><p class=\"mono\">");
            if p > 0 {
                let _ = write!(b, "<a href=\"ledger-{}.html\">上一頁</a>｜", p);
            }
            if p + 1 < chunks {
                let _ = write!(b, "<a href=\"ledger-{}.html\">下一頁</a>", p + 2);
            }
            b.push_str("</p></section>");
            std::fs::write(
                base.join(format!("ledger-{}.html", p + 1)),
                page("帳本", "ledger", &b),
            )?;
            pages += 1;
        }
    }

    // ── 佇列頁 ──
    {
        let mut b = String::new();
        b.push_str("<section class=\"panel\"><h2>待審佇列</h2>");
        if queue_pending.is_empty() {
            b.push_str("<div class=\"empty\">佇列清空。</div>");
        } else {
            b.push_str("<table><thead><tr><th>主張</th><th>原因</th></tr></thead><tbody>");
            for (id, reasons) in queue_pending {
                let _ = write!(
                    b,
                    "<tr><td><code>{}</code></td><td>{}</td></tr>",
                    esc(id),
                    esc(&reasons.join("；"))
                );
            }
            b.push_str("</tbody></table>");
        }
        b.push_str(
            "</section><section class=\"panel\"><h2>处置命令</h2><p><code class=\"cmd\">kem review --claim <ID> --disposition accept|reject|supersede|defer [--reviewer R] [--note N]</code></p></section>",
        );
        std::fs::write(base.join("queue.html"), page("審查佇列", "gov", &b))?;
        pages += 1;
    }

    // ── 向量索引頁 ──
    {
        let mut b = String::new();
        b.push_str("<section class=\"panel\"><h2>向量索引健康</h2>");
        match vectors {
            Some(v) => {
                let _ = write!(
                    b,
                    "<table><tbody><tr><th>模型</th><td class=\"mono\">{}</td></tr><tr><th>維度</th><td class=\"mono\">{}</td></tr><tr><th>條目</th><td class=\"mono\">{}</td></tr></tbody></table>",
                    esc(&v.model),
                    v.dim,
                    v.entries.len()
                );
                let stale = claims
                    .iter()
                    .filter(|c| {
                        v.entries
                            .iter()
                            .find(|e| e.id == c.id)
                            .map(|e| e.body_sha != crate::capture::sha256_hex(c.body.as_bytes()))
                            .unwrap_or(true)
                    })
                    .count();
                let _ = write!(
                    b,
                    "<p>陳舊／缺失：{stale} 筆（重建：<code class=\"cmd\">kem embed</code>）</p>"
                );
            }
            None => b.push_str(
                "<div class=\"empty\">向量索引未建立——執行 <code>kem embed</code>。</div>",
            ),
        }
        b.push_str("</section>");
        std::fs::write(base.join("vectors.html"), page("向量索引", "gov", &b))?;
        pages += 1;
    }

    // ── 維運頁（排程＋日誌尾部）──
    {
        let mut b = String::new();
        b.push_str(
            "<section class=\"panel\"><h2>Phase I 自維運排程</h2><table><tbody>\
             <tr><th>任務名</th><td class=\"mono\">kem-maintain</td></tr>\
             <tr><th>觸發</th><td>每 15 分鐘（腳本內窗口守衛：平日 08:00–18:00 外靜默退出）</td></tr>\
             <tr><th>腳本</th><td class=\"mono\">Dev\\scripts\\kem-maintain.ps1</td></tr>\
             <tr><th>日誌</th><td class=\"mono\">%LOCALAPPDATA%\\kem-maintain\\kem-maintain.log</td></tr>\
             <tr><th>可逆</th><td class=\"mono\">schtasks /end /tn kem-maintain; schtasks /delete /tn kem-maintain /f</td></tr>\
             </tbody></table></section>",
        );
        let log_path = std::env::var("LOCALAPPDATA")
            .map(|l| format!("{l}\\kem-maintain\\kem-maintain.log"))
            .unwrap_or_default();
        b.push_str("<section class=\"panel\"><h2>維運日誌尾部</h2>");
        if let Ok(log) = std::fs::read_to_string(&log_path) {
            b.push_str("<pre class=\"mono\" style=\"white-space:pre-wrap\">");
            let tail: Vec<&str> = log.lines().rev().take(30).collect();
            b.push_str(&esc(&tail.into_iter().rev().collect::<Vec<_>>().join("\n")));
            b.push_str("</pre>");
        } else {
            b.push_str("<div class=\"empty\">日誌尚未生成（排程未觸發或窗口外）。</div>");
        }
        b.push_str("</section>");
        std::fs::write(base.join("maintenance.html"), page("維運", "ops", &b))?;
        pages += 1;
    }

    Ok(pages)
}

/// 供 dashboard 狀態卡重導出的政府計數（避免重複實作）。
pub fn status_summary(claims: &[Claim]) -> [(ClaimStatus, usize); 6] {
    status_counts(claims)
}
