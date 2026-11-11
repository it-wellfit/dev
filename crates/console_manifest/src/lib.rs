#![forbid(unsafe_code)]
//! 統一營運控制台 manifest（單一事實來源）。
//!
//! 計畫：`.kilo/plans/1793033568961-unified-ops-console-manifest.md` §3。
//! 雙渲染器契約：
//! - `platform`（Dynamic）：axum/Askama 動態頁；`nav::model` 依 `NAV_GROUPS`＋`PAGES`
//!   以 `NavFlags` 過濾生成導覽（無權限入口不渲染）。
//! - `kem`（Static）：`generate()` 生成 `knowledge/console/` 靜態頁；頁數＝
//!   [`static_console_pages`]（擴展律，可審計）。
//!
//! 規則：`id`／動態 `path` 唯一；`nav_group` 來自 [`NAV_GROUPS`] 封閉集合；
//! `Method::Post` 僅 2 筆（呈現型輸出）；導覽順序＝`PAGES` 宣告順序。

/// 渲染器種類：動態（platform 伺服器）或靜態（KEM 生成器）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Renderer {
    /// platform axum/Askama 動態渲染。
    Dynamic,
    /// KEM `generate()` 靜態生成。
    Static,
}

/// 頁面受眾層級。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tier {
    /// 一線操作者。
    Operator,
    /// 管理者。
    Admin,
    /// 技術／樣本頁。
    Technical,
}

/// 存取閘：對映平台 RBAC 權限碼（`NavFlags`）。
///
/// - `Public`：免登入。
/// - `All`：任一登入者。
/// - `Read`：需 `LEDGER_READ`。
/// - `Write`：需 `LEDGER_WRITE`。
/// - `Manage`：需 `ROLE_MANAGE`。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Gate {
    Public,
    All,
    Read,
    Write,
    Manage,
}

/// HTTP 方法：頁面以 GET 為主；POST 僅登記「呈現型輸出」。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Method {
    Get,
    Post,
}

/// 導覽渲染形態：頂層連結或 `<details>` 下拉群組。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavKind {
    /// 群組內每項渲染為頂層連結。
    Flat,
    /// 群組渲染為單一 `<details class="nav-menu">` 下拉。
    Menu,
}

/// 單一頁面登記（動態與靜態共用）。
#[derive(Debug, Clone, Copy)]
pub struct PageSpec {
    /// 穩定 slug（導覽 `aria-current` 與路由對帳鍵）。
    pub id: &'static str,
    /// 動態：路由路徑；靜態：`/console/...` 伺服路徑。
    pub path: &'static str,
    /// 頁面方法（POST 僅呈現型輸出）。
    pub method: Method,
    /// zh-Hant 顯示名。
    pub title: &'static str,
    /// 導覽群組；`None`＝不入導覽（詳情頁、login、states、tauri-preview）。
    pub nav_group: Option<&'static str>,
    /// 導覽連結覆寫（可帶查詢參數預設）；`None`＝用 `path`。
    pub nav_href: Option<&'static str>,
    pub renderer: Renderer,
    pub tier: Tier,
    pub gate: Gate,
    /// 關聯工作流 id（每頁 ≥1）。
    pub workflows: &'static [&'static str],
}

/// 靜態高基數族：manifest 不枚舉實例，只登記族與基數函式。
#[derive(Debug, Clone, Copy)]
pub struct PageFamily {
    pub id: &'static str,
    pub title: &'static str,
    /// 基數：`fn(claims, ledger_entries) -> usize`。
    pub cardinality: fn(usize, usize) -> usize,
    /// 族內第 N 筆（1-based）的路徑模板；`{}` 為序號佔位符。
    pub path_template: &'static str,
}

impl PageFamily {
    /// 族內第 `index` 筆（1-based）的伺服路徑。
    pub fn path(&self, index: usize) -> String {
        self.path_template.replace("{}", &index.to_string())
    }
}

/// 靜態控制台伺服根路徑（platform 以 `/console/*` 託管 KEM 生成物）。
pub const STATIC_ROOT: &str = "/console";

/// 導覽群組（§3.1；順序即渲染順序；`studio` 決定所屬工作室）。
#[derive(Debug, Clone, Copy)]
pub struct NavGroup {
    pub id: &'static str,
    /// zh-Hant 顯示名。
    pub label: &'static str,
    pub kind: NavKind,
    /// 群組內容的典型最低閘（advisory；實際可見性＝任一項目通過其頁面閘）。
    pub gate: Gate,
    /// 所屬工作室 id（STUDIOS 封閉集合；""＝置頂首頁群組，不入工作室）。
    pub studio: &'static str,
}

/// 工作室（Studio）：導覽 IA 第一層（使用者裁定：studio + section 兩層式；
/// 對應產品五大工作室心智模型——業務/銀行支付/治理與管理/系統與知識）。
#[derive(Debug, Clone, Copy)]
pub struct Studio {
    pub id: &'static str,
    /// zh-Hant 顯示名（側欄區段標頭）。
    pub label: &'static str,
}

