#![forbid(unsafe_code)]

//! Agent policy — low-risk auto-approval gate.
//!
//! **Gated by DEC-15.** Until DEC-15 resolves, this crate operates in
//! advisory-only mode (no auto-approval). Auto-approval of high-risk,
//! paid, or sensitive-classified actions is never permitted.

/// Risk classification for an action.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Whether an action requires human approval.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApprovalDecision {
    AutoApprove,
    RequireHuman,
    Reject,
}

/// Classification of data involved.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataClassification {
    Public,
    Internal,
    Confidential,
    Restricted,
}

/// Agent recommendation (advisory).
#[derive(Debug, Clone)]
pub struct AgentRecommendation {
    pub risk_level: RiskLevel,
    pub confidence: f32,
    pub reason: String,
    pub classification: DataClassification,
}

/// Auto-approval gate. **Feature-gated by DEC-15.**
pub struct AutoApprovalGate {
    /// Whether auto-approval is enabled (DEC-15 must resolve to true).
    enabled: bool,
    /// Confidence threshold for auto-approval (only applies to Low risk).
    confidence_threshold: f32,
}

impl AutoApprovalGate {
    pub fn new(enabled: bool, confidence_threshold: f32) -> Self {
        Self {
            enabled,
            confidence_threshold,
        }
    }

    /// Advisory-only mode (DEC-15 not resolved).
    pub fn advisory_only() -> Self {
        Self {
            enabled: false,
            confidence_threshold: 1.0,
        }
    }

    /// Decide whether an action can be auto-approved.
    /// High/Critical risk, paid actions, and sensitive classifications
    /// always require human approval. Fail-closed on any uncertainty.
    pub fn decide(&self, recommendation: &AgentRecommendation, is_paid: bool) -> ApprovalDecision {
        // Fail-closed: if not enabled, always require human.
        if !self.enabled {
            return ApprovalDecision::RequireHuman;
        }

        // High-risk / paid / sensitive → human required.
        if is_paid
            || matches!(recommendation.risk_level, RiskLevel::High | RiskLevel::Critical)
            || matches!(
                recommendation.classification,
                DataClassification::Confidential | DataClassification::Restricted
            )
        {
            return ApprovalDecision::RequireHuman;
        }

        // Low/Medium risk: auto-approve only if confidence exceeds threshold.
        if recommendation.confidence >= self.confidence_threshold {
            ApprovalDecision::AutoApprove
        } else {
            ApprovalDecision::RequireHuman
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn advisory_only_never_auto_approves() {
        let gate = AutoApprovalGate::advisory_only();
        let rec = AgentRecommendation {
            risk_level: RiskLevel::Low,
            confidence: 0.99,
            reason: "test".into(),
            classification: DataClassification::Public,
        };
        assert_eq!(gate.decide(&rec, false), ApprovalDecision::RequireHuman);
    }

    #[test]
    fn paid_action_requires_human() {
        let gate = AutoApprovalGate::new(true, 0.9);
        let rec = AgentRecommendation {
            risk_level: RiskLevel::Low,
            confidence: 0.99,
            reason: "test".into(),
            classification: DataClassification::Public,
        };
        assert_eq!(gate.decide(&rec, true), ApprovalDecision::RequireHuman);
    }

    #[test]
    fn high_risk_requires_human() {
        let gate = AutoApprovalGate::new(true, 0.9);
        let rec = AgentRecommendation {
            risk_level: RiskLevel::High,
            confidence: 0.99,
            reason: "test".into(),
            classification: DataClassification::Public,
        };
        assert_eq!(gate.decide(&rec, false), ApprovalDecision::RequireHuman);
    }
}