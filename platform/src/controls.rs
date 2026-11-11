#![forbid(unsafe_code)]

//! G3.2 (GP-02): one control library mapped across frameworks.
//!
//! Each control names the audit event types that evidence its operation and
//! every framework obligation it satisfies (ISO 27001 annex refs, internal
//! control numbers, bank-facing controls). The lint reuses the Finding
//! pattern from the kem governance module (rule/subject/detail) so findings
//! stay grep-friendly and machine-checkable.

/// One control: what it is, which audit events evidence it, which
/// frameworks it maps to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Control {
    pub control_id: &'static str,
    pub name: &'static str,
    /// Audit event types whose presence evidences this control operated.
    pub event_types: &'static [&'static str],
    /// Framework obligations satisfied (ISO 27001 annex / internal IC / bank).
    pub frameworks: &'static [&'static str],
}

/// The single control library (SSOT). Extend, never fork: new obligations map
/// onto existing controls first; a genuinely new control gets a new entry.
pub const CONTROL_LIBRARY: &[Control] = &[
    Control {
        control_id: "CTRL-POST",
        name: "傳票過帳控制",
        event_types: &["TRANSACTION_POSTED"],
        frameworks: &["ISO-27001-A.8.2", "INTERNAL-IC-01", "BANK-FSC-3"],
    },
    Control {
        control_id: "CTRL-IMPORT",
        name: "匯入批次控制",
        event_types: &["IMPORT_CONFIRMED"],
        frameworks: &["ISO-27001-A.8.9", "INTERNAL-IC-02", "BANK-FSC-3"],
    },
    Control {
        control_id: "CTRL-REVIEW",
        name: "覆核裁決控制",
        // REVIEW_BULK_SUMMARY（D-5/B-1）為同控制之批次摘要事件：每 POST
        // 一列，承載 batch_size/ok_count/fail_count/elapsed_ms。未映射將
        // 觸發 UNMAPPED_EVENT lint（治理稽核視其為未控制活動）。
        event_types: &["REVIEW_BULK", "REVIEW_BULK_SUMMARY"],
        frameworks: &["ISO-27001-A.5.3", "INTERNAL-IC-03"],
    },
    Control {
        control_id: "CTRL-CRED",
        name: "憑證管理控制",
        event_types: &["PASSWORD_RESET"],
        frameworks: &["ISO-27001-A.5.17", "INTERNAL-IC-04"],
    },
    Control {
        control_id: "CTRL-AI",
        name: "AI 建議治理控制",
        event_types: &["AI_SUGGEST_REQUESTED"],
        frameworks: &["INTERNAL-IC-05"],
    },
];

/// Map an audit event type to its evidencing control id.
pub fn control_for_event(event_type: &str) -> Option<&'static str> {
    CONTROL_LIBRARY
        .iter()
        .find(|c| c.event_types.contains(&event_type))
        .map(|c| c.control_id)
}

/// Lint finding (kem gov.rs Finding shape: rule / subject / detail).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ControlFinding {
    pub rule: &'static str,
    pub subject: String,
    pub detail: String,
}

/// Lint the observed audit event types (one period) against the library.
///
/// - `UNMAPPED_EVENT`: an event type was observed but no control evidences
///   it — uncontrolled activity, must be mapped or explained.
/// - `INACTIVE_CONTROL`: a library control has zero observed events in the
///   period — either the control never fires (fine for rare controls) or it
///   is not operating (CCM escalation). Reported for human judgement, not
///   auto-alarmed.
pub fn lint_control_coverage(observed_event_types: &[&str]) -> Vec<ControlFinding> {
    let mut findings = Vec::new();
    for observed in observed_event_types {
        if control_for_event(observed).is_none() {
            findings.push(ControlFinding {
                rule: "UNMAPPED_EVENT",
                subject: (*observed).to_string(),
                detail: "已觀察的稽核事件未映射到任何控制".to_string(),
            });
        }
    }
    for control in CONTROL_LIBRARY {
        let active = observed_event_types
            .iter()
            .any(|o| control.event_types.contains(o));
        if !active {
            findings.push(ControlFinding {
                rule: "INACTIVE_CONTROL",
                subject: control.control_id.to_string(),
                detail: format!("{} 於本期無觀察事件", control.name),
            });
        }
    }
    findings
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn observed_events_map_to_controls() {
        assert_eq!(control_for_event("TRANSACTION_POSTED"), Some("CTRL-POST"));
        assert_eq!(control_for_event("REVIEW_BULK"), Some("CTRL-REVIEW"));
        assert_eq!(control_for_event("MYSTERY_EVENT"), None);
    }

    #[test]
    fn every_library_control_maps_at_least_one_event_and_framework() {
        for control in CONTROL_LIBRARY {
            assert!(
                !control.event_types.is_empty(),
                "{} must evidence via at least one event type",
                control.control_id
            );
            assert!(
                !control.frameworks.is_empty(),
                "{} must map to at least one framework obligation",
                control.control_id
            );
        }
    }

    #[test]
    fn lint_flags_unmapped_events_and_inactive_controls() {
        let observed = ["TRANSACTION_POSTED", "MYSTERY_EVENT"];
        let findings = lint_control_coverage(&observed);

        assert!(findings.contains(&ControlFinding {
            rule: "UNMAPPED_EVENT",
            subject: "MYSTERY_EVENT".to_string(),
            detail: "已觀察的稽核事件未映射到任何控制".to_string(),
        }));

        // Every control except CTRL-POST is inactive in this period.
        let inactive: Vec<&str> = findings
            .iter()
            .filter(|f| f.rule == "INACTIVE_CONTROL")
            .map(|f| f.subject.as_str())
            .collect();
        assert_eq!(
            inactive,
            vec!["CTRL-IMPORT", "CTRL-REVIEW", "CTRL-CRED", "CTRL-AI"]
        );
    }

    #[test]
    fn full_observation_yields_no_findings() {
        let observed: Vec<&str> = CONTROL_LIBRARY
            .iter()
            .flat_map(|c| c.event_types.iter().copied())
            .collect();
        assert!(lint_control_coverage(&observed).is_empty());
    }
}
