#![forbid(unsafe_code)]
//! G4.9 (D3-04) — runtime feature flags.
//!
//! Env-driven (`AUXBRIDGE_FLAG_*`); unset or unrecognized values fall back to
//! the FLAG defaults (all ON). Each flag has a documented "off" behavior and a
//! test (see `FLAG` section of the execution plan).

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Flags {
    /// Vector leg of hybrid search (`search_hybrid=off` → deterministic BM25-only).
    pub search_hybrid: bool,
    /// RRF fusion (`search_rrf=off` → weighted α-fusion).
    pub search_rrf: bool,
    /// CCM chain/signature verification (`ccm_chain_verify=off` → indicator null + note).
    pub ccm_chain_verify: bool,
}

impl Default for Flags {
    fn default() -> Self {
        Self {
            search_hybrid: true,
            search_rrf: true,
            ccm_chain_verify: true,
        }
    }
}

impl Flags {
    /// Read `AUXBRIDGE_FLAG_*` from the environment.
    pub fn from_env() -> Self {
        let d = Self::default();
        Self {
            search_hybrid: env_flag("AUXBRIDGE_FLAG_SEARCH_HYBRID", d.search_hybrid),
            search_rrf: env_flag("AUXBRIDGE_FLAG_SEARCH_RRF", d.search_rrf),
            ccm_chain_verify: env_flag("AUXBRIDGE_FLAG_CCM_CHAIN_VERIFY", d.ccm_chain_verify),
        }
    }

    /// Parse one flag value. Returns `None` for unrecognized input so callers
    /// can keep the default instead of silently choosing a side.
    pub fn parse(value: &str) -> Option<bool> {
        match value.trim().to_ascii_lowercase().as_str() {
            "1" | "true" | "on" | "yes" | "enabled" => Some(true),
            "0" | "false" | "off" | "no" | "disabled" => Some(false),
            _ => None,
        }
    }
}

fn env_flag(key: &str, default: bool) -> bool {
    std::env::var(key)
        .ok()
        .and_then(|v| Flags::parse(&v))
        .unwrap_or(default)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_accepts_common_spellings() {
        assert_eq!(Flags::parse("on"), Some(true));
        assert_eq!(Flags::parse(" TRUE "), Some(true));
        assert_eq!(Flags::parse("enabled"), Some(true));
        assert_eq!(Flags::parse("off"), Some(false));
        assert_eq!(Flags::parse("0"), Some(false));
        assert_eq!(Flags::parse("maybe"), None);
    }

    #[test]
    fn default_is_fail_safe_on() {
        let f = Flags::default();
        assert!(f.search_hybrid && f.search_rrf && f.ccm_chain_verify);
    }
}
