#![forbid(unsafe_code)]
//! Zero-JS 多頁操作主控台（knowledge/console/）— 系統與流程操作介面。
//!
//! 設計原則（憲章 §2.3＋AGENTS §4＋KEM §8）：
//! - 純靜態 HTML＋共享 CSS；`<script>` 計數恆為 0；互動僅用 :target/:has()
//! - 深色模式（prefers-color-scheme）、列印樣式、:focus-visible 全覆蓋
//! - 頁數擴展律（計畫 1793033568961 §3.3 實測校準）：
//!   pages = 24 + claims + ceil(ledger/200)（24＝HTML 23 頁＋console.css 計數；
//!   與 console_manifest::static_console_pages 逐次相等，test 斷言鎖定）
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

/// 主導覽項目 → manifest 靜態頁 id（`None`＝高基數族首頁）。
const NAV_ITEMS: [(&str, &str, Option<&str>); 7] = [
    ("index", "首頁", Some("console.index")),
    ("roles", "角色", Some("console.index")),
    ("workflows", "工作流", Some("console.workflow.capture")),
    ("claims", "主張", Some("console.claims-index")),
    ("ledger", "帳本", None),
    ("gov", "治理", Some("console.queue")),
    ("ops", "維運", Some("console.maintenance")),
];

/// manifest 靜態頁 id → 相對於 `depth` 層的零 JS 相對連結。
fn console_href(id: &str, depth: usize) -> String {
    let path = console_manifest::static_path_of(id)
        .unwrap_or_else(|| panic!("manifest 未登記靜態頁：{id}"));
    console_manifest::relative_static_path(path, depth)
}

/// 高基數族首頁連結（帳本分頁族第 1 頁）。
fn family_first_href(family_id: &str, depth: usize) -> String {
    let family = console_manifest::family_by_id(family_id)
        .unwrap_or_else(|| panic!("manifest 未登記頁族：{family_id}"));
    console_manifest::relative_static_path(&family.path(1), depth)
}

/// 主導覽連結（路徑全部由 manifest 推導；`depth`＝目前頁面在 `/console` 下的層數）。
fn nav_href(key: &str, depth: usize) -> String {
    match key {
        "roles" => format!("{}#roles", console_href("console.index", depth)),
        "ledger" => family_first_href("kb.ledger-chunk", depth),
        other => {
            let id = NAV_ITEMS
                .iter()
                .find(|(k, _, _)| *k == other)
                .and_then(|(_, _, id)| *id)
                .unwrap_or("console.index");
            console_href(id, depth)
        }
    }
}

/// 工作流頁連結（id 由 manifest 推導；新增工作流不需再改生成器）。
fn workflow_href(name: &str, depth: usize) -> String {
    console_href(&format!("console.workflow.{name}"), depth)
}