/// 工作室封閉集合（渲染順序＝宣告順序；overview 為置頂首頁不入工作室）。
pub const STUDIOS: &[Studio] = &[
    Studio {
        id: "biz",
        label: "業務工作室",
    },
    Studio {
        id: "bank",
        label: "銀行支付",
    },
    Studio {
        id: "gov",
        label: "治理與管理",
    },
    Studio {
        id: "ops",
        label: "系統與知識",
    },
];

/// 導覽群組封閉集合（渲染順序＝宣告順序；`studio` 決定所屬工作室）。
pub const NAV_GROUPS: &[NavGroup] = &[
    NavGroup {
        id: "overview",
        label: "總覽",
        kind: NavKind::Flat,
        gate: Gate::All,
        studio: "",
    },
    NavGroup {
        id: "ledger",
        label: "📒 帳務作業",
        kind: NavKind::Menu,
        gate: Gate::Read,
        studio: "biz",
    },
    NavGroup {
        id: "capture",
        label: "📥 匯入與草稿",
        kind: NavKind::Menu,
        gate: Gate::Write,
        studio: "biz",
    },
    NavGroup {
        id: "bankpay",
        label: "🏦 銀行支付",
        kind: NavKind::Menu,
        gate: Gate::Read,
        studio: "bank",
    },
    NavGroup {
        id: "reports",
        label: "📈 報表分析",
        kind: NavKind::Menu,
        gate: Gate::Read,
        studio: "biz",
    },
    NavGroup {
        id: "query",
        label: "🔍 檢索",
        kind: NavKind::Flat,
        gate: Gate::All,
        studio: "biz",
    },
    NavGroup {
        id: "governance",
        label: "🛡️ 治理稽核",
        kind: NavKind::Menu,
        gate: Gate::Manage,
        studio: "gov",
    },
    NavGroup {
        id: "admin",
        label: "⚙️ 管理",
        kind: NavKind::Menu,
        gate: Gate::Manage,
        studio: "gov",
    },
    NavGroup {
        id: "apihub",
        label: "🔌 API 中樞",
        kind: NavKind::Menu,
        gate: Gate::Manage,
        studio: "gov",
    },
    NavGroup {
        id: "system",
        label: "🖥️ 系統",
        kind: NavKind::Flat,
        gate: Gate::All,
        studio: "ops",
    },
    NavGroup {
        id: "knowledge",
        label: "📚 知識治理",
        kind: NavKind::Flat,
        gate: Gate::All,
        studio: "ops",
    },
];

// 平台工作流 id（上位計畫 §3.6 六工作流＋工作階段）。
pub const WF_AUTH: &str = "auth.session";
pub const WF_TX_CREATE: &str = "tx.create";
pub const WF_IMPORT_PARSE: &str = "import.parse";
pub const WF_REVIEW_DECIDE: &str = "review.decide";
pub const WF_BANKPAY: &str = "bankpay.operate";
pub const WF_REPORTS: &str = "reports.read";
pub const WF_GOVERN: &str = "gov.audit";

