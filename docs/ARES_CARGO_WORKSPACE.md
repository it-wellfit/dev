# ARES_CARGO_WORKSPACE.md

## ARES Cargo Workspace — Crate Responsibilities and Dependencies

**Version**: 1.0-draft  
**Date**: 2026-11-01  
**Status**: Proposed target design — not the current workspace manifest
**Role**: Design proposal only; reconcile workspace boundary, crate inventory, and dependencies before implementation.

---

## 1. Workspace Overview

The ARES workspace uses Cargo workspace with `resolver = "3"` (Rust 2024 edition). All crates inherit version, edition, license, and rust-version from `[workspace.package]`.

### 1.1 Workspace Root Cargo.toml

```toml
[workspace]
resolver = "3"
members = [
    "crates/ares-types",
    "crates/ares-errors",
    "crates/ares-config",
    "crates/ares-provider-api",
    "crates/ares-provider-registry",
    "crates/ares-artifact",
    "crates/ares-evidence",
    "crates/ares-storage-sqlite",
    "crates/ares-scheduler",
    "crates/ares-policy",
    "crates/ares-mcp-gateway",
    "crates/ares-cli",
    "apps/ares",
    "apps/ares-cli",
    "apps/ares-mcp",
    "apps/ares-bench",
    "providers/rea",
    "providers/ghidra",
    "providers/rizin",
]
exclude = ["upstream/rea-rust"]

[workspace.package]
version = "0.1.0"
edition = "2024"
rust-version = "1.93"
license = "MIT"
publish = false

[workspace.dependencies]
# Core
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1", features = ["full"] }
anyhow = "1.0"
thiserror = "2.0"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
async-trait = "0.1"
uuid = { version = "1.6", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }

# Binary analysis
goblin = "0.10"
object = "0.39"
scroll = "0.13"
iced-x86 = "1.22"
rustc-demangle = "0.1"
petgraph = "0.8"
wasmparser = "0.247"

# Storage
rusqlite = { version = "0.32", features = ["bundled"] }

# MCP
rmcp = { version = "3.8", features = ["server", "transport-io", "schemars"] }

# AI
# rig = "0.6"  # Optional, evaluate in Phase 2

# HTTP
reqwest = { version = "0.11", features = ["json"] }
axum = "0.7"

# CLI
clap = { version = "4.5", features = ["derive"] }

# Config
toml = "0.8"
```

## 2. Crate Responsibilities

### 2.1 Foundation Layer

| Crate | Responsibility | Dependencies |
|-------|---------------|--------------|
| `ares-types` | Core IDs (ProjectId, ArtifactId, etc.), common types, serialization | serde, uuid, chrono |
| `ares-errors` | Structured error types, error codes, retry classification | thiserror |
| `ares-config` | Configuration loading (env vars, TOML files, CLI args) | toml, serde |

### 2.2 Domain Layer

| Crate | Responsibility | Dependencies |
|-------|---------------|--------------|
| `ares-artifact` | Artifact management, content-addressed storage, file inspection, format detection | ares-types, ares-errors, goblin, object, scroll, sha2 |
| `ares-evidence` | Evidence repository, provenance chain, confidence scoring | ares-types, ares-errors, ares-storage-sqlite |
| `ares-analysis` | Analysis run tracking, function identity, cross-engine matching | ares-types, ares-evidence, petgraph |

### 2.3 Provider Layer

| Crate | Responsibility | Dependencies |
|-------|---------------|--------------|
| `ares-provider-api` | AnalysisProvider trait, ProviderDescriptor, AnalysisRequest/Result, ExecutionContext | ares-types, ares-errors, async-trait, tokio, serde |
| `ares-provider-registry` | Provider registration, health check, capability discovery, platform filtering | ares-provider-api, ares-errors, tokio |
| `ares-provider-mcp` | MCP protocol adapter (wraps McpClient as AnalysisProvider) | ares-provider-api, rea-mcp (or rmcp), tokio |

### 2.4 Storage Layer

| Crate | Responsibility | Dependencies |
|-------|---------------|--------------|
| `ares-storage-api` | Storage trait (read/write/query) | ares-types, ares-errors |
| `ares-storage-sqlite` | SQLite implementation, migrations, FTS5 | ares-storage-api, rusqlite, serde_json |

### 2.5 Application Layer

| Crate | Responsibility | Dependencies |
|-------|---------------|--------------|
| `ares-scheduler` | Task scheduling, cancellation, timeout, retry | ares-types, ares-errors, tokio |
| `ares-policy` | Authorization, permission levels, sandbox policy | ares-types, ares-errors |
| `ares-workflow` | Analysis plan execution, step orchestration | ares-scheduler, ares-provider-registry |

