#![forbid(unsafe_code)]
//! 導覽模型（計畫 1793033568961 W2；grill-B 裁定：middleware 注入）。
//!
//! 單一事實來源＝`console_manifest`（`NAV_GROUPS`＋`PAGES`）；本模組依
//! `NavFlags` 過濾生成每請求的 [`NavModel`]——**無權限入口不渲染**（根治
//! 死連結：導覽可見性＝gate 過濾，而非渲染後擋下）。
//!
//! 計算位置：`auth::require_session` middleware 每認證請求一次
//! （NavFlags 查詢＋純記憶體過濾）；handler 一行從 Extension 複製。

use console_manifest::{Gate, NAV_GROUPS, NavKind, PAGES, STUDIOS};

use crate::api::NavFlags;

/// 單一導覽項（來源＝`PageSpec`；金額/語彙零使用者輸入）。
#[derive(Debug, Clone)]
pub struct NavItem {
    pub id: &'static str,
    pub href: String,
    pub label: &'static str,
    /// 目前路徑比對命中時為 true（模板輸出 `aria-current="page"`）。
    pub active: bool,
}

/// 導覽群組檢視（`is_menu`＝`<details>` 下拉；否則頂層連結）。
#[derive(Debug, Clone)]
pub struct NavGroupView {
    pub id: &'static str,
    pub label: &'static str,
    /// 圖標 key（Lucide；2026-11-14 UI-1）。
    pub icon: &'static str,
    /// 預渲染 SVG（`icons::svg`；建構時一次，模板零過濾器）。
    pub icon_svg: String,
    pub is_menu: bool,
    pub items: Vec<NavItem>,
}

/// 工作室檢視（導覽 IA 第一層：studio header + 所屬群組）。
#[derive(Debug, Clone)]
pub struct StudioView {
    pub id: &'static str,
    pub label: &'static str,
    /// 圖標 key（Lucide；2026-11-14 UI-1）。
    pub icon: &'static str,
    /// 預渲染 SVG（`icons::svg`；建構時一次，模板零過濾器）。
    pub icon_svg: String,
    pub groups: Vec<NavGroupView>,
}
/// 完整導覽模型（每請求一份；Askama 模板直接迭代）。
///
/// IA（使用者裁定 studio + section 兩層）：`home`＝總覽置頂連結；
/// `top`＝overview 群組其餘自我服務頁（我的裁決紀錄，Gate::All）；
/// `studios`＝5 大工作室（順序＝`STUDIOS` 宣告順序），其下為所屬群組
/// （gate 過濾；無權限入口不渲染——根治死連結）。
#[derive(Debug, Clone)]
pub struct NavModel {
    pub home: NavItem,
    pub top: Vec<NavItem>,
    pub studios: Vec<StudioView>,
    /// 麵包屑（2026-11-13）：目前頁所屬工作室＋群組（`None`＝總覽/自我服務頁）。
    pub section: Option<SectionLabel>,
    /// 目前角色徽章（bootstrap admin 為空字串時不渲染）。
    pub role_code: String,
}

/// 麵包屑段（studio → group 兩級標籤；頁名由各頁模板自行渲染）。
#[derive(Debug, Clone)]
pub struct SectionLabel {
    pub studio_label: &'static str,
    pub group_label: &'static str,
}

