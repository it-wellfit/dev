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
            || matches!(
                recommendation.risk_level,
                RiskLevel::High | RiskLevel::Critical
            )
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

// ============================================================================
// G2.5 (HP-09): governed agent access — rate limiting, draft-only writes,
// and audit markers. DEC-15 stays fail-closed throughout: nothing here can
// auto-post to the ledger; agent output always lands as a draft.
// ============================================================================

/// Sliding-window rate limiter for one agent identity.
pub struct AgentRateLimiter {
    max_requests: u32,
    window: std::time::Duration,
    hits: std::sync::Mutex<std::collections::VecDeque<std::time::Instant>>,
}

/// Outcome of a rate-limit check.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RateDecision {
    Allow,
    RateLimited,
}

impl AgentRateLimiter {
    pub fn new(max_requests: u32, window: std::time::Duration) -> Self {
        Self {
            max_requests,
            window,
            hits: std::sync::Mutex::new(std::collections::VecDeque::new()),
        }
    }

    /// Check and (on allow) consume one request slot at instant `now`.
    /// `now` is caller-supplied so tests and schedulers stay deterministic.
    pub fn check(&self, now: std::time::Instant) -> RateDecision {
        let mut hits = self.hits.lock().unwrap_or_else(|p| p.into_inner());
        let horizon = now - self.window;
        while hits.front().is_some_and(|&hit| hit < horizon) {
            hits.pop_front();
        }
        if hits.len() >= self.max_requests as usize {
            return RateDecision::RateLimited;
        }
        hits.push_back(now);
        RateDecision::Allow
    }
}

/// Where an agent write must land. Single-variant by design: DEC-15 keeps
/// every agent write in the human adoption queue.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WriteDisposition {
    /// Write to the draft store; a named human must adopt before posting.
    DraftRequired,
}

/// Fail-closed disposition: independent of risk level, confidence, or any
/// future flag, agent writes never post directly to the ledger.
pub fn write_disposition(
    _risk: RiskLevel,
    _confidence: f32,
    _auto_approval_enabled: bool,
) -> WriteDisposition {
    WriteDisposition::DraftRequired
}

/// Structured audit marker emitted with every governed agent request.
#[derive(Debug, Clone)]
pub struct AgentAuditMarker {
    pub agent_id: String,
    pub action: String,
    pub classification: DataClassification,
    pub risk_level: RiskLevel,
}

impl AgentAuditMarker {
    /// Single-line, grep-friendly audit event (key=value pairs).
    pub fn audit_event(&self) -> String {
        format!(
            "AGENT_ACT agent={} action={} class={:?} risk={:?}",
            self.agent_id, self.action, self.classification, self.risk_level
        )
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

    // ------------------------------------------------------------------
    // G2.5 (HP-09): rate limiting, draft-only writes, audit markers.
    // ------------------------------------------------------------------

    #[test]
    fn rate_limiter_blocks_burst_and_recovers_after_window() {
        let limiter = AgentRateLimiter::new(3, std::time::Duration::from_secs(60));
        let t0 = std::time::Instant::now();

        // Three requests fill the window.
        for i in 0..3 {
            assert_eq!(
                limiter.check(t0 + std::time::Duration::from_millis(i * 10)),
                RateDecision::Allow,
                "request {i} within budget must be allowed"
            );
        }
        // Fourth request inside the window is rejected.
        assert_eq!(
            limiter.check(t0 + std::time::Duration::from_millis(50)),
            RateDecision::RateLimited,
            "burst beyond the window budget must be limited"
        );
        // Sliding window: the first hit expires at t0+60s.
        assert_eq!(
            limiter.check(t0 + std::time::Duration::from_secs(61)),
            RateDecision::Allow,
            "window must slide: expired hits free budget"
        );
    }

    #[test]
    fn agent_writes_are_always_draft_only() {
        // DEC-15 fail-closed: no risk level, confidence, or flag changes the
        // disposition — agent writes land as drafts for human adoption.
        for risk in [
            RiskLevel::Low,
            RiskLevel::Medium,
            RiskLevel::High,
            RiskLevel::Critical,
        ] {
            assert_eq!(
                write_disposition(risk, 1.0, false),
                WriteDisposition::DraftRequired
            );
        }
    }

    #[test]
    fn audit_marker_renders_agent_identity_and_classification() {
        let marker = AgentAuditMarker {
            agent_id: "kem-vet-01".into(),
            action: "draft_suggest".into(),
            classification: DataClassification::Internal,
            risk_level: RiskLevel::Medium,
        };
        let line = marker.audit_event();
        assert!(line.contains("agent=kem-vet-01"), "{line}");
        assert!(line.contains("action=draft_suggest"), "{line}");
        assert!(line.contains("class=Internal"), "{line}");
        assert!(line.contains("risk=Medium"), "{line}");
    }
}
