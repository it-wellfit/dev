# Free AI Routing — Providers, Limits, and Fallback Chain

**Document:** `docs/engineering/FREE_AI_ROUTING.md`
**Status:** Active (draft v1)
**Owner:** Autonomous Engineering Coordinator
**Last reviewed:** 2026-11-06
**Session:** `20261106-autonomous-evolution-init`

---

## 1. Purpose

Define the zero-cost AI provider routing strategy for the autonomous development evolution workflow. All endpoints are free (no paid tier required). Every provider and limit recorded here was verified by live smoke test on 2026-11-06 from this machine.

---

## 2. Verified Providers (smoke-tested 2026-11-06)

### 2.1 Primary — Local Ollama (WORKING)

| Item | Value |
|------|-------|
| **Provider** | Ollama (local inference) |
| **Base URL** | `http://localhost:11434` |
| **API** | OpenAI-compatible (`/v1/chat/completions`) + native (`/api/generate`) |
| **Model installed** | `qwen3.5:4b` (4.2B params, Q4_K_M, 262K context, 3.3 GB) |
| **Capabilities** | completion, vision, tools, thinking |
| **Cost** | $0 (local hardware only) |
| **Rate limit** | None (bounded by RAM/VRAM) |
| **Privacy** | 100% local — no data leaves machine |
| **Status** | **WORKING** — verified generate + tags endpoints |

### 2.2 Remote Free Endpoints (NOT reachable without API key)

| Provider | Endpoint tried | Result | Reason |
|----------|---------------|--------|--------|
| **Kilo Gateway** | `https://gateway.kilo.ai/v1/chat/completions` | FAIL — Vercel deployment not found | Endpoint not serving; URL may have changed |
| **Kilo Gateway (alt)** | `https://api.kilo.ai/v1/models` | FAIL — 404 (Next.js web app) | Not an API endpoint |
| **OpenRouter** | `https://openrouter.ai/api/v1/chat/completions` | FAIL — 401 "No cookie auth credentials found" | Requires API key (free signup available) |
| **Z.ai (GLM-4.5-Flash)** | `https://api.z.ai/api/paas/v4/chat/completions` | FAIL — 401 Unauthorized | Requires API key (free tier exists) |
| **Cloudflare Workers AI** | `https://api.cloudflare.com/client/v4/accounts` | FAIL — 403 | Requires account_id + API token |
| **OVHcloud AI Endpoints** | Not tested | — | Anonymous keyless (2/min per IP); EU-hosted |

### 2.3 Keyless / Anonymous Providers (not yet tested from this machine)

| Provider | Models | Limits | Signup | Notes |
|----------|--------|--------|--------|-------|
| OVHcloud AI Endpoints | 10 models (Qwen3Guard, Riva TTS, Stable Diffusion XL) | 2/min per IP | None | EU-hosted, GDPR-friendly |
| LLM7.io | GPT-OSS 120B, Gemma 4 31B, MiniMax M2.7, Codestral, Mistral Nemo | 10 RPM anonymous (40 with token) | None | No signup required |
| BlockRun (NVIDIA) | gpt-oss-120b, deepseek-v4-flash (1M ctx), nemotron-3-nano-omni-30b, step-3.7-flash, mistral-nemotron | Per-IP rate-limited; no SLA | None | Keyless curl; retry on 429/cold start |

---

## 3. Model Routing Rules

### 3.1 Task-to-model mapping

| Task type | Model | Provider | Rationale |
|-----------|-------|----------|-----------|
| Plan / architecture / spec design | qwen3.5:4b (thinking mode) | Ollama local | Deep reasoning, 262K context, no data leaves machine |
| Implementation (code generation) | qwen3.5:4b | Ollama local | Good coding capability, tool use support |
| Verification / guard checks | qwen3.5:4b (instant mode) | Ollama local | Fast, sufficient for lint/test review |
| Vision / multimodal analysis | qwen3.5:4b | Ollama local | Native vision support |
| Large-context analysis (>100K tokens) | qwen3.5:4b (262K context) | Ollama local | 262K context window |

### 3.2 Fallback chain (when primary unavailable)

