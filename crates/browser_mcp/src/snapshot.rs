#![forbid(unsafe_code)]
//! 無障礙樹快照：將扁平的 AX 節點清單轉為縮排文字與元素 ref 對照。
//!
//! 與 CDP 型別解耦（`AxNodeFlat` 為本模組的輸入），以便單元測試
//! 不需真實瀏覽器即可驗證格式化、ref 指派與截斷行為。

use std::collections::BTreeMap;
use std::collections::HashMap;

/// 扁平化的單一 AX 節點。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AxNodeFlat {
    /// CDP `AXNodeId` 字串表示。
    pub id: String,
    /// 父節點 id（根為 `None`）。
    pub parent_id: Option<String>,
    /// 角色（如 `button`、`textbox`、`text`）。
    pub role: String,
    /// 無障礙名稱。
    pub name: String,
    /// 目前值（若有）。
    pub value: Option<String>,
    /// 關注的狀態旗標（如 `checked=true`、`disabled`）。
    pub flags: Vec<String>,
    /// CDP 標記為 ignored 的節點不輸出，但其子節點仍會下探。
    pub ignored: bool,
    /// 對應 DOM 節點（有值者才指派 ref）。
    pub backend_node_id: Option<i64>,
}

/// 快照輸出上限（保護模型上下文）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Limits {
    /// 輸出節點數上限。
    pub max_nodes: usize,
    /// 輸出字元數上限。
    pub max_chars: usize,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            max_nodes: 2_000,
            max_chars: 60_000,
        }
    }
}

/// 渲染結果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Snapshot {
    /// 縮排文字（含截斷註記）。
    pub text: String,
    /// `ref` → `backend_node_id` 對照（ref 形式 `e1`、`e2`、…）。
    pub refs: BTreeMap<String, i64>,
    /// 是否因上限而截斷。
    pub truncated: bool,
}

/// 單一節點名稱的長度上限，避免超長文字撐爆輸出。
const MAX_NAME_CHARS: usize = 200;

/// 將扁平節點清單渲染為快照。
pub fn render(nodes: &[AxNodeFlat], limits: &Limits) -> Snapshot {
    let mut children: HashMap<&str, Vec<&AxNodeFlat>> = HashMap::new();
    let mut known: HashMap<&str, &AxNodeFlat> = HashMap::new();
    for node in nodes {
        known.insert(node.id.as_str(), node);
    }
    let mut roots: Vec<&AxNodeFlat> = Vec::new();
    for node in nodes {
        match node.parent_id.as_deref() {
            Some(parent) if known.contains_key(parent) => {
                children.entry(parent).or_default().push(node);
            }
            _ => roots.push(node),
        }
    }

    let mut out = String::new();
    let mut refs: BTreeMap<String, i64> = BTreeMap::new();
    let mut counters = Counters {
        nodes: 0,
        refs: 0,
        truncated: false,
    };
    for root in roots {
        render_node(
            root,
            0,
            limits,
            &children,
            &mut out,
            &mut refs,
            &mut counters,
        );
        if counters.truncated {
            break;
        }
    }

    if counters.truncated {
        out.push_str(
            "\n…（快照已截斷：節點或字元數達上限。處理方式：縮小頁面範圍後重新呼叫 snapshot。）\n",
        );
    }
    if counters.nodes == 0 {
        out.push_str("（無可顯示的無障礙節點：可能原因：頁面尚未載入或內容為空白。處理方式：先 navigate 並以 wait_for 等待後重試。）\n");
    }

    Snapshot {
        text: out,
        refs,
        truncated: counters.truncated,
    }
}

struct Counters {
    nodes: usize,
    refs: usize,
    truncated: bool,
}

#[allow(clippy::too_many_arguments)]
fn render_node(
    node: &AxNodeFlat,
    depth: usize,
    limits: &Limits,
    children: &HashMap<&str, Vec<&AxNodeFlat>>,
    out: &mut String,
    refs: &mut BTreeMap<String, i64>,
    counters: &mut Counters,
) {
    if counters.truncated {
        return;
    }
    let mut child_depth = depth;

    if !node.ignored {
        if counters.nodes >= limits.max_nodes || out.chars().count() >= limits.max_chars {
            counters.truncated = true;
            return;
        }
        let mut line = String::new();
        line.push_str(&"  ".repeat(depth));
        line.push_str("- ");
        line.push_str(&sanitize(&node.role, MAX_NAME_CHARS));
        if !node.name.is_empty() {
            line.push_str(" \"");
            line.push_str(&sanitize(&node.name, MAX_NAME_CHARS));
            line.push('"');
        }
        if let Some(ref_id) = node.backend_node_id {
            let ref_name = format!("e{}", counters.refs + 1);
            refs.insert(ref_name.clone(), ref_id);
            counters.refs += 1;
            line.push_str(&format!(" [ref={ref_name}]"));
        }
        if let Some(value) = &node.value {
            let text = sanitize(value, MAX_NAME_CHARS);
            if !text.is_empty() {
                line.push_str(&format!(" {{value=\"{text}\"}}"));
            }
        }
        if !node.flags.is_empty() {
            line.push_str(&format!(" {{{}}}", node.flags.join("; ")));
        }
        line.push('\n');
        out.push_str(&line);
        counters.nodes += 1;
        child_depth = depth + 1;
    }

    if let Some(kids) = children.get(node.id.as_str()) {
        for kid in kids {
            render_node(kid, child_depth, limits, children, out, refs, counters);
            if counters.truncated {
                return;
            }
        }
    }
}

