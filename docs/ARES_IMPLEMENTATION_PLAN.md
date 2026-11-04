# ARES_IMPLEMENTATION_PLAN.md

## ARES Implementation Plan — GitHub Issues and Milestones

**Version**: 1.0-draft  
**Date**: 2026-11-01  
**Status**: Proposed — execution blocked pending owner approval
**Role**: Proposed execution plan only. Do not treat tasks as authorized or complete without approval and dated verification evidence.

---

## 1. Epic 0: REA Baseline (Highest Priority)

**Goal**: Fork and audit REA Rust as the baseline.

### Tasks
- [ ] Fork `nanashili/rea-rust` at pinned commit
- [ ] Preserve LICENSE/NOTICE/THIRD_PARTY_NOTICES.md
- [ ] Audit 209 tool contracts in `crates/interfaces/rea-mcp/contracts/catalog/mcp-tools.json`
- [ ] Confirm 200 default tool registrations
- [ ] Document Frida feature (9 additional contracts)
- [ ] Verify Ghidra Worker
- [ ] Record existing test results (1681 passed, 0 failed, 5 ignored)
- [ ] Establish upstream update strategy (pin commit, periodic sync)
- [ ] Create Migration Gap Report (what REA has vs what ARES needs)

### Deliverable
`ARES-BASELINE-001`: REA baseline audit document

### Acceptance Criteria
- REA builds successfully on Windows x64
- All REA tests pass
- Tool contract catalog documented
- Gap report identifies missing features

## 2. Epic 1: Rust Foundation

**Goal**: Core types, provider API, registry, config, errors, events, policy.

### Tasks
- [ ] `ares-types`: ProjectId, ArtifactId, ProviderId, AnalysisRunId, FunctionId, EvidenceId, FindingId, HypothesisId, ReportId
- [ ] `ares-errors`: Structured error types, error codes, retry classification
- [ ] `ares-config`: Environment variables, TOML config files, CLI args
- [ ] `ares-provider-api`: AnalysisProvider trait, ProviderDescriptor, AnalysisRequest, AnalysisResult, AnalysisEvidence, ExecutionContext
- [ ] `ares-provider-registry`: Registration, health check, capability discovery, platform filtering
- [ ] `ares-events`: Event bus for async notifications
- [ ] `ares-policy`: Permission levels L0-L5, authorization checks
- [ ] Unit tests for all crates
- [ ] Clippy clean (`-D warnings`)
- [ ] Fmt clean

### Deliverable
`ARES-CORE-001`: Rust foundation crates

### Acceptance Criteria
- All crates compile
- All unit tests pass
- Clippy zero warnings
- Provider trait compiles with async-trait

## 3. Epic 2: Multi-Engine Integration

**Goal**: Integrate REA, Ghidra, Rizin, radare2 as providers.

### Tasks
- [ ] REA adapter (MCP client wrapper)
- [ ] Ghidra adapter (MCP/Worker)
- [ ] Rizin adapter (Process/RPC)
- [ ] radare2 adapter (Process)
- [ ] Provider manifest (capabilities, platforms, versions)
- [ ] Capability discovery
- [ ] Health check for all providers
- [ ] Cross-engine tests
- [ ] Provider contract tests for each

### Deliverable
`ARES-PROVIDER-001`: Multi-engine provider integration

### Acceptance Criteria
- At least 2 independent engines operational
- Cross-engine function matching works
- Provider health checks pass
- Contract tests pass for each provider

## 4. Epic 3: Data & Evidence

**Goal**: Artifact management, evidence store, analysis runs, provenance.

### Tasks
- [ ] Artifact manager (content-addressed storage)
- [ ] SQLite storage (migrations, transactions)
- [ ] Evidence repository (CRUD, provenance chain)
- [ ] Analysis run tracking
- [ ] Function identity resolution
- [ ] Report schema
- [ ] Golden test fixtures (ELF, PE, Mach-O, WASM, Android, .NET, JS, firmware, malformed)

### Deliverable
`ARES-DATA-001`: Data and evidence management

### Acceptance Criteria
- Artifact registration with SHA-256
- Evidence provenance chain intact
- Analysis runs tracked with provider versions
- Golden test fixtures validated

## 5. Epic 4: MCP Gateway

**Goal**: RMCP server/client, multi-provider routing, capability filtering.

### Tasks
- [ ] RMCP server (official SDK)
- [ ] RMCP client
- [ ] STDIO transport
- [ ] Streamable HTTP transport
- [ ] Provider routing
- [ ] Capability filtering
- [ ] MCP authentication
- [ ] Long-running tasks + cancellation
- [ ] MCP conformance tests

### Deliverable
`ARES-MCP-001`: MCP gateway

