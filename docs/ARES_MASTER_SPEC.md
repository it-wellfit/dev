# ARES_MASTER_SPEC.md

## Autonomous Reverse Engineering System — Master Specification

**Version**: 1.0-draft  
**Date**: 2026-11-01  
**Status**: Proposed — not approved
**License**: MIT (inherited from REA Rust) + ARES additions TBD
**Role**: Proposed requirements and architecture; not implementation evidence. Phase 1 acceptance criteria in §16 are the criteria source for the implementation plan.

---

## 1. Project Vision

ARES is an independent Rust-first reverse engineering platform. External tools (REA, Ghidra, Rizin, Frida, etc.) are professional engines ARES invokes — not the platform itself. ARES owns the architecture, data model, provider ecosystem, AI system, and long-term roadmap.

**Core Principle**: Fork and evolve `nanashili/rea-rust` as the initial baseline without letting it dictate architecture.

## 2. Core Principles

1. **Rust-First, Not Rust-Only**: Autonomous core in Rust. Mature engines (C/C++/Java/Python) integrated as providers.
2. **Multi-Engine, Not Single-Engine**: Cross-validate results from independent engines with different IRs and analysis strategies.
3. **MCP Is Not the Only Integration**: Support MCP, CLI, RPC, REST/WebSocket, SDK, FFI, WASM, File Exchange.
4. **Evidence-First AI**: Every AI conclusion must carry source evidence, confidence level, and limitations.
5. **Fork REA, Don't Rebuild**: Start from `nanashili/rea-rust` as verifiable baseline; extend, don't replace.

## 3. System Architecture

### 3.1 Three-Layer Architecture

```
Layer 3: AI Agent / MCP Coordination
  - Multi-agent orchestrator
  - MCP Gateway (RMCP SDK)
  - Provider routing
  - Evidence reasoning

Layer 2: Native Rust Analysis
  - Binary format parsing (goblin, object, scroll)
  - Disassembly (iced-x86)
  - Symbol demangling (rustc-demangle)
  - Call graph (petgraph)
  - String extraction
  - WASM parsing (wasmparser)

Layer 1: Evidence & Verification
  - SQLite/PostgreSQL storage
  - SHA-256 content addressing
  - Isolation policy enforcement
  - Report generation
  - Audit trail
```

### 3.2 Dependency Direction

```
foundation (types, errors, config)
    ↓
domain (artifact, evidence, analysis)
    ↓
provider-api (AnalysisProvider trait)
    ↓
providers (ghidra, rizin, rea, frida, ...)
    ↓
application (workflows, scheduling)
    ↓
interfaces (CLI, MCP, REST, RPC)
    ↓
apps (ares-cli, ares-mcp, ares-server)
```

Rules:
- Domain never depends on any external Provider.
- Provider never depends on CLI.
- AI Agent never directly operates host machine.
- MCP is protocol adapter only.
- Web/Desktop cannot implement analysis logic independently.
- Evidence must be managed by unified Core.
- All long-running tasks managed by Scheduler.
- All external programs launched by controlled Process Manager.

## 4. Provider Ecosystem

### 4.1 Provider Positioning

| Tool | ARES Role | Priority | Integration Method |
|------|-----------|----------|-------------------|
| REA Rust | Optional engine provider + baseline | P0 | Rust Native / MCP |
| Ghidra | Primary open-source decompiler | P0 | Worker / MCP |
| Rizin | Secondary analysis engine | P0 | Process / RPC |
| radare2 | CLI analysis engine | P1 | Process |
| IDA Pro | Optional commercial engine | P2 | SDK |
| Binary Ninja | Optional commercial engine | P2 | SDK |
| Frida | Dynamic instrumentation | P1 | Process / RPC |
| angr | Symbolic execution | P1 | Process / Python |
| JADX | Android decompilation | P1 | Process / MCP |
| ILSpy | .NET decompilation | P2 | Process |
| Binwalk | Firmware analysis | P2 | Process |
| Wireshark | Network/protocol analysis | P2 | Process |
| YARA-X | Signature matching | P2 | Rust Native / FFI |
| OXC | JavaScript/TypeScript analysis | P2 | Rust Native |

### 4.2 Provider API (v1)

```rust
#[async_trait]
pub trait AnalysisProvider: Send + Sync {
    fn descriptor(&self) -> ProviderDescriptor;
    async fn health_check(&self) -> Result<bool, ReError>;
    async fn analyze(&self, context: ExecutionContext, request: AnalysisRequest) -> Result<AnalysisResult, ReError>;
    async fn prepare(&self) -> Result<(), ReError> { Ok(()) }
    async fn close(&self) -> Result<(), ReError> { Ok(()) }
}
```