/// 全量登記（§3.2 動態 41 筆＋§3.3 靜態固定 23 筆）。
/// 順序＝導覽渲染順序（同群組內宣告序）。
pub const PAGES: &[PageSpec] = &[
    // ── Public（1）──
    PageSpec {
        id: "auth.login",
        path: "/login",
        method: Method::Get,
        title: "登入",
        nav_group: None,
        nav_href: None,
        renderer: Renderer::Dynamic,
        tier: Tier::Operator,
        gate: Gate::Public,
        workflows: &[WF_AUTH],
    },
    // ── overview ──
    PageSpec {
        id: "home.index",
        path: "/",
        method: Method::Get,
        title: "總覽",
        nav_group: Some("overview"),
        nav_href: None,
        renderer: Renderer::Dynamic,
        tier: Tier::Operator,
        gate: Gate::All,
        workflows: &[WF_TX_CREATE, WF_IMPORT_PARSE, WF_REPORTS],
    },
    // ── D-6 自我稽核視圖（計畫 1793033761021 B-5.0）：僅本人稽核事件，
    // 伺服器端強制 performed_by=self；Gate::All＝任一登入者可自查。──
    PageSpec {
        id: "my.decisions",
        path: "/my-decisions",
        method: Method::Get,
        title: "我的裁決紀錄",
        nav_group: Some("overview"),
        nav_href: None,
        renderer: Renderer::Dynamic,
        tier: Tier::Operator,
        gate: Gate::All,
        workflows: &[WF_REVIEW_DECIDE, WF_GOVERN],
    },
    // ── ledger（帳務作業）──
    PageSpec {
        id: "tx.list",
        path: "/transactions",
        method: Method::Get,
        title: "交易",
        nav_group: Some("ledger"),
        nav_href: None,
        renderer: Renderer::Dynamic,
        tier: Tier::Operator,
        gate: Gate::Read,
        workflows: &[WF_TX_CREATE],
    },
    PageSpec {
        id: "tx.detail",
        path: "/transactions/:id",
        method: Method::Get,
        title: "交易詳情",
        nav_group: None,
        nav_href: None,
        renderer: Renderer::Dynamic,
        tier: Tier::Operator,
        gate: Gate::Read,
        workflows: &[WF_TX_CREATE],
    },
    PageSpec {
        id: "tx.wizard",
        path: "/new",
        method: Method::Get,
        title: "新增交易",
        nav_group: Some("ledger"),
        nav_href: None,
        renderer: Renderer::Dynamic,
        tier: Tier::Operator,
        gate: Gate::Write,
        workflows: &[WF_TX_CREATE],
    },
    PageSpec {
        id: "accounts.list",
        path: "/accounts",
        method: Method::Get,
        title: "會計科目",
        nav_group: Some("ledger"),
        nav_href: None,
        renderer: Renderer::Dynamic,
        tier: Tier::Operator,
        gate: Gate::Read,
        workflows: &[WF_TX_CREATE, WF_REPORTS],
    },
    PageSpec {
        id: "accounts.detail",
        path: "/accounts/:id",
        method: Method::Get,
        title: "科目詳情",
        nav_group: None,
        nav_href: None,
        renderer: Renderer::Dynamic,
        tier: Tier::Operator,
        gate: Gate::Read,
        workflows: &[WF_REPORTS],
    },
    PageSpec {
        id: "ai.decisions",
        path: "/ai-decisions",
        method: Method::Get,
        title: "AI 決策",
        nav_group: Some("ledger"),
        nav_href: None,
        renderer: Renderer::Dynamic,
        tier: Tier::Operator,
        gate: Gate::Write,
        workflows: &[WF_REVIEW_DECIDE],
    },
    // ── capture（匯入與草稿）──
    PageSpec {
        id: "import.upload",
        path: "/import",
        method: Method::Get,
        title: "快速匯入",
        nav_group: Some("capture"),
        nav_href: None,
        renderer: Renderer::Dynamic,
        tier: Tier::Operator,
        gate: Gate::Write,
        workflows: &[WF_IMPORT_PARSE],
    },
    PageSpec {
        id: "import.history",
        path: "/import/history",
        method: Method::Get,
        title: "匯入歷史",
        nav_group: Some("capture"),
        nav_href: None,
        renderer: Renderer::Dynamic,
        tier: Tier::Operator,
        gate: Gate::Read,
        workflows: &[WF_IMPORT_PARSE],
    },
    PageSpec {
        id: "drafts.list",
        path: "/drafts",
        method: Method::Get,
        title: "草稿解析",
        nav_group: Some("capture"),
        nav_href: None,
        renderer: Renderer::Dynamic,
        tier: Tier::Operator,
        gate: Gate::Write,
        workflows: &[WF_IMPORT_PARSE, WF_REVIEW_DECIDE],
    },
    // ── bankpay ──
    PageSpec {
        id: "bankpay.main",
        path: "/bankpay",
        method: Method::Get,
        title: "銀行支付",
        nav_group: Some("bankpay"),
        nav_href: None,
        renderer: Renderer::Dynamic,
        tier: Tier::Operator,
        gate: Gate::Read,
        workflows: &[WF_BANKPAY],
    },
    PageSpec {
        id: "bankpay.batches",
        path: "/bankpay/batches",
        method: Method::Get,
        title: "付款批次",
        nav_group: Some("bankpay"),
        nav_href: None,
        renderer: Renderer::Dynamic,
        tier: Tier::Operator,
        gate: Gate::Read,
        workflows: &[WF_BANKPAY],
    },
    PageSpec {
        id: "bankpay.batch-detail",
        path: "/bankpay/batches/:batch_id",
        method: Method::Get,
        title: "批次詳情",
        nav_group: None,
        nav_href: None,
        renderer: Renderer::Dynamic,
        tier: Tier::Operator,
        gate: Gate::Read,
        workflows: &[WF_BANKPAY],
    },
    PageSpec {
        id: "bankpay.exceptions",
        path: "/bankpay/exceptions",
        method: Method::Get,
        title: "例外處理",
        nav_group: Some("bankpay"),
        nav_href: None,
        renderer: Renderer::Dynamic,
        tier: Tier::Operator,
        gate: Gate::Read,
        workflows: &[WF_BANKPAY],
    },
    // ── reports ──
    PageSpec {
        id: "reports.monthly",
        path: "/reports/monthly",
        method: Method::Get,
        title: "月報表",
        nav_group: Some("reports"),
        nav_href: None,
        renderer: Renderer::Dynamic,
        tier: Tier::Operator,
        gate: Gate::Read,
        workflows: &[WF_REPORTS],
    },
    PageSpec {
        id: "reports.trial-balance",
        path: "/reports/trial-balance",
        method: Method::Get,
        title: "試算表",
        nav_group: Some("reports"),
        nav_href: None,
        renderer: Renderer::Dynamic,
        tier: Tier::Operator,
        gate: Gate::Read,
        workflows: &[WF_REPORTS],
    },
    PageSpec {
        id: "reports.ledger",
        path: "/reports/ledger",
        method: Method::Get,
        title: "總分類帳",
        nav_group: Some("reports"),
        nav_href: None,
        renderer: Renderer::Dynamic,
        tier: Tier::Operator,
        gate: Gate::Read,
        workflows: &[WF_REPORTS],
    },
    PageSpec {
        id: "reports.pivot",
        path: "/reports/pivot",
        method: Method::Get,
        title: "樞紐分析",
        nav_group: Some("reports"),
        nav_href: Some("/reports/pivot?rows=account&cols=month&measure=debit"),
        renderer: Renderer::Dynamic,
        tier: Tier::Operator,
        gate: Gate::Read,
        workflows: &[WF_REPORTS],
    },
    PageSpec {
        id: "reports.parallel",
        path: "/reports/parallel",
        method: Method::Get,
        title: "平行座標投影",
        nav_group: Some("reports"),
        nav_href: Some("/reports/parallel?months=6"),
        renderer: Renderer::Dynamic,
        tier: Tier::Operator,
        gate: Gate::Read,
        workflows: &[WF_REPORTS],
    },
    // ── query ──
    PageSpec {
        id: "search.page",
        path: "/search",
        method: Method::Get,
        title: "檢索",
        nav_group: Some("query"),
        nav_href: None,
        renderer: Renderer::Dynamic,
        tier: Tier::Operator,
        gate: Gate::All,
        workflows: &[WF_REPORTS],
    },
    // ── governance ──
    PageSpec {
        id: "audit.page",
        path: "/audit",
        method: Method::Get,
        title: "稽核軌跡",
        nav_group: Some("governance"),
        nav_href: None,
        renderer: Renderer::Dynamic,
        tier: Tier::Admin,
        gate: Gate::Manage,
        workflows: &[WF_GOVERN],
    },
    // ── system ──
    PageSpec {
        id: "status.page",
        path: "/status",
        method: Method::Get,
        title: "系統狀態",
        nav_group: Some("system"),
        nav_href: None,
        renderer: Renderer::Dynamic,
        tier: Tier::Operator,
        gate: Gate::All,
        workflows: &[WF_GOVERN],
    },
    PageSpec {
        id: "help.page",
        path: "/help",
        method: Method::Get,
        title: "操作指引",
        nav_group: Some("system"),
        nav_href: None,
        renderer: Renderer::Dynamic,
        tier: Tier::Operator,
        gate: Gate::All,
        workflows: &[WF_AUTH],
    },
    PageSpec {
        id: "profile.page",
        path: "/profile",
        method: Method::Get,
        title: "個人設定",
        nav_group: Some("system"),
        nav_href: None,
        renderer: Renderer::Dynamic,
        tier: Tier::Operator,
        gate: Gate::All,
        workflows: &[WF_AUTH],
    },
    PageSpec {
        id: "states.specimen",
        path: "/states",
        method: Method::Get,
        title: "介面狀態",
        nav_group: None,
        nav_href: None,
        renderer: Renderer::Dynamic,
        tier: Tier::Technical,
        gate: Gate::All,
        workflows: &[WF_AUTH],
    },
    PageSpec {
        id: "tauri.preview",
        path: "/tauri-preview",
        method: Method::Get,
        title: "桌面版對照",
        nav_group: None,
        nav_href: None,
        renderer: Renderer::Dynamic,
        tier: Tier::Technical,
        gate: Gate::All,
        workflows: &[WF_AUTH],
    },
    // ── admin ──
    PageSpec {
        id: "admin.users",
        path: "/admin/users",
        method: Method::Get,
        title: "使用者管理",
        nav_group: Some("admin"),
        nav_href: None,
        renderer: Renderer::Dynamic,
        tier: Tier::Admin,
        gate: Gate::Manage,
        workflows: &[WF_GOVERN],
    },
    PageSpec {
        id: "admin.roles",
        path: "/admin/roles",
        method: Method::Get,
        title: "角色權限",
        nav_group: Some("admin"),
        nav_href: None,
        renderer: Renderer::Dynamic,
        tier: Tier::Admin,
        gate: Gate::Manage,
        workflows: &[WF_GOVERN],
    },
    PageSpec {
        id: "admin.sessions",
        path: "/admin/sessions",
        method: Method::Get,
        title: "工作階段",
        nav_group: Some("admin"),
        nav_href: None,
        renderer: Renderer::Dynamic,
        tier: Tier::Admin,
        gate: Gate::Manage,
        workflows: &[WF_GOVERN],
    },
    PageSpec {
        id: "admin.system",
        path: "/admin/system",
        method: Method::Get,
        title: "系統資訊",
        nav_group: Some("admin"),
        nav_href: None,
        renderer: Renderer::Dynamic,
        tier: Tier::Admin,
        gate: Gate::Manage,
        workflows: &[WF_GOVERN],
    },
    PageSpec {
        id: "admin.parse-coverage",
        path: "/admin/parse_coverage",
        method: Method::Get,
        title: "解析覆蓋率",
        nav_group: Some("admin"),
        nav_href: None,
        renderer: Renderer::Dynamic,
        tier: Tier::Admin,
        gate: Gate::Manage,
        workflows: &[WF_IMPORT_PARSE],
    },
    // ── apihub ──
    PageSpec {
        id: "apihub.catalog",
        path: "/admin/api/catalog",
        method: Method::Get,
        title: "API 中樞",
        nav_group: Some("apihub"),
        nav_href: None,
        renderer: Renderer::Dynamic,
        tier: Tier::Admin,
        gate: Gate::Manage,
        workflows: &[WF_GOVERN],
    },
    PageSpec {
        id: "apihub.test",
        path: "/admin/api/test",
        method: Method::Get,
        title: "API 測試台",
        nav_group: Some("apihub"),
        nav_href: None,
        renderer: Renderer::Dynamic,
        tier: Tier::Admin,
        gate: Gate::Manage,
        workflows: &[WF_GOVERN],
    },
    PageSpec {
        id: "apihub.logs",
        path: "/admin/api/logs",
        method: Method::Get,
        title: "API 日誌",
        nav_group: Some("apihub"),
        nav_href: None,
        renderer: Renderer::Dynamic,
        tier: Tier::Admin,
        gate: Gate::Manage,
        workflows: &[WF_GOVERN],
    },
    PageSpec {
        id: "apihub.config",
        path: "/admin/api/config",
        method: Method::Get,
        title: "API 設定",
        nav_group: Some("apihub"),
        nav_href: None,
        renderer: Renderer::Dynamic,
        tier: Tier::Admin,
        gate: Gate::Manage,
        workflows: &[WF_BANKPAY, WF_GOVERN],
    },
    PageSpec {
        id: "apihub.records",
        path: "/admin/api/records",
        method: Method::Get,
        title: "API 紀錄",
        nav_group: Some("apihub"),
        nav_href: None,
        renderer: Renderer::Dynamic,
        tier: Tier::Admin,
        gate: Gate::Manage,
        workflows: &[WF_GOVERN],
    },
    PageSpec {
        id: "apihub.codes",
        path: "/admin/api/codes",
        method: Method::Get,
        title: "API 代碼",
        nav_group: Some("apihub"),
        nav_href: None,
        renderer: Renderer::Dynamic,
        tier: Tier::Admin,
        gate: Gate::Manage,
        workflows: &[WF_GOVERN],
    },
    PageSpec {
        id: "apihub.release",
        path: "/admin/api/release",
        method: Method::Get,
        title: "API 發布",
        nav_group: Some("apihub"),
        nav_href: None,
        renderer: Renderer::Dynamic,
        tier: Tier::Admin,
        gate: Gate::Manage,
        workflows: &[WF_GOVERN],
    },
    PageSpec {
        id: "apihub.net",
        path: "/admin/api/net",
        method: Method::Get,
        title: "API 網路",
        nav_group: Some("apihub"),
        nav_href: None,
        renderer: Renderer::Dynamic,
        tier: Tier::Admin,
        gate: Gate::Manage,
        workflows: &[WF_GOVERN],
    },
    // ── POST 呈現頁（2；grill-C：/import/preview、/new/step2 為表單流程中間態，
    // 不入 PAGES——矩陣測頁面不變量，專項測試測流程）──
    PageSpec {
        id: "review.bulk-result",
        path: "/reviews/bulk",
        method: Method::Post,
        title: "批次覆核結果",
        nav_group: None,
        nav_href: None,
        renderer: Renderer::Dynamic,
        tier: Tier::Operator,
        gate: Gate::Write,
        workflows: &[WF_REVIEW_DECIDE],
    },
    PageSpec {
        id: "ai.suggestion",
        path: "/drafts/ai_suggest",
        method: Method::Post,
        title: "AI 建議",
        nav_group: None,
        nav_href: None,
        renderer: Renderer::Dynamic,
        tier: Tier::Operator,
        gate: Gate::Write,
        workflows: &[WF_REVIEW_DECIDE],
    },
    // ── knowledge（KEM 靜態固定頁 23；renderer=Static；伺服於 /console/*）──
    PageSpec {
        id: "console.index",
        path: "/console/index.html",
        method: Method::Get,
        title: "知識控制台",
        nav_group: Some("knowledge"),
        nav_href: Some("/console/index.html"),
        renderer: Renderer::Static,
        tier: Tier::Operator,
        gate: Gate::All,
        workflows: &[
            "capture",
            "ingest",
            "ask",
            "sync",
            "supersede",
            "embed",
            "eval",
            "vet2",
            "review",
            "lint",
            "doctor",
            "status",
            "finalize",
        ],
    },
    PageSpec {
        id: "console.claims-index",
        path: "/console/claims/index.html",
        method: Method::Get,
        title: "主張索引",
        nav_group: None,
        nav_href: None,
        renderer: Renderer::Static,
        tier: Tier::Operator,
        gate: Gate::All,
        workflows: &["ingest", "review"],
    },
    PageSpec {
        id: "console.queue",
        path: "/console/queue.html",
        method: Method::Get,
        title: "審查佇列",
        nav_group: None,
        nav_href: None,
        renderer: Renderer::Static,
        tier: Tier::Operator,
        gate: Gate::All,
        workflows: &["review"],
    },
    PageSpec {
        id: "console.vectors",
        path: "/console/vectors.html",
        method: Method::Get,
        title: "向量索引",
        nav_group: None,
        nav_href: None,
        renderer: Renderer::Static,
        tier: Tier::Operator,
        gate: Gate::All,
        workflows: &["embed"],
    },
    PageSpec {
        id: "console.maintenance",
        path: "/console/maintenance.html",
        method: Method::Get,
        title: "維運",
        nav_group: None,
        nav_href: None,
        renderer: Renderer::Static,
        tier: Tier::Operator,
        gate: Gate::All,
        workflows: &["doctor", "status", "finalize"],
    },
    PageSpec {
        id: "console.role.developer",
        path: "/console/roles/developer.html",
        method: Method::Get,
        title: "開發者視角",
        nav_group: None,
        nav_href: None,
        renderer: Renderer::Static,
        tier: Tier::Operator,
        gate: Gate::All,
        workflows: &["capture", "ingest", "ask", "sync", "supersede", "embed"],
    },
    PageSpec {
        id: "console.role.reviewer",
        path: "/console/roles/reviewer.html",
        method: Method::Get,
        title: "審查者視角",
        nav_group: None,
        nav_href: None,
        renderer: Renderer::Static,
        tier: Tier::Operator,
        gate: Gate::All,
        workflows: &["eval", "vet2", "review", "supersede"],
    },
    PageSpec {
        id: "console.role.operator",
        path: "/console/roles/operator.html",
        method: Method::Get,
        title: "維運者視角",
        nav_group: None,
        nav_href: None,
        renderer: Renderer::Static,
        tier: Tier::Operator,
        gate: Gate::All,
        workflows: &["sync", "embed", "lint", "doctor", "status", "finalize"],
    },
    PageSpec {
        id: "console.role.pm",
        path: "/console/roles/pm.html",
        method: Method::Get,
        title: "產品經理視角",
        nav_group: None,
        nav_href: None,
        renderer: Renderer::Static,
        tier: Tier::Operator,
        gate: Gate::All,
        workflows: &["ask", "eval", "status", "finalize"],
    },
    PageSpec {
        id: "console.role.auditor",
        path: "/console/roles/auditor.html",
        method: Method::Get,
        title: "稽核者視角",
        nav_group: None,
        nav_href: None,
        renderer: Renderer::Static,
        tier: Tier::Operator,
        gate: Gate::All,
        workflows: &["doctor", "status", "review"],
    },
    PageSpec {
        id: "console.workflow.capture",
        path: "/console/workflows/capture.html",
        method: Method::Get,
        title: "工作流 capture",
        nav_group: None,
        nav_href: None,
        renderer: Renderer::Static,
        tier: Tier::Operator,
        gate: Gate::All,
        workflows: &["capture"],
    },
    PageSpec {
        id: "console.workflow.ingest",
        path: "/console/workflows/ingest.html",
        method: Method::Get,
        title: "工作流 ingest",
        nav_group: None,
        nav_href: None,
        renderer: Renderer::Static,
        tier: Tier::Operator,
        gate: Gate::All,
        workflows: &["ingest"],
    },
    PageSpec {
        id: "console.workflow.ask",
        path: "/console/workflows/ask.html",
        method: Method::Get,
        title: "工作流 ask",
        nav_group: None,
        nav_href: None,
        renderer: Renderer::Static,
        tier: Tier::Operator,
        gate: Gate::All,
        workflows: &["ask"],
    },
    PageSpec {
        id: "console.workflow.sync",
        path: "/console/workflows/sync.html",
        method: Method::Get,
        title: "工作流 sync",
        nav_group: None,
        nav_href: None,
        renderer: Renderer::Static,
        tier: Tier::Operator,
        gate: Gate::All,
        workflows: &["sync"],
    },
    PageSpec {
        id: "console.workflow.supersede",
        path: "/console/workflows/supersede.html",
        method: Method::Get,
        title: "工作流 supersede",
        nav_group: None,
        nav_href: None,
        renderer: Renderer::Static,
        tier: Tier::Operator,
        gate: Gate::All,
        workflows: &["supersede"],
    },
    PageSpec {
        id: "console.workflow.embed",
        path: "/console/workflows/embed.html",
        method: Method::Get,
        title: "工作流 embed",
        nav_group: None,
        nav_href: None,
        renderer: Renderer::Static,
        tier: Tier::Operator,
        gate: Gate::All,
        workflows: &["embed"],
    },
    PageSpec {
        id: "console.workflow.eval",
        path: "/console/workflows/eval.html",
        method: Method::Get,
        title: "工作流 eval",
        nav_group: None,
        nav_href: None,
        renderer: Renderer::Static,
        tier: Tier::Operator,
        gate: Gate::All,
        workflows: &["eval"],
    },
    PageSpec {
        id: "console.workflow.vet2",
        path: "/console/workflows/vet2.html",
        method: Method::Get,
        title: "工作流 vet2",
        nav_group: None,
        nav_href: None,
        renderer: Renderer::Static,
        tier: Tier::Operator,
        gate: Gate::All,
        workflows: &["vet2"],
    },
    PageSpec {
        id: "console.workflow.review",
        path: "/console/workflows/review.html",
        method: Method::Get,
        title: "工作流 review",
        nav_group: None,
        nav_href: None,
        renderer: Renderer::Static,
        tier: Tier::Operator,
        gate: Gate::All,
        workflows: &["review"],
    },
    PageSpec {
        id: "console.workflow.lint",
        path: "/console/workflows/lint.html",
        method: Method::Get,
        title: "工作流 lint",
        nav_group: None,
        nav_href: None,
        renderer: Renderer::Static,
        tier: Tier::Operator,
        gate: Gate::All,
        workflows: &["lint"],
    },
    PageSpec {
        id: "console.workflow.doctor",
        path: "/console/workflows/doctor.html",
        method: Method::Get,
        title: "工作流 doctor",
        nav_group: None,
        nav_href: None,
        renderer: Renderer::Static,
        tier: Tier::Operator,
        gate: Gate::All,
        workflows: &["doctor"],
    },
    PageSpec {
        id: "console.workflow.status",
        path: "/console/workflows/status.html",
        method: Method::Get,
        title: "工作流 status",
        nav_group: None,
        nav_href: None,
        renderer: Renderer::Static,
        tier: Tier::Operator,
        gate: Gate::All,
        workflows: &["status"],
    },
    PageSpec {
        id: "console.workflow.finalize",
        path: "/console/workflows/finalize.html",
        method: Method::Get,
        title: "工作流 finalize",
        nav_group: None,
        nav_href: None,
        renderer: Renderer::Static,
        tier: Tier::Operator,
        gate: Gate::All,
        workflows: &["finalize"],
    },
];