/// 共用頁框：頭部導航＋內容＋頁腳（CSS 由 assets/console.css 提供）。
///
/// `depth`＝頁面在 `/console` 下的目錄深度（0＝根頁；1＝roles／workflows／claims），
/// 供相對連結與樣式表路徑推導，避免生成器自造路徑與 manifest 漂移。
fn page(title: &str, active: &str, depth: usize, body: &str) -> String {
    let mut h = String::with_capacity(8 * 1024);
    let css = if depth == 0 {
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
    for (key, label, _) in &NAV_ITEMS {
        let cls = if *key == active {
            " class=\"active\""
        } else {
            ""
        };
        let _ = write!(
            h,
            "<a{} href=\"{}\">{}</a>",
            cls,
            nav_href(key, depth),
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

pub const CONSOLE_CSS: &str = r#"/* 知識控制台樣式（W7 全收斂：token 名稱與值＝web/assets/design-system.css canonical；
   色差屬 grill-E 裁定之收斂結果，映射表見 PLAN_REGISTER addendum #20）。
   本檔為獨立部署工件：token 值複製自 canonical（漂移由 guards 測試比對攔截）。 */
:root{
  --color-surface:#ffffff;
  --color-surface-sunken:#f8fafc;
  --color-text-primary:#0f172a;
  --color-text-secondary:#475569;
  --color-border:#e2e8f0;
  --color-neutral-100:#f1f5f9;
  --color-primary:#2563eb;
  --color-success:#16a34a;
  --color-warning:#d97706;
  --color-error:#dc2626;
  --space-1:4px;--space-2:8px;--space-4:16px;--space-6:24px;
  --radius-lg:12px;
  --font-family-sans:"Inter","Noto Sans TC","Microsoft JhengHei","PingFang TC",system-ui,-apple-system,"Segoe UI",Roboto,Arial,sans-serif;
  --font-family-mono:"JetBrains Mono","Cascadia Mono","Fira Code",Consolas,"Liberation Mono",monospace}
@media (prefers-color-scheme:dark){:root{
  --color-surface:#0f172a;
  --color-surface-sunken:#020617;
  --color-text-primary:#f1f5f9;
  --color-text-secondary:#cbd5e1;
  --color-border:#334155;
  --color-neutral-100:#1e293b;
  --color-primary:#60a5fa;
  --color-success:#22c55e;
  --color-warning:#f59e0b;
  --color-error:#f87171}}
*{box-sizing:border-box}
body{margin:0;background:var(--color-surface-sunken);color:var(--color-text-primary);font-family:var(--font-family-sans);line-height:1.65}
header{background:var(--color-surface);border-bottom:1px solid var(--color-border);padding:var(--space-2) var(--space-4);position:sticky;top:0;z-index:2}
h1{margin:0;font-size:1.25rem}.sub{margin:2px 0 0;color:var(--color-text-secondary);font-size:.8rem}
nav{display:flex;gap:var(--space-1);margin-top:var(--space-2);flex-wrap:wrap}
nav a{padding:5px 12px;border-radius:999px;text-decoration:none;color:var(--color-text-primary);border:1px solid var(--color-border);font-size:.85rem}
nav a:hover{background:var(--color-neutral-100)}
nav a.active,nav a:focus-visible{background:var(--color-neutral-100);border-color:var(--color-primary);color:var(--color-primary);font-weight:600}
nav a:focus-visible{outline:3px solid var(--color-primary);outline-offset:2px}
main{max-width:1100px;margin:0 auto;padding:var(--space-4)}
.panel{background:var(--color-surface);border:1px solid var(--color-border);border-radius:var(--radius-lg);padding:var(--space-4);margin-bottom:var(--space-2)}
h2{font-size:1rem;margin:0 0 var(--space-2)}
h2 small{font-weight:400;color:var(--color-text-secondary)}
h2.mt-4{margin-top:var(--space-6)}
.mt-3{margin-top:var(--space-4)}
.kpi-num{font-size:1.5rem;font-weight:700}
.log-tail{white-space:pre-wrap}
table{width:100%;border-collapse:collapse;font-size:.86rem}
th{text-align:left;color:var(--color-text-secondary);border-bottom:2px solid var(--color-border);padding:6px var(--space-2);white-space:nowrap}
td{padding:8px var(--space-2);border-bottom:1px solid var(--color-border);vertical-align:top}
tbody tr:hover{background:color-mix(in srgb,var(--color-primary) 5%,transparent)}
code,.mono{font-family:var(--font-family-mono);font-size:.82rem}
code.cmd{background:var(--color-neutral-100);padding:2px 8px;border-radius:6px;display:inline-block}
a{color:var(--color-primary)}
a:focus-visible{outline:3px solid var(--color-primary);outline-offset:2px;border-radius:3px}
.pill{display:inline-block;border-radius:999px;padding:1px 10px;font-size:.72rem;font-weight:600;background:var(--color-neutral-100);color:var(--color-text-secondary)}
.pill.published{background:color-mix(in srgb,var(--color-success) 15%,var(--color-surface));color:var(--color-success)}
.pill.superseded,.pill.deprecated{background:color-mix(in srgb,var(--color-warning) 16%,var(--color-surface));color:var(--color-warning)}
.pill.draft,.pill.pending_review{background:var(--color-neutral-100);color:var(--color-text-secondary)}
.pill.retired{background:color-mix(in srgb,var(--color-error) 12%,var(--color-surface));color:var(--color-error)}
.bar{position:relative;height:8px;background:var(--color-neutral-100);border-radius:99px;min-width:80px;display:inline-block;vertical-align:middle}
.bar i{position:absolute;inset:0 auto 0 0;border-radius:99px;background:var(--color-primary)}
.bar.hi i{background:var(--color-success)}.bar.lo i{background:var(--color-warning)}
.banner{border-left:4px solid var(--color-warning);background:color-mix(in srgb,var(--color-warning) 8%,var(--color-surface));padding:var(--space-2) var(--space-4);border-radius:var(--radius-lg);margin-bottom:var(--space-2)}
.empty{color:var(--color-text-secondary);text-align:center;padding:var(--space-6)}
.grid{display:grid;grid-template-columns:repeat(auto-fit,minmax(280px,1fr));gap:var(--space-2)}
.type{color:var(--color-text-secondary);font-size:.78rem}
footer{color:var(--color-text-secondary);font-size:.74rem;padding:var(--space-4);max-width:1100px;margin:0 auto}
@media print{header{position:static}nav{display:none}}
@media (prefers-reduced-motion:reduce){*{transition:none!important}}
/* 跨文件視圖轉場（零 JS：@view-transition；Chrome/Edge 126+、Safari 18.2+） */
@media (prefers-reduced-motion:no-preference){@view-transition{navigation:auto}}
header{view-transition-name:kem-console-header}
"#;

/// 工作流詳情頁本體。
fn workflow_body(name: &str, depth: usize) -> String {
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
                .map(|w| { format!("<a href=\"{}\">{}</a>", workflow_href(w.0, depth), esc(w.0)) })
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
    b.push_str("<h2 class=\"mt-4\">主張內容</h2><p>");
    b.push_str(&esc(&c.body));
    b.push_str("</p><h2 class=\"mt-4\">證據</h2><table><thead><tr><th>來源檔</th><th>修訂版</th><th>逐字引證</th></tr></thead><tbody>");
    for ev in &c.evidence {
        // 原始證據檔不隨主控台部署（平台 /console 僅供應 .html/.css/.json），
        // 故以文字呈現路徑，避免產生部署後的死連結；逐字引證與修訂版仍完整保留。
        let _ = write!(
            b,
            "<tr><td><code>{}</code></td><td><code>{}</code></td><td>{}</td></tr>",
            esc(&ev.path),
            esc(&ev.rev),
            esc(&ev.quote)
        );
    }
    b.push_str("</tbody></table>");
    let _ = write!(
        b,
        "<p class=\"mono mt-3\"><a href=\"{}.json\">檢視機器 SSOT JSON →</a></p>",
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
                "<div class=\"panel\"><div class=\"mono kpi-num\">{count}</div><div class=\"type\">{label}</div></div>"
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
                "<div class=\"panel\"><h2>{}</h2><p>{}</p><p class=\"mono\">{}</p><p><a href=\"{}\">進入 {} 視角 →</a></p></div>",
                esc(label),
                esc(desc),
                esc(flows),
                console_href(&format!("console.role.{key}"), 0),
                esc(label)
            );
        }
        b.push_str("</div></section>");
        b.push_str("<section class=\"panel\"><h2>工作流目錄（13）</h2><p class=\"mono\">");
        b.push_str(
            &WORKFLOWS
                .iter()
                .map(|w| format!("<a href=\"{}\">{}</a>", workflow_href(w.0, 0), esc(w.0)))
                .collect::<Vec<_>>()
                .join(" · "),
        );
        b.push_str(
            "</p><p class=\"type\">擴展律：頁數 ≈ 21 + 主張數 + ⌈帳本/200⌉——隨知識庫成長有機擴展。</p></section>",
        );
        std::fs::write(base.join("index.html"), page("首頁", "index", 0, &b))?;
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
                    workflow_href(w.0, 1),
                    esc(w.0),
                    esc(w.3)
                );
            }
        }
        b.push_str("</tbody></table></section>");
        b.push_str(&format!(
            "<section class=\"panel\"><h2>快捷</h2><p><a href=\"{}\">回首頁</a>｜<a href=\"{}\">審查佇列</a>｜<a href=\"{}\">帳本</a>｜<a href=\"{}\">維運排程</a></p></section>",
            console_href("console.index", 1),
            console_href("console.queue", 1),
            family_first_href("kb.ledger-chunk", 1),
            console_href("console.maintenance", 1),
        ));
        std::fs::write(
            base.join("roles").join(format!("{key}.html")),
            page(&format!("{label}視角"), "roles", 1, &b),
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
                1,
                &workflow_body(name, 1),
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
            page("主張索引", "claims", 1, &b),
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
                page(&c.title, "claims", 1, &claim_body(c, vec_present)),
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
            let ledger_family =
                console_manifest::family_by_id("kb.ledger-chunk").expect("manifest 帳本分頁族");
            if p > 0 {
                let href = console_manifest::relative_static_path(&ledger_family.path(p), 0);
                let _ = write!(b, "<a href=\"{}\">上一頁</a>｜", href);
            }
            if p + 1 < chunks {
                let href = console_manifest::relative_static_path(&ledger_family.path(p + 2), 0);
                let _ = write!(b, "<a href=\"{}\">下一頁</a>", href);
            }
            b.push_str("</p></section>");
            std::fs::write(
                base.join(console_manifest::relative_static_path(
                    &ledger_family.path(p + 1),
                    0,
                )),
                page("帳本", "ledger", 0, &b),
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
        std::fs::write(base.join("queue.html"), page("審查佇列", "gov", 0, &b))?;
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
        std::fs::write(base.join("vectors.html"), page("向量索引", "gov", 0, &b))?;
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
            b.push_str("<pre class=\"mono log-tail\">");
            let tail: Vec<&str> = log.lines().rev().take(30).collect();
            b.push_str(&esc(&tail.into_iter().rev().collect::<Vec<_>>().join("\n")));
            b.push_str("</pre>");
        } else {
            b.push_str("<div class=\"empty\">日誌尚未生成（排程未觸發或窗口外）。</div>");
        }
        b.push_str("</section>");
        std::fs::write(base.join("maintenance.html"), page("維運", "ops", 0, &b))?;
        pages += 1;
    }

    Ok(pages)
}

/// 供 dashboard 狀態卡重導出的政府計數（避免重複實作）。
pub fn status_summary(claims: &[Claim]) -> [(ClaimStatus, usize); 6] {
    status_counts(claims)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    /// W6（計畫 1793033568961）：generate() 回傳頁數＝manifest 擴展律
    /// `static_console_pages(claims, ledger)`——逐次相等斷言。
    /// 以 repo 知識庫（../../knowledge）為輸入、temp 目錄為輸出（不觸碰交付物）。
    #[test]
    fn generated_page_count_matches_manifest_expansion_law() {
        let root = PathBuf::from("../../knowledge");
        if !root.join("claims").is_dir() {
            // 環境無知識庫：誠實跳過（UNKNOWN ≠ PASS）。
            eprintln!("skip: knowledge root not found at {}", root.display());
            return;
        }
        let claims = crate::filing::load_claims(&root.join("claims")).expect("load claims");
        let ledger_path = root.join("ledger.jsonl");
        let entries = if ledger_path.exists() {
            crate::ledger::read_all(&ledger_path).expect("read ledger")
        } else {
            Vec::new()
        };
        let queue =
            crate::gov::pending_queue(&root.join(".kem/review-queue.jsonl")).unwrap_or_default();
        let vectors = crate::embed::load_vectors(&root.join(".kem/vectors.json"))
            .ok()
            .flatten();

        let out = std::env::temp_dir().join(format!("kem-console-law-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&out);
        std::fs::create_dir_all(&out).expect("temp out");
        let root_tmp = out.join("knowledge");
        std::fs::create_dir_all(&root_tmp).expect("tmp knowledge root");

        let pages =
            generate(&root_tmp, &claims, &entries, &queue, vectors.as_ref()).expect("generate");
        let expected = console_manifest::static_console_pages(claims.len(), entries.len());
        assert_eq!(
            pages,
            expected,
            "擴展律失準：generate()={pages} vs manifest={expected}（claims={}，ledger={}）",
            claims.len(),
            entries.len()
        );
        // HTML 檔數＝回傳值 −1（console.css 計數）。
        let html_count = count_html(&root_tmp.join("console"));
        assert_eq!(html_count, pages - 1, "HTML 檔數應為回傳頁數減 CSS 計數");
        // W7：生成 CSS 不得殘留舊詞彙 token（--bg/--accent 等自造名稱）。
        let css = std::fs::read_to_string(root_tmp.join("console/assets/console.css"))
            .expect("console.css");
        for legacy in ["--bg:", "--accent:", "--ink:", "--s1:", "--font:"] {
            assert!(
                !css.contains(legacy),
                "legacy token {legacy} survived W7 convergence"
            );
        }
        // W6：生成頁零內聯樣式（CSP 相容契約）。
        let index_html =
            std::fs::read_to_string(root_tmp.join("console/index.html")).expect("index.html");
        assert!(
            !index_html.contains("style="),
            "inline style survived de-inlining"
        );

        let _ = std::fs::remove_dir_all(&out);
    }

    fn count_html(dir: &Path) -> usize {
        let mut n = 0;
        if let Ok(rd) = std::fs::read_dir(dir) {
            for e in rd.flatten() {
                let p = e.path();
                if p.is_dir() {
                    n += count_html(&p);
                } else if p.extension().and_then(|x| x.to_str()) == Some("html") {
                    n += 1;
                }
            }
        }
        n
    }

    /// 產生主控台到暫存目錄（連結完整性與 manifest 覆蓋檢查共用）。
    /// 無知識庫時回傳 `None`（誠實跳過；UNKNOWN ≠ PASS）。
    fn generate_console_fixture(tag: &str) -> Option<PathBuf> {
        let root = PathBuf::from("../../knowledge");
        if !root.join("claims").is_dir() {
            eprintln!("skip: knowledge root not found at {}", root.display());
            return None;
        }
        let claims = crate::filing::load_claims(&root.join("claims")).expect("load claims");
        let ledger_path = root.join("ledger.jsonl");
        let entries = if ledger_path.exists() {
            crate::ledger::read_all(&ledger_path).expect("read ledger")
        } else {
            Vec::new()
        };
        let queue =
            crate::gov::pending_queue(&root.join(".kem/review-queue.jsonl")).unwrap_or_default();
        let vectors = crate::embed::load_vectors(&root.join(".kem/vectors.json"))
            .ok()
            .flatten();
        let out = std::env::temp_dir().join(format!("kem-console-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&out);
        std::fs::create_dir_all(&out).expect("temp out");
        let root_tmp = out.join("knowledge");
        std::fs::create_dir_all(&root_tmp).expect("tmp knowledge root");
        generate(&root_tmp, &claims, &entries, &queue, vectors.as_ref()).expect("generate");
        Some(root_tmp.join("console"))
    }

    fn collect_html(dir: &Path, out: &mut Vec<PathBuf>) {
        if let Ok(rd) = std::fs::read_dir(dir) {
            for e in rd.flatten() {
                let p = e.path();
                if p.is_dir() {
                    collect_html(&p, out);
                } else if p.extension().and_then(|x| x.to_str()) == Some("html") {
                    out.push(p);
                }
            }
        }
    }

    /// W11：生成物連結完整性——所有內部相對連結必須解析到實際存在的檔案
    /// （零 JS 主控台無路由可用，死連結即功能缺口）。
    #[test]
    fn generated_console_links_resolve_within_bundle() {
        let Some(console) = generate_console_fixture("links") else {
            return;
        };
        let mut htmls = Vec::new();
        collect_html(&console, &mut htmls);
        assert!(!htmls.is_empty(), "generated console has no html");
        let mut problems = Vec::new();
        for file in &htmls {
            let text = std::fs::read_to_string(file).unwrap_or_default();
            for attr in ["href=\"", "src=\""] {
                let mut rest = text.as_str();
                while let Some(idx) = rest.find(attr) {
                    rest = &rest[idx + attr.len()..];
                    let Some(end) = rest.find('"') else { break };
                    let value = &rest[..end];
                    rest = &rest[end..];
                    if value.is_empty()
                        || value.starts_with('#')
                        || value.starts_with("http://")
                        || value.starts_with("https://")
                        || value.starts_with("mailto:")
                        || value.starts_with("data:")
                    {
                        continue;
                    }
                    let path = value.split('#').next().unwrap_or(value);
                    let path = path.split('?').next().unwrap_or(path);
                    if path.is_empty() {
                        continue;
                    }
                    if path.starts_with('/') {
                        problems.push(format!(
                            "{}: absolute internal link {} (bundle 可攜性)",
                            file.display(),
                            value
                        ));
                        continue;
                    }
                    let resolved = file.parent().unwrap_or(Path::new(".")).join(path);
                    if !resolved.exists() {
                        problems.push(format!("{}: dead link {}", file.display(), value));
                    }
                }
            }
        }
        assert!(
            problems.is_empty(),
            "generated console link integrity violated:\n{}",
            problems.join("\n")
        );
    }

    /// W11：manifest 登記的 23 個靜態頁必須全部實際生成（雙向對帳）。
    #[test]
    fn generated_console_covers_every_manifest_static_page() {
        let Some(console) = generate_console_fixture("coverage") else {
            return;
        };
        let ids = console_manifest::static_ids_with_prefix("console.");
        assert_eq!(ids.len(), 23, "manifest 靜態頁應為 23 筆");
        for id in ids {
            let path = console_manifest::static_path_of(id).expect("static path");
            let rel = console_manifest::relative_static_path(path, 0);
            let file = console.join(&rel);
            assert!(
                file.is_file(),
                "manifest 靜態頁 {id} 未生成：{}",
                file.display()
            );
        }
    }
}