Future versions must add: Provider Session, Progress Events, Capability Schema, Resource Budget, Structured Error, Output Artifact, Retry Policy, Version Negotiation.

### 4.3 Provider Lifecycle

1. `prepare()` — Start process, connect, initialize
2. `health_check()` — Verify readiness
3. `analyze()` — Execute analysis
4. `close()` — Release resources, cleanup

Cancellation, timeout, and cleanup must propagate to external processes, not just Rust-side waiting.

## 5. Core Data Model

### 5.1 Identity System

| ID | Purpose |
|----|---------|
| ProjectId | Project |
| ArtifactId | Sample/binary |
| ProviderId | Analysis engine |
| AnalysisRunId | Single analysis execution |
| FunctionId | Unified function entity |
| EvidenceId | Raw evidence |
| FindingId | Analysis finding |
| HypothesisId | AI hypothesis |
| ReportId | Report |

All IDs globally unique and traceable.

### 5.2 Evidence Schema

Every evidence record carries:
- evidence_id, project_id, artifact_id, provider_id, provider_version
- analysis_run_id, timestamp, target_location
- raw_result_hash, normalized_result, source_reference
- confidence (tool-reported / rule-matched / cross-engine / AI-assessed)
- verification_status, limitations

Confidence must distinguish:
- Tool-reported confidence
- Rule match certainty
- Cross-engine validation degree
- AI subjective assessment

These must NOT be mixed into a single number.

### 5.3 Function Identity

Cross-engine function matching by:
- Address (primary key)
- Normalized name (strip engine prefixes: FUN_, fcn., sub_, etc.)
- Size
- Content hash (optional)

## 6. MCP Tool Namespaces

| Namespace | Responsibility |
|-----------|---------------|
| `ares.system.*` | Health check, version, capabilities |
| `ares.project.*` | Project and workspace management |
| `ares.artifact.*` | Binary/file management |
| `ares.analysis.*` | Static analysis and decompilation |
| `ares.evidence.*` | Evidence and verification |
| `ares.ai.*` | Analysis plans, reasoning, reports |

Provider-specific tools discovered dynamically via capability negotiation.

## 7. Security Architecture

### 7.1 Permission Levels

| Level | Permission | Default |
|-------|-----------|---------|
| L0 | View metadata | Allowed |
| L1 | Read analysis results | Allowed |
| L2 | Static analysis | Allowed |
| L3 | Controlled dynamic observation | Requires authorization |
| L4 | Execute target program | Isolated env + explicit auth |
| L5 | Modify target / side effects | Denied by default |

AI agents default to L0-L2 only.

### 7.2 Untrusted File Handling

- Low-privilege parser workers
- Max file size, CPU, memory, time limits
- Path traversal prevention
- Zip bomb protection
- Decompression depth limits
- Content-addressed storage
- No network access for parsers

### 7.3 External Tool Execution

- Fixed executable paths
- Separate argv (no shell concatenation)
- Output limits
- Process groups / Windows Job Objects
- Cancellation support
- Timeouts
- Resource cleanup
- Version and parameter logging

## 8. AI Agent System

### 8.1 Agent Roles

- Orchestrator (global task control)
- Planner (task planning)
- Native Agent (binary analysis)
- Mobile Agent (Android/iOS)
- Firmware Agent (firmware/IoT)
- Runtime Agent (dynamic analysis)
- Diff Agent (version comparison)
- Evidence Agent (evidence audit)
- Report Agent (report generation)

### 8.2 Standard Analysis Workflow

```
User Request
  → Project/Artifact Resolution
  → Authorization & Policy Check
  → Analysis Planner
  → Capability Discovery
  → Provider Selection (hard filters → soft ranking)
  → Analysis Scheduler
  → [Ghidra | Rizin | REA] → Evidence Collection
  → Cross-Engine Comparison
  → Hypothesis Review
  → Verification
  → Final Report
```

### 8.3 Evidence Reasoning

Every finding must carry:
- Observed facts
- Tool inferences
- AI hypotheses
- Unverified conclusions

These must be clearly separated, not conflated.

## 9. Database Strategy

| Technology | Purpose | Priority |
|------------|---------|----------|
| SQLite | Personal/local | P0 |
| PostgreSQL | Multi-user/server | P1 |
| MinIO/S3 | Raw file storage | P1 |
| FTS5 | Local full-text search | P0 |
| pgvector | Semantic search | P2 |
| Petgraph | Runtime graph analysis | P0 |