/// 靜態高基數族（每筆主張一頁、帳本分頁）。
pub const FAMILIES: &[PageFamily] = &[
    PageFamily {
        id: "kb.claim",
        title: "主張詳情",
        cardinality: |claims, _ledger| claims,
        path_template: "/console/claims/{}.html",
    },
    PageFamily {
        id: "kb.ledger-chunk",
        title: "帳本分頁",
        cardinality: |_claims, ledger| ledger.max(1).div_ceil(200),
        path_template: "/console/ledger-{}.html",
    },
];

/// KEM 主控台擴展律（§3.3 已裁決；與 `crates/kem/src/console.rs::generate()`
/// 回傳值逐次相等）。`generate()` 將 `assets/console.css` 亦計 1 頁，
/// 故固定常數為 24（HTML 23＋CSS 1）。
pub const KEM_FIXED_PAGES: usize = 24;

/// 靜態控制台頁數＝固定頁（含 CSS 計數）＋主張頁＋帳本分頁。
pub fn static_console_pages(claims: usize, ledger_entries: usize) -> usize {
    KEM_FIXED_PAGES + claims + ledger_entries.max(1).div_ceil(200)
}

/// 依 id 查頁（線性掃描；表小，編譯期常數無需索引）。
pub fn page_by_id(id: &str) -> Option<&'static PageSpec> {
    PAGES.iter().find(|p| p.id == id)
}