/// 去除控制字元並限制長度，避免輸出注入與爆量。
fn sanitize(input: &str, max_chars: usize) -> String {
    let cleaned: String = input
        .chars()
        .map(|c| match c {
            '\n' | '\r' | '\t' => ' ',
            c if c.is_control() => ' ',
            c => c,
        })
        .collect();
    let trimmed = cleaned.trim();
    if trimmed.chars().count() <= max_chars {
        trimmed.to_string()
    } else {
        let mut short: String = trimmed.chars().take(max_chars).collect();
        short.push('…');
        short
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flat(
        id: &str,
        parent: Option<&str>,
        role: &str,
        name: &str,
        backend: Option<i64>,
    ) -> AxNodeFlat {
        AxNodeFlat {
            id: id.to_string(),
            parent_id: parent.map(str::to_string),
            role: role.to_string(),
            name: name.to_string(),
            value: None,
            flags: Vec::new(),
            ignored: false,
            backend_node_id: backend,
        }
    }

    #[test]
    fn renders_nested_tree_with_refs_in_document_order() {
        let nodes = vec![
            flat("1", None, "RootWebArea", "首頁", None),
            flat("2", Some("1"), "heading", "交易清單", Some(101)),
            flat("3", Some("2"), "StaticText", "交易清單", None),
            flat("4", Some("1"), "button", "新增", Some(202)),
        ];
        let snap = render(&nodes, &Limits::default());
        assert_eq!(snap.refs.get("e1"), Some(&101));
        assert_eq!(snap.refs.get("e2"), Some(&202));
        assert!(snap.text.contains("- RootWebArea \"首頁\"\n"));
        assert!(snap.text.contains("  - heading \"交易清單\" [ref=e1]\n"));
        assert!(snap.text.contains("    - StaticText \"交易清單\"\n"));
        assert!(snap.text.contains("  - button \"新增\" [ref=e2]\n"));
        assert!(!snap.truncated);
    }

    #[test]
    fn ignored_node_is_skipped_but_children_survive() {
        let nodes = vec![
            flat("1", None, "RootWebArea", "", None),
            AxNodeFlat {
                ignored: true,
                ..flat("2", Some("1"), "generic", "", Some(5))
            },
            flat("3", Some("2"), "link", "說明", Some(6)),
        ];
        let snap = render(&nodes, &Limits::default());
        assert!(!snap.text.contains("generic"));
        assert!(snap.text.contains("  - link \"說明\" [ref=e1]"));
        assert_eq!(snap.refs.len(), 1);
    }

    #[test]
    fn truncates_at_node_limit() {
        let mut nodes = vec![flat("1", None, "RootWebArea", "root", None)];
        for i in 0..10 {
            nodes.push(flat(
                &format!("n{i}"),
                Some("1"),
                "button",
                &format!("b{i}"),
                Some(i),
            ));
        }
        let snap = render(
            &nodes,
            &Limits {
                max_nodes: 4,
                max_chars: 60_000,
            },
        );
        assert!(snap.truncated);
        assert_eq!(
            snap.refs.len(),
            3,
            "root 無 backend id，其餘 3 個節點各得一個 ref"
        );
        assert!(snap.text.contains("快照已截斷"));
    }

    #[test]
    fn sanitizes_control_chars_and_long_names() {
        let long = "x".repeat(MAX_NAME_CHARS + 50);
        let nodes = vec![AxNodeFlat {
            name: format!("a\nb\t{long}"),
            ..flat("1", None, "button", "", Some(1))
        }];
        let snap = render(&nodes, &Limits::default());
        assert!(!snap.text.contains('\t'));
        assert!(snap.text.contains("…"));
        assert!(snap.text.lines().count() <= 2);
    }

    #[test]
    fn empty_input_yields_actionable_note() {
        let snap = render(&[], &Limits::default());
        assert!(snap.refs.is_empty());
        assert!(snap.text.contains("無可顯示的無障礙節點"));
    }
}