Phase 1: SQLite + content-addressed storage. Phase 2: PostgreSQL + object storage.

## 10. Testing Strategy

### 10.1 Unit Tests
Every crate: IDs, serialization, hashing, format detection, provider contracts, scheduling, cancellation, timeout, evidence provenance, transactions, migrations, policy boundaries, tool schemas.

### 10.2 Golden Tests
Fixed samples: ELF, PE, Mach-O, WASM, Android, .NET, JavaScript, firmware, malformed. Each with SHA-256, expected format, expected parse results, license info.

### 10.3 Provider Contract Tests
Every provider must pass: startup, health check, capability declaration, architecture support, schema validity, invalid input handling, cancellation, timeout, resource cleanup, evidence preservation, version logging, platform rejection.

### 10.4 AI Benchmarks
- AgentRE-Bench (ELF/PE, fixed tool budget, deterministic scoring)
- SRE-Bench (19 real programs, 262 instances, 1572 tasks)
- Record: model version, prompt version, provider version, tool calls, duration, tokens, evidence count, accuracy, unsupported claim rate, reproducibility

### 10.5 Cross-Platform Matrix
Windows (x64/ARM64), Linux (x64/ARM64), macOS (Intel/Apple Silicon). Each provider tested independently. Unsupported platforms must show `UnsupportedPlatform`, not fail the entire core.

## 11. GitHub Repository Strategy

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
```
0.1.x  Rust Foundation
0.2.x  Multi-Engine
0.3.x  Multi-Domain
0.4.x  AI Intelligence
0.5.x  Distributed Workers
0.8.x  Desktop/Web
0.9.x  Release Candidate
1.0.0  Stable
```

## 12. License Handling

- REA is MIT — preserve original LICENSE/NOTICE
- New ARES modules: own license policy (TBD)
- Create `THIRD_PARTY_NOTICES.md`
- Distinguish: original code, derived code, third-party dependencies
- Do not relicense inherited MIT code as Apache-2.0

## 13. Platform Compatibility Matrix

| OS | CPU | Core | CLI/MCP | Provider |
|----|-----|------|---------|----------|
| Windows | x64 | Target | Target | Per-provider |
| Windows | ARM64 | Target | Target | Per-provider |
| Linux | x64 | Target | Target | Per-provider |
| Linux | ARM64 | Target | Target | Per-provider |
| macOS | Intel | Target | Target | Per-provider |
| macOS | Apple Silicon | Target | Target | Per-provider |

Core and CLI/MCP are cross-platform targets. Each provider's platform support is independently verified.

## 14. Immediate Next Steps

1. Fork `nanashili/rea-rust` at pinned commit
2. Create `ares-types`, `ares-provider-api`, `ares-provider-registry` crates
3. Implement Provider trait with `prepare()`/`close()`/progress/cancellation
4. Create Ghidra and Rizin adapters
5. Implement MCP Gateway with RMCP SDK
6. Set up CI with cross-platform matrix
7. Create golden test fixtures
8. Establish benchmark runner

## 15. Deliverables

Three formal engineering documents:
1. `ARES_MASTER_SPEC.md` — this document
2. `ARES_CARGO_WORKSPACE.md` — crate responsibilities, dependencies, Provider API, data schemas
3. `ARES_IMPLEMENTATION_PLAN.md` — GitHub Issues/Milestones, first-phase code, tests

## 16. Acceptance Criteria for Phase 1

- REA original functionality preserved
- Rust workspace reproducible build
- Provider API defined and tested
- Provider Registry functional
- Artifact Manager operational
- Evidence Store with provenance
- SQLite storage
- Ghidra Provider (if available)
- Rizin Provider (if available)
- MCP Gateway (RMCP)
- CLI
- Authorization and Process Lifecycle
- Cross-Platform CI
- Provider Contract Tests

**First version success standard**: The same authorized binary sample can be imported, identified, dispatched to different analysis engines, collect results, preserve source evidence, and obtain consistent analysis records through CLI and MCP.

## 17. What NOT to Do in Phase 1

- Do NOT introduce PostgreSQL, MinIO, Firecracker, Kubernetes
- Do NOT build Web UI or Tauri Desktop
- Do NOT implement 8-agent orchestration
- Do NOT create 80+ crates — start with existing structure
- Do NOT rebuild Ghidra or any mature engine
- Do NOT pursue false all-platform claims
- Do NOT let MCP become the database — Core manages data
- Do NOT relicense inherited MIT code