/// 導覽用連結：`nav_href` 覆寫優先，否則用 `path`。
pub fn nav_href_of(page: &PageSpec) -> &'static str {
    page.nav_href.unwrap_or(page.path)
}

/// 依 id 查靜態頁（`renderer == Static`）的伺服路徑。
pub fn static_path_of(id: &str) -> Option<&'static str> {
    PAGES
        .iter()
        .find(|p| p.id == id && p.renderer == Renderer::Static)
        .map(|p| p.path)
}

/// 列出以 `prefix` 開頭的靜態頁 id（宣告序）；例如 `"console.workflow."` → 13 筆。
pub fn static_ids_with_prefix(prefix: &str) -> Vec<&'static str> {
    PAGES
        .iter()
        .filter(|p| p.renderer == Renderer::Static && p.id.starts_with(prefix))
        .map(|p| p.id)
        .collect()
}

/// 依 id 查高基數族。
pub fn family_by_id(id: &str) -> Option<&'static PageFamily> {
    FAMILIES.iter().find(|f| f.id == id)
}

/// 靜態頁路徑轉為相對於 `depth` 層子目錄的連結（`depth`＝頁面在 `/console` 下的
/// 目錄深度；0＝根頁）。KEM 生成器以零 JS 相對連結互相引用，故路徑由 manifest 推導。
pub fn relative_static_path(path: &str, depth: usize) -> String {
    let rest = path.strip_prefix(STATIC_ROOT).unwrap_or(path);
    let rest = rest.trim_start_matches('/');
    format!("{}{}", "../".repeat(depth), rest)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn page_ids_are_unique() {
        let mut seen = HashSet::new();
        for p in PAGES {
            assert!(seen.insert(p.id), "duplicate page id: {}", p.id);
        }
    }

    #[test]
    fn dynamic_paths_are_unique() {
        let mut seen = HashSet::new();
        for p in PAGES.iter().filter(|p| p.renderer == Renderer::Dynamic) {
            assert!(seen.insert(p.path), "duplicate dynamic path: {}", p.path);
        }
    }

    #[test]
    fn nav_groups_are_closed_set() {
        let ids: HashSet<&str> = NAV_GROUPS.iter().map(|g| g.id).collect();
        for p in PAGES {
            if let Some(g) = p.nav_group {
                assert!(
                    ids.contains(g),
                    "page {} references unknown group {}",
                    p.id,
                    g
                );
            }
        }
    }

    #[test]
    fn every_group_has_at_least_one_page() {
        for g in NAV_GROUPS {
            let n = PAGES.iter().filter(|p| p.nav_group == Some(g.id)).count();
            assert!(n > 0, "nav group {} has no pages", g.id);
        }
    }

    #[test]
    fn public_gate_only_for_login() {
        for p in PAGES {
            if p.gate == Gate::Public {
                assert_eq!(p.id, "auth.login", "Public gate leaked to {}", p.id);
            }
        }
    }

    #[test]
    fn post_pages_are_exactly_two() {
        let posts: Vec<_> = PAGES.iter().filter(|p| p.method == Method::Post).collect();
        assert_eq!(
            posts.len(),
            2,
            "POST pages must be exactly bulk-review and ai-suggestion"
        );
        for p in posts {
            assert!(
                p.id == "review.bulk-result" || p.id == "ai.suggestion",
                "unexpected POST page {}",
                p.id
            );
        }
    }

    #[test]
    fn every_page_has_at_least_one_workflow() {
        for p in PAGES {
            assert!(!p.workflows.is_empty(), "page {} has no workflows", p.id);
        }
    }

    #[test]
    fn expansion_law_matches_measured_generator() {
        // 實測：claims=20、ledger=103 → generate() 回傳 45（24 固定＋20 主張＋1 帳本分頁）。
        assert_eq!(static_console_pages(20, 103), 45);
        // 無資料：24 固定＋0 主張＋1 帳本分頁（max(0,1) 保底一頁）。
        assert_eq!(static_console_pages(0, 0), 25);
        // HTML 檔數＝回傳值 − 1（console.css 計數）。
        assert_eq!(static_console_pages(20, 103) - 1, 44);
    }

    #[test]
    fn expansion_law_is_monotonic() {
        for c in [0usize, 1, 10, 100] {
            for l in [0usize, 1, 199, 200, 201, 1000] {
                assert!(static_console_pages(c, l) >= KEM_FIXED_PAGES);
                assert!(static_console_pages(c + 1, l) >= static_console_pages(c, l));
                assert!(static_console_pages(c, l + 1) >= static_console_pages(c, l));
            }
        }
    }

    #[test]
    fn families_cover_claim_and_ledger() {
        assert_eq!(FAMILIES.len(), 2);
        assert_eq!((FAMILIES[0].cardinality)(20, 103), 20);
        assert_eq!((FAMILIES[1].cardinality)(20, 103), 1);
        assert_eq!((FAMILIES[1].cardinality)(0, 0), 1);
        assert_eq!((FAMILIES[1].cardinality)(0, 200), 1);
        assert_eq!((FAMILIES[1].cardinality)(0, 201), 2);
    }

    #[test]
    fn page_by_id_resolves_seed_entries() {
        assert!(page_by_id("home.index").is_some());
        assert!(page_by_id("accounts.detail").is_some());
        assert!(page_by_id("console.index").is_some());
        assert!(page_by_id("nonexistent").is_none());
    }

    #[test]
    fn nav_href_override_applies_only_where_declared() {
        let pivot = page_by_id("reports.pivot").expect("pivot");
        assert_eq!(
            nav_href_of(pivot),
            "/reports/pivot?rows=account&cols=month&measure=debit"
        );
        let tx = page_by_id("tx.list").expect("tx.list");
        assert_eq!(nav_href_of(tx), "/transactions");
    }

    #[test]
    fn dynamic_page_count_matches_plan_seed() {
        // 組成：Public 1（login）＋GET 39（38 實測路由＋W3 前瞻登記之
        // accounts.detail；含 bankpay.batch-detail）＋POST 2＝42。
        // 路由對帳測試（W2）須於 W3 落地 /accounts/:id 後方成立。
        // 〔修訂 2026-11-10，計畫 1793033761021 B-5.0〕＋my.decisions
        // （D-6 自我稽核視圖，/my-decisions）→ Dynamic 43。
        let dynamic = PAGES
            .iter()
            .filter(|p| p.renderer == Renderer::Dynamic)
            .count();
        let static_html = PAGES
            .iter()
            .filter(|p| p.renderer == Renderer::Static)
            .count();
        assert_eq!(dynamic, 43);
        assert_eq!(static_html, 23);
    }

    #[test]
    fn static_workflow_and_role_families_are_complete() {
        let workflows = static_ids_with_prefix("console.workflow.");
        assert_eq!(workflows.len(), 13, "13 工作流頁必須全數登記");
        let roles = static_ids_with_prefix("console.role.");
        assert_eq!(roles.len(), 5, "5 角色視角頁必須全數登記");
        for id in workflows.iter().chain(roles.iter()) {
            let path = static_path_of(id).expect("static path");
            assert!(
                path.starts_with(STATIC_ROOT),
                "{id} 靜態路徑須位於 {STATIC_ROOT} 之下：{path}"
            );
        }
    }

    #[test]
    fn family_paths_are_derived_from_template() {
        let claims = family_by_id("kb.claim").expect("claim family");
        assert_eq!(claims.path(1), "/console/claims/1.html");
        let ledger = family_by_id("kb.ledger-chunk").expect("ledger family");
        assert_eq!(ledger.path(1), "/console/ledger-1.html");
        assert_eq!(ledger.path(12), "/console/ledger-12.html");
    }

    #[test]
    fn relative_static_path_resolves_depth() {
        assert_eq!(
            relative_static_path("/console/workflows/capture.html", 0),
            "workflows/capture.html"
        );
        assert_eq!(
            relative_static_path("/console/workflows/capture.html", 1),
            "../workflows/capture.html"
        );
        assert_eq!(
            relative_static_path("/console/index.html", 1),
            "../index.html"
        );
    }
}