### Acceptance Criteria
- MCP server responds to tool calls
- Provider routing works
- Capability filtering restricts tool access
- Conformance tests pass

## 6. Epic 5: Specialist Providers

**Goal**: Domain-specific providers for mobile, firmware, network, signatures.

### Tasks
- [ ] Frida (dynamic instrumentation)
- [ ] angr (symbolic execution)
- [ ] JADX (Android decompilation)
- [ ] ILSpy (.NET decompilation)
- [ ] Binwalk (firmware analysis)
- [ ] YARA-X (signature matching)
- [ ] Wireshark (network/protocol analysis)
- [ ] QEMU/Unicorn (emulation)
- [ ] Binary diff (BinDiff/Diaphora)

### Deliverable
`ARES-DOMAIN-001`: Specialist providers

### Acceptance Criteria
- At least 3 specialist providers operational
- Each passes contract tests
- Platform support documented

## 7. Epic 6: AI Intelligence

**Goal**: Model router, agent orchestrator, knowledge graph, benchmarks.

### Tasks
- [ ] Model router (multi-provider: OpenAI, Anthropic, local)
- [ ] Agent orchestrator (single planner agent)
- [ ] Planner (task planning from user request)
- [ ] Evidence reasoner (cross-engine comparison)
- [ ] Hypothesis store
- [ ] Knowledge graph (petgraph-based)
- [ ] Semantic search
- [ ] Report agent
- [ ] Benchmark runner (AgentRE-Bench)
- [ ] Record: model version, prompt version, provider version, tool calls, duration, tokens, evidence count, accuracy

### Deliverable
`ARES-AI-001`: AI intelligence layer

### Acceptance Criteria
- Planner generates analysis plans
- Evidence reasoner correlates cross-engine findings
- Benchmark produces measurable accuracy score
- Report generation works

## 8. Epic 7: Productization

**Goal**: Web UI, desktop app, distributed workers, sandbox, audit, release.

### Tasks
- [ ] Web UI (TypeScript, REST API consumer)
- [ ] Tauri desktop app
- [ ] Distributed workers
- [ ] PostgreSQL + object storage
- [ ] Sandbox (Windows Sandbox / process isolation)
- [ ] Audit logging
- [ ] Installer
- [ ] Documentation
- [ ] Release pipeline

### Deliverable
`ARES-1.0-RC1`: Release candidate

### Acceptance Criteria
- Web UI functional
- Desktop app builds
- Sandbox isolates dynamic analysis
- Audit trail complete

## 9. Epic 8: Release

**Goal**: Cross-platform CI, security audit, performance benchmarks, release.

### Tasks
- [ ] Cross-platform CI (GitHub Actions)
- [ ] Security audit
- [ ] Performance benchmarks
- [ ] Release artifacts (Windows, Linux, macOS)
- [ ] Documentation review
- [ ] Release notes

### Deliverable
`ARES-1.0`: Stable release

### Acceptance Criteria
- All CI checks pass
- Security audit clean
- Performance within targets
- Release artifacts published

## 10. Immediate Next Steps (Ordered)

See [ARES_MASTER_SPEC.md §14](ARES_MASTER_SPEC.md#14-immediate-next-steps). Track execution status and verification evidence in this plan.

## 11. Validation Plan

- All existing tests must continue passing
- Clippy clean (`-D warnings`)
- Fmt clean
- Build clean
- New provider contract tests pass
- Cross-engine matching test passes
- Benchmark produces measurable accuracy score
- No regression in CLI or MCP behavior

## 12. Risks and Mitigations

| Risk | Mitigation |
|------|-----------|
| REA API changes | Pin commit, adapter pattern isolates changes |
| Ghidra not installed | Graceful degradation, mock provider for tests |
| Benchmark too large | Start with AgentRE-Bench subset |
| AI hallucination | Evidence attribution + confidence levels |
| Platform incompatibility | Per-provider platform matrix, explicit UnsupportedPlatform |
| Single developer bottleneck | Milestone-based, not time-based delivery |
| Scope creep | Explicit "What NOT to Do" list |

## 13. GitHub Issue Template

Each issue must include:
- Title: `[EPIC-X] Task description`
- Labels: `epic`, `priority`, `area`
- Description: What, Why, Acceptance Criteria
- Dependencies: Blocked by / Blocks
- Estimate: Story points (1, 2, 3, 5, 8)
- Assignee: (unassigned initially)

## 14. Acceptance Criteria Summary

The implementation plan tracks execution status and evidence only. The canonical Phase 1 acceptance criteria are maintained in the [master specification, §16](ARES_MASTER_SPEC.md#16-acceptance-criteria-for-phase-1); do not duplicate or edit them here. No Phase 1 criterion is complete until its dated verification evidence is recorded against an approved revision.