### 2.6 Interface Layer

| Crate | Responsibility | Dependencies |
|-------|---------------|--------------|
| `ares-mcp-gateway` | Multi-MCP server routing, capability filtering | ares-provider-registry, rmcp, tokio |
| `ares-cli` | CLI commands, argument parsing, output formatting | ares-workflow, ares-artifact, clap |

### 2.7 Apps

| Crate | Responsibility | Dependencies |
|-------|---------------|--------------|
| `ares` | Main binary, composition root | All crates |
| `ares-cli` | CLI binary | ares-cli |
| `ares-mcp` | MCP server binary | ares-mcp-gateway, ares-workflow |
| `ares-bench` | Benchmark runner | ares-workflow, ares-artifact |

### 2.8 Providers

| Crate | Responsibility | Dependencies |
|-------|---------------|--------------|
| `providers/rea` | REA Rust adapter (MCP client) | ares-provider-api, ares-provider-mcp |
| `providers/ghidra` | Ghidra adapter (MCP/Worker) | ares-provider-api, ares-provider-mcp |
| `providers/rizin` | Rizin adapter (Process/RPC) | ares-provider-api, ares-provider-process |
| `providers/radare2` | radare2 adapter (Process) | ares-provider-api, ares-provider-process |
| `providers/frida` | Frida adapter (Process/RPC) | ares-provider-api, ares-provider-process |
| `providers/jadx` | JADX adapter (Process) | ares-provider-api, ares-provider-process |
| `providers/angr` | angr adapter (Python/Process) | ares-provider-api, ares-provider-process |
| `providers/ilspy` | ILSpy adapter (Process) | ares-provider-api, ares-provider-process |
| `providers/binwalk` | Binwalk adapter (Process) | ares-provider-api, ares-provider-process |
| `providers/yara-x` | YARA-X adapter (Rust Native/FFI) | ares-provider-api, yara-x |

## 3. Dependency Direction

```
ares-types ← ares-errors ← ares-config
    ↓
ares-artifact ← ares-evidence ← ares-analysis
    ↓
ares-provider-api ← ares-provider-registry ← ares-provider-mcp
    ↓
ares-storage-api ← ares-storage-sqlite
    ↓
ares-scheduler ← ares-policy ← ares-workflow
    ↓
ares-mcp-gateway ← ares-cli
    ↓
apps (ares, ares-cli, ares-mcp, ares-bench)
    ↓
providers (rea, ghidra, rizin, ...)
```

Rules:
- Lower layers never depend on upper layers.
- Providers depend on provider-api, never on CLI or apps.
- Storage implementations depend on storage-api, never on providers.
- Apps depend on everything but nothing depends on apps.

## 4. Provider Trait and Lifecycle API

```rust
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::Duration;
use tokio_util::sync::CancellationToken;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderDescriptor {
    pub id: String,
    pub name: String,
    pub version: String,
    pub api_version: String,
    pub capabilities: Vec<String>,
    pub platforms: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisRequest {
    pub request_id: String,
    pub artifact_id: String,
    pub operation: String,
    pub parameters: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisEvidence {
    pub id: String,
    pub artifact_id: String,
    pub provider_id: String,
    pub category: String,
    pub location: Option<String>,
    pub raw_reference: Option<String>,
    pub content: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResult {
    pub request_id: String,
    pub provider_id: String,
    pub success: bool,
    pub data: Value,
    pub evidence: Vec<AnalysisEvidence>,
}

pub struct ExecutionContext {
    pub project_id: String,
    pub cancellation: CancellationToken,
    pub timeout: Duration,
}

#[async_trait]
pub trait AnalysisProvider: Send + Sync {
    fn descriptor(&self) -> ProviderDescriptor;
    async fn health_check(&self) -> Result<bool, ReError>;
    async fn analyze(&self, context: ExecutionContext, request: AnalysisRequest) -> Result<AnalysisResult, ReError>;
    async fn prepare(&self) -> Result<(), ReError> { Ok(()) }
    async fn close(&self) -> Result<(), ReError> { Ok(()) }
}
```

### Lifecycle

1. `prepare()` — Start process, connect, initialize
2. `health_check()` — Verify readiness
3. `analyze()` — Execute analysis
4. `close()` — Release resources, cleanup

Cancellation, timeout, and cleanup must propagate to external processes.