impl NavModel {
    /// 由 manifest＋權限旗標建構（無 I/O；除 `role_code` 複製外全為借用過濾）。
    pub fn build(flags: &NavFlags, current_path: &str) -> NavModel {
        // 群組 → 過濾後檢視（快取於閉包內部；group.studio 決定所屬工作室）。
        let group_views: Vec<(&console_manifest::NavGroup, Option<NavGroupView>)> = NAV_GROUPS
            .iter()
            .map(|g| {
                let mut items: Vec<NavItem> = Vec::new();
                for p in PAGES.iter().filter(|p| p.nav_group == Some(g.id)) {
                    if !gate_allows(p.gate, flags) {
                        continue;
                    }
                    let href = console_manifest::nav_href_of(p).to_string();
                    let active = is_active_path(current_path, &href);
                    items.push(NavItem {
                        id: p.id,
                        href,
                        label: p.title,
                        active,
                    });
                }
                // 群組內所有項目皆被過濾 → 整組不渲染（連空 dropdown 都不出現）。
                let view = if items.is_empty() {
                    None
                } else {
                    Some(NavGroupView {
                        id: g.id,
                        label: g.label,
                        icon: g.icon,
                        icon_svg: crate::icons::svg(g.icon, "ico"),
                        is_menu: g.kind == NavKind::Menu,
                        items,
                    })
                };
                (g, view)
            })
            .collect();

        // home + top：overview 群組（studio=""）拆解——home.index 提升為
        // 置頂首頁；其餘自我服務頁（我的裁決紀錄）成為頂層平面連結。
        let mut home = NavItem {
            id: "",
            href: "/".to_string(),
            label: "總覽",
            active: current_path == "/",
        };
        let mut top: Vec<NavItem> = Vec::new();
        let mut studios: Vec<StudioView> = Vec::new();

        for s in STUDIOS {
            let mut groups: Vec<NavGroupView> = Vec::new();
            for (g, view) in &group_views {
                if g.studio == s.id
                    && let Some(v) = view
                {
                    groups.push(v.clone());
                }
            }
            // 工作室內所有群組皆被過濾 → 整室不渲染。
            if !groups.is_empty() {
                studios.push(StudioView {
                    id: s.id,
                    label: s.label,
                    icon: s.icon,
                    icon_svg: crate::icons::svg(s.icon, "ico"),
                    groups,
                });
            }
        }

        for (g, view) in &group_views {
            if g.id == "overview"
                && let Some(v) = view
            {
                for it in &v.items {
                    if it.id == "home.index" {
                        home = it.clone();
                    } else {
                        top.push(it.clone());
                    }
                }
            }
        }

        // home.index 未過濾成功（例如群組空）→ 防禦性預設。
        if home.id.is_empty() {
            home = NavItem {
                id: "home.index",
                href: "/".to_string(),
                label: "總覽",
                active: current_path == "/",
            };
        }

        // 麵包屑：目前路徑所屬群組 → 其 studio（含群組 href）。
        let mut section: Option<SectionLabel> = None;
        for (g, view) in &group_views {
            if view.is_some()
                && let Some(v) = view
                && v.items.iter().any(|it| it.active)
            {
                let studio_label = STUDIOS
                    .iter()
                    .find(|s| s.id == g.studio)
                    .map(|s| s.label)
                    .unwrap_or("");
                section = Some(SectionLabel {
                    studio_label,
                    group_label: g.label,
                });
                break;
            }
        }

        NavModel {
            home,
            top,
            studios,
            section,
            role_code: flags.role_code.clone(),
        }
    }
}

/// 頁面閘 → 權限旗標（fail-closed：Public 頁不入導覽）。
fn gate_allows(gate: Gate, flags: &NavFlags) -> bool {
    match gate {
        Gate::Public => false,
        Gate::All => true,
        Gate::Read => flags.ledger_read,
        Gate::Write => flags.ledger_write,
        Gate::Manage => flags.manage_users,
    }
}