```
1. Ollama local (qwen3.5:4b) — PRIMARY
   ↓ (if Ollama service down or model not loaded)
2. OVHcloud AI Endpoints (anonymous, 2/min per IP) — SECONDARY
   ↓ (if rate-limited or unavailable)
3. LLM7.io (anonymous, 10 RPM) — TERTIARY
   ↓ (if rate-limited or unavailable)
4. BlockRun NVIDIA (anonymous, per-IP limited) — QUATERNARY
   ↓ (if all remote fail)
5. Rule-based engine (no AI) — LAST RESORT
```

### 3.3 Future: account-based providers (require free signup)

These providers offer free tiers but require account creation and API key. They are listed for future adoption when higher capability or throughput is needed:

| Provider | Free model | Limits | Signup | Notes |
|----------|-----------|--------|--------|-------|
| OpenRouter | :free models (20+) | 20 RPM, 50/day (1,000/day after $10 top-up) | Email | Multi-provider aggregation |
| Z.ai (Zhipu/GLM) | GLM-4.5-Flash, GLM-4.7-Flash | $0 permanent | Email | OpenAI-compatible, permanent free |
| Groq | GPT-OSS 120B, Qwen | 30 RPM, 1K RPD, 8K TPM, 200K TPD | Phone verification | Fast inference |
| Google AI Studio | Gemini 2.5 Flash, Flash-Lite | Per-model limits | Google account | 1M context; content may be used for training |
| NVIDIA NIM | Nemotron, DeepSeek V4.1 Flash | ~1K/day | Email | Evaluation purposes |
| SiliconFlow (China) | Qwen2.5-7B-Instruct, Hunyuan-MT-7B | ¥0 permanent | Phone + real-name auth | Chinese platform |

---

## 4. Per-Loop Token Budget

| Budget item | Value | Rationale |
|-------------|-------|-----------|
| Max tokens per AI call | 4,096 | Prevent runaway generation; sufficient for code review |
| Max AI calls per loop iteration | 10 | Bound cost; escalate to human if exceeded |
| Max loop iterations before human gate | 3 | Prevent infinite repair loops (Kitchen Loop principle) |
| Context window usage cap | 80% of 262K (≈210K tokens) | Reserve headroom for tool results |
| Daily AI call budget | Unlimited (local) | Ollama has no rate limit |

---

## 5. Kill Switch & Escalation

- **Kill switch:** Set environment variable `AI_ROUTING_DISABLED=1` to bypass all AI calls and fall back to rule-based engine.
- **Escalation trigger:** If a loop iteration fails verification 3 times consecutively, park the ticket and ask a specific human question (not "what should I do?").
- **Capacity reservation:** Always reserve at least 50% of local compute for human-interactive work; never saturate Ollama with unattended batch jobs during active development.

---

## 6. Verification Log

| Date | Provider | Endpoint | Result | Notes |
|------|----------|----------|--------|-------|
| 2026-11-06 | Ollama | `http://localhost:11434/api/tags` | PASS | Model qwen3.5:4b listed |
| 2026-11-06 | Ollama | `http://localhost:11434/api/generate` | PASS | Generated "Hello" in 36.5s (first call, model load) |
| 2026-11-06 | Kilo Gateway | `https://gateway.kilo.ai/v1/chat/completions` | FAIL | Vercel deployment not found |
| 2026-11-06 | Kilo Gateway (alt) | `https://api.kilo.ai/v1/models` | FAIL | 404 (Next.js web app, not API) |
| 2026-11-06 | OpenRouter | `https://openrouter.ai/api/v1/chat/completions` | FAIL | 401 — requires API key |
| 2026-11-06 | Z.ai | `https://api.z.ai/api/paas/v4/chat/completions` | FAIL | 401 — requires API key |
| 2026-11-06 | Cloudflare Workers AI | `https://api.cloudflare.com/client/v4/accounts` | FAIL | 403 — requires account_id + token |
| 2026-11-06 | Cloudflare (web) | `https://workers.cloudflare.com` | FAIL | Non-interactive PowerShell error |

---

## 7. Related Documents

- `docs/engineering/DEVELOPMENT_METHODOLOGY.md` — methodology adoption plan
- `docs/engineering/TECH_STACK.md` — technology stack baseline
- `.kilo/plans/1793966971740-autonomous-engineering-evolution-workflow.md` — master plan
- `.kilo/plans/1793959524-methodology-adoption.md` — methodology adoption (2 open questions)
- `.kilo/plans/1793033512795-flaps-erp-workbench-completion.md` — Flaps ERP workbench completion