## 5. Evidence Schema

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evidence {
    pub id: String,
    pub project_id: String,
    pub artifact_id: String,
    pub provider_id: String,
    pub provider_version: String,
    pub analysis_run_id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub target_location: Option<String>,
    pub raw_result_hash: String,
    pub normalized_result: serde_json::Value,
    pub source_reference: Option<String>,
    pub confidence: Confidence,
    pub verification_status: VerificationStatus,
    pub limitations: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Confidence {
    ToolReported,
    RuleMatched,
    CrossEngine,
    AiAssessed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VerificationStatus {
    Unverified,
    Reviewed,
    Verified,
    Rejected,
}
```

## 6. Scheduler Behavior

- All long-running tasks managed by Scheduler
- Task types: Analysis, ProviderHealthCheck, Benchmark, Report
- Each task has: Input, Required Capability, Provider, Timeout, Resource Budget, Permission, Output Schema, Evidence, Retry Policy
- Cancellation support via CancellationToken
- Progress events via channel
- Resource limits: CPU, memory, time
- Retry with exponential backoff for transient failures
- Dead letter queue for failed tasks

## 7. MCP Tool Namespaces

| Namespace | Responsibility |
|-----------|---------------|
| `ares.system.*` | Health check, version, capabilities |
| `ares.project.*` | Project and workspace management |
| `ares.artifact.*` | Binary/file management |
| `ares.analysis.*` | Static analysis and decompilation |
| `ares.evidence.*` | Evidence and verification |
| `ares.ai.*` | Analysis plans, reasoning, reports |

Provider-specific tools discovered dynamically via capability negotiation.

## 8. Cross-Platform CI

### GitHub Actions Matrix

| OS | CPU | Rust | Providers |
|----|-----|------|-----------|
| ubuntu-latest | x64 | stable | rea, rizin, radare2, frida, jadx, binwalk, yara-x |
| ubuntu-latest | arm64 | stable | rea, rizin, radare2 |
| windows-latest | x64 | stable | rea, ghidra (if installed) |
| windows-latest | arm64 | stable | rea |
| macos-latest | x64 | stable | rea, rizin |
| macos-latest | arm64 | stable | rea, rizin |

### CI Steps
1. Rust fmt check
2. Clippy (-D warnings)
3. Build (all features)
4. Unit tests
5. Integration tests
6. Dependency audit (cargo-deny)
7. SBOM generation
8. Release artifact verification

## 9. Testing Strategy

### Unit Tests
Every crate: IDs, serialization, hashing, format detection, provider contracts, scheduling, cancellation, timeout, evidence provenance, transactions, migrations, policy boundaries, tool schemas.

### Golden Tests
Fixed samples: ELF, PE, Mach-O, WASM, Android, .NET, JavaScript, firmware, malformed. Each with SHA-256, expected format, expected parse results, license info.

### Provider Contract Tests
Every provider: startup, health check, capability declaration, architecture support, schema validity, invalid input handling, cancellation, timeout, resource cleanup, evidence preservation, version logging, platform rejection.

### AI Benchmarks
- AgentRE-Bench (ELF/PE, fixed tool budget, deterministic scoring)
- SRE-Bench (19 real programs, 262 instances, 1572 tasks)

### Cross-Platform Matrix
Per-provider platform support independently verified. Unsupported platforms show `UnsupportedPlatform`.

## 10. License Handling

- REA is MIT — preserve original LICENSE/NOTICE
- New ARES modules: own license policy (TBD)
- Create `THIRD_PARTY_NOTICES.md`
- Distinguish: original code, derived code, third-party dependencies
- Do not relicense inherited MIT code as Apache-2.0
- cargo-deny for license compliance checking

## 11. Release Milestones

| Version | Scope |
|---------|-------|
| 0.1.x | Rust Foundation (types, provider API, registry, artifact, evidence, SQLite, scheduler, policy, MCP gateway, CLI) |
| 0.2.x | Multi-Engine (REA, Ghidra, Rizin adapters, cross-engine matching) |
| 0.3.x | Multi-Domain (Frida, JADX, ILSpy, Binwalk, YARA-X) |
| 0.4.x | AI Intelligence (model router, agent orchestrator, knowledge graph, benchmark) |
| 0.5.x | Distributed Workers |
| 0.8.x | Desktop/Web |
| 0.9.x | Release Candidate |
| 1.0.0 | Stable |

## 12. GitHub Repository Strategy

### Repositories
- `ares` (main workspace)
- `ares-providers` (third-party providers)
- `ares-schemas` (data schemas)
- `ares-benchmarks` (test fixtures)
- `ares-docs` (documentation)
- `ares-deployment` (deployment scripts)

Initial: monorepo. Split when modules mature.

### Branch Strategy
```
main → develop → feature/* (per epic)
```

### Version Strategy
Semantic versioning with pre-release tags.