/// aria-current 路徑比對：精確命中或路徑前綴（`/accounts` 命中 `/accounts/:id`）。
/// `href == "/"` 僅精確命中（否則所有頁都會命中首頁）。
fn is_active_path(current: &str, href: &str) -> bool {
    if href == "/" {
        return current == "/";
    }
    current == href || current.starts_with(&format!("{href}/"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flags(read: bool, write: bool, manage: bool) -> NavFlags {
        NavFlags {
            ledger_read: read,
            ledger_write: write,
            manage_users: manage,
            role_code: "ADMIN".to_string(),
        }
    }

    /// 工作室輔助：取得指定 id 的 StudioView。
    fn studio<'a>(m: &'a NavModel, id: &str) -> Option<&'a StudioView> {
        m.studios.iter().find(|s| s.id == id)
    }

    #[test]
    fn visitor_flags_yield_minimal_nav() {
        // 全 false（未登入者不會走到此，但防禦性：零權限→僅 All 頁）。
        let m = NavModel::build(&flags(false, false, false), "/");
        assert!(m.home.id == "home.index" && m.home.active, "總覽置頂");
        assert!(studio(&m, "biz").is_some(), "biz 含 All 頁（檢索）");
        assert!(studio(&m, "ops").is_some(), "ops 含 All 頁（系統/知識）");
        assert!(studio(&m, "bank").is_none(), "Read 頁不應對零權限者出現");
        assert!(studio(&m, "gov").is_none(), "Manage 頁不應對零權限者出現");
    }

    #[test]
    fn read_only_user_never_sees_audit_link() {
        // 根治死連結：store1（LEDGER_READ）不得見 /audit（Manage 閘）——
        // gov 工作室所有項目皆被過濾 → 整室不渲染（連空區段都不出現）。
        let m = NavModel::build(&flags(true, false, false), "/transactions");
        assert!(
            studio(&m, "gov").is_none(),
            "gov 工作室對 LEDGER_READ-only 者不應渲染"
        );
        assert!(
            !m.studios
                .iter()
                .flat_map(|s| s.groups.iter())
                .flat_map(|g| g.items.iter())
                .any(|it| it.id == "audit.page"),
            "稽核連結對 LEDGER_READ-only 者渲染＝死連結（B7a 回歸）"
        );
    }

    #[test]
    fn manage_user_sees_governance_and_admin() {
        let m = NavModel::build(&flags(true, true, true), "/audit");
        let gov = studio(&m, "gov").expect("gov 工作室");
        assert!(
            gov.groups
                .iter()
                .flat_map(|g| g.items.iter())
                .any(|it| it.id == "audit.page" && it.active)
        );
        assert!(
            studio(&m, "gov")
                .expect("gov")
                .groups
                .iter()
                .any(|g| g.id == "admin")
        );
        assert!(
            studio(&m, "gov")
                .expect("gov")
                .groups
                .iter()
                .any(|g| g.id == "apihub")
        );
    }

    #[test]
    fn aria_current_exact_and_prefix() {
        let f = flags(true, false, false);
        assert!(
            NavModel::build(&f, "/transactions")
                .studios
                .iter()
                .flat_map(|s| s.groups.iter())
                .flat_map(|g| g.items.iter())
                .any(|it| it.id == "tx.list" && it.active)
        );
        // 前綴邊界：/accounts/ACC-1 命中 /accounts；/accounts-foo 不命中。
        assert!(
            NavModel::build(&f, "/accounts/ACC-1")
                .studios
                .iter()
                .flat_map(|s| s.groups.iter())
                .flat_map(|g| g.items.iter())
                .any(|it| it.id == "accounts.list" && it.active)
        );
        assert!(
            !NavModel::build(&f, "/accounts-foo")
                .studios
                .iter()
                .flat_map(|s| s.groups.iter())
                .flat_map(|g| g.items.iter())
                .any(|it| it.active)
        );
        // 首頁僅精確命中。
        assert!(NavModel::build(&f, "/").home.active);
        assert!(!NavModel::build(&f, "/transactions").home.active);
    }

    #[test]
    fn knowledge_group_present_for_all_authenticated() {
        let m = NavModel::build(&flags(false, false, false), "/");
        let ops = studio(&m, "ops").expect("ops 工作室");
        assert!(
            ops.groups
                .iter()
                .flat_map(|g| g.items.iter())
                .any(|it| it.href == "/console/index.html")
        );
    }

    #[test]
    fn home_item_is_overview_top_link() {
        let m = NavModel::build(&flags(true, true, true), "/");
        assert_eq!(m.home.id, "home.index");
        assert_eq!(m.home.href, "/");
        assert_eq!(m.home.label, "總覽");
        // home 不重複出現在工作室區段。
        assert!(
            !m.studios
                .iter()
                .flat_map(|s| s.groups.iter())
                .flat_map(|g| g.items.iter())
                .any(|it| it.id == "home.index")
        );
    }
}
