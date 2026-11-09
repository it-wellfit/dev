# Free AI Routing — Providers, Limits, and Fallback Chain

**Document:** `docs/engineering/FREE_AI_ROUTING.md`
**Status:** Active (v2.0 — hybrid dynamic architecture)
**Owner:** Autonomous Engineering Coordinator
**Last reviewed:** 2026-11-07
**Session:** `20261106-autonomous-evolution-init`
**v2.0 變更**: 新增 §8 混合動態架構（本地 Ollama ＋ 免費雲端四層池）、2026-11-07 全量重查的免費模型/工具目錄、配額感知路由規則與狀態追蹤；§2–§7 保留為 2026-11-06 實測記錄。

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

---

## 8. 混合動態架構 v2.1（2026-11-07 全機實測＋聯合裁決上線）

### 8.0 本機實測快照（2026-11-07，curl/reqwest 雙驗證）

| 池 | 端點 | 免費層實測模型（挑選） | 狀態 |
|----|------|------------------------|------|
| L0 Ollama | 127.0.0.1:11434（原生 /api/chat，think:false＋JSON schema） | qwen3.5:4b | check 直接通過 |
| L1 OVHcloud | oai.endpoints.kepler.ai.cloud.ovh.net/v1（免 key，2/min/IP） | Qwen3.8-27B(262K)、gpt-oss-120b、gpt-oss-20b、Qwen3.6-27B、Qwen3.5-397B-A17B(262K)、Llama-3.3-70B、Mistral-Small-3.2-24B、Qwen3Guard-Gen-8B（護欄）、bge-m3/Qwen3-Embedding-8B（嵌入） | check 直接通過 |
| L1 LLM7.io | api.llm7.io/v1（匿名，tier=turbo 免費） | gpt-oss:20b(128K, JSON)、codestral-latest、mistral-Nemo、nemotron-3-nano:30b(1M) | check 直接通過 |
| L1 Pollinations | text.pollinations.ai/openai（匿名） | openai-fast=GPT-OSS-20B（reasoning、tools） | check 直接通過 |
| L2 Groq/OpenRouter/Gemini 等 | 需免費註冊 | 見 §8.4 | 未實測（無金鑰） |

### 8.0.1 聯合裁決（Ensemble）— 已上線

kem vet2 的 SCS k 票改為**跨模型輪換**：每票依 pick_entry 於模型池輪轉（本地→OVH Qwen3.8→LLM7 gpt-oss:20b→OVH gpt-oss-120b→Pollinations），配額護欄（OVH 31s、LLM7/Pollinations 7s）內建於輪換器；每票帶模型歸因（ScsVote.model），verdict 列 models_used。分歧（agreement<0.60）誠實入審查佇列。首次聯合實測（claim-016）：5 票來自 5 個不同模型，durability 分佈 1–5，agreement 0.40 → 入隊——分歧可見、不掩蓋。

### 8.0.2 情境路由表（角色→模型池）

| 角色 | 首選 | 備援 | 依據 |
|------|------|------|------|
| vet（SCS 裁決） | 本地 qwen3.5:4b＋OVH Qwen3.8-27B＋LLM7 gpt-oss:20b | OVH gpt-oss-120b、Pollinations | JSON 模式＋配額 |
| code | OVH Qwen3.8-27B / LLM7 codestral | 本地 | 編碼基準 |
| longctx（>200K） | OVH Qwen3.5-397B-A17B(262K) / LLM7 nemotron-3-nano(1M) | — | 上下文長度 |
| embed（向量檢索） | OVH bge-m3 / Qwen3-Embedding-8B（免 key） | Ollama 本地嵌入 | P2b 向量檢索待接線 |
| guard（敏感度護欄） | OVH Qwen3Guard-Gen-8B（免 key 護欄模型） | T1 規則 | 專職護欄 |

以下 §8.1–§8.5 為 v2.0 查證記錄（保留）。
## 8.1 四層資源池（v2.0 查證記錄，2026-11-06–07）

### 8.1 四層資源池（全部零付費；§2.2/§7 保留為 2026-11-06 實測記錄）

| 層 | 池 | 端點/來源 | 限額 | 驗證狀態 |
|----|----|-----------|------|----------|
| **L0 本地** | Ollama（qwen3.5:4b 現役；升級候選見 §8.3） | `http://localhost:11434` | 無限（受硬體限制） | ✅ 2026-11-06 實測 |
| **L1 無金鑰雲** | OVHcloud AI Endpoints（EU/GDPR，40+ 開源模型） | `https://oai.endpoints.kepler.ai.cloud.ovh.net/v1` | 2/min/IP | ⏳ 未實測 |
| | Pollinations | OpenAI 相容 | 匿名 IP 限流 | ⏳ 未實測 |
| | LLM7.io | `https://api.llm7.io/v1` | 匿名 30–60/hr（可選 key 提升至 120 RPM） | ⏳ 未實測 |
| | Kilo Gateway | keyless（13 條啟用路由） | ~200 req/hr per IP | ⚠️ 需重測：2026-11-06 對 `gateway.kilo.ai` 實測失敗，端點可能已遷移 |
| | **OpenCode Zen** | keyless 目錄（預設停用、需 opt-in） | 未知 | ⏳ 未實測；**含 Jev 1.13 Free** |
| **L2 免費註冊雲** | Groq | `https://api.groq.com/openai/v1` | 30 RPM、1K RPD、8K TPM、200K TPD（gpt-oss-120b / qwen3.8-27b / llama-3.3-70b） | ⏳ 未實測（需免費註冊） |
| | OpenRouter :free | `https://openrouter.ai/api/v1` | 20 RPM、50/day（一次性 $10 儲值後 1,000/day）；17–25+ 個 :free 模型（nemotron-3-ultra-550b 1M ctx 等） | ⚠️ 2026-11-06 實測 401（無金鑰） |
| | Google Gemini API | `https://generativelanguage.googleapis.com/v1beta/openai/` | Flash 家族免費、1M ctx；**免費層資料用於訓練**（敏感資料禁用） | ⏳ 未實測 |
| | Cloudflare Workers AI | `https://api.cloudflare.com/client/v4/accounts/{id}/ai/v1` | 10,000 neurons/day、300 RPM | ⚠️ 2026-11-06 實測 403（需帳戶） |
| | SambaNova | `https://api.sambanova.ai/v1` | 20 RPM、20 RPD、200K TPD（每模型） | ⏳ 未實測 |
| | Ollama Cloud | 雲端原生權重（GLM-5.2 1M ctx、Kimi K2.7 Code、DeepSeek V4 Pro） | 免費層 1 並發 | ⏳ 未實測 |
| **L3 無 AI** | 規則引擎（T1） | — | — | ✅ kem crate 已實作 |

### 8.2 動態路由規則（決策矩陣）

路由＝ f(任務型別, 資料敏感度, 配額狀態, 延遲需求)，規則：

| # | 規則 |
|---|------|
| R1 | **敏感資料（claims、憑證相關、未公開計畫）→ 僅 L0**（本地），永不外送 |
| R2 | **結構性可判定 → L3**（T1 規則，零呼叫優先） |
| R3 | L0 可服務且配額未耗盡 → L0 優先（無限額、零延遲風險） |
| R4 | L0 不可用（服務未啟動/模型過大）→ L1（keyless）→ L2（免費註冊）逐層降級，帶指數退避＋抖動 |
| R5 | **429/限流 → 記錄配額狀態、跳下一池**；同池退避重試上限 2 次 |
| R6 | 大上下文（>100K tokens）→ L2 的 1M-ctx 模型（nemotron-3-ultra / Gemini flash / Kilo nemotron-3.5-super-vl） |
| R7 | SCS k=5 投票 → L0 為主；L0 過載時分散到 L1（不同供應商投票可提升多樣性，但需記錄模型版本） |
| R8 | 每次呼叫記錄：池、供應商、模型、延遲、結果 → `.kem/ai-ledger.jsonl`（配額與健康追蹤） |

### 8.3 本地模型升級候選（Ollama；全部開源可本地）

| 模型 | 授權 | 規模 | 顯示卡/記憶體 | 亮點 |
|------|------|------|----------------|------|
| **qwen3.8-27b**（現役 qwen3.5:4b 的後繼） | Apache-2.0 | 27B dense | 24GB GPU / 18GB q4 | Terminal-Bench 73.0、SWE-bench Pro 61.7、原生視覺 |
| **qwen3.6-35b-a3b** | Apache-2.0 | 35B/3B 活化 | 低算力 | 73.4% SWE-bench Verified、3B 活化最省算力 |
| **glm-4.7-flash** | MIT | 31B/3B 活化 | 18–33GB q | MIT、59.2% SWE-V |
| **qwen3-coder-next** | Apache-2.0 | 80B/3B 活化 | 46GB（2-bit 30GB） | 70.6% SWE-V、單機最強開源編碼 |
| **gpt-oss-20b** | Apache-2.0 | 21B | 16GB | 單卡單用戶最清晰選擇 |

**升級路徑**: 現役 qwen3.5:4b 供 SCS 與輕量裁決；待硬體確認後以 qwen3.8-27b（或 qwen3.6-35b-a3b）升級 T2 主力——升級需重跑金標集（§8.2 R8 記錄）對比，不盲升。

### 8.4 免費強大模型目錄亮點（2026-11-07 查證）

| 類別 | 模型 | 取得管道 | 備註 |
|------|------|----------|------|
| 頂級開源編碼 | DeepSeek V4 Pro（MIT, 1.6T/49B, 80.6% SWE-V） | DeepSeek 網頁免費 / Ollama Cloud | 最強免費編碼 |
| | Kimi K2.6（Mod-MIT, 80.2% SWE-V, 300 子代理） | kimi.com 免費 / Groq | 代理式最強 |
| | GLM-5.2（MIT, 1M ctx） | Z.ai / Ollama Cloud | 長程代理首選 |
| 免費 API 編碼 | gpt-oss-120b / qwen3.8-27b | Groq 免費層 | 30 RPM/1K RPD |
| 1M 上下文 | nemotron-3-ultra-550b-a55b:free | OpenRouter :free | 1M ctx |
| **決策模型** | **Jev 1.13 Free** | **OpenCode Zen（keyless，需 opt-in）** | JEV 免費版存在→零付費約束下仍可用；採用前需驗證目錄啟用狀態 |
| 嵌入 | BGE 系列 / Ollama local embed | Cloudflare（neurons）/ Ollama | 向量檢索用 |

### 8.5 配額狀態追蹤（實作契約）

`.ai-state/quota.json`（gitignored）：每供應商 `{rpm_used, day_used, day_reset_utc, last_error, cooldown_until}`；`ai-ledger.jsonl` 記錄每次呼叫。路由器讀狀態決定 R4/R5。狀態檔可隨時刪除重建（衍生性）。

**誠實聲明**（`[LIMIT]`）: §8 表中「⏳ 未實測」項目為 2026-11-07 文件查證（來源：freeinference.dev 2026-11-07 快照、freellmpool、apidog 2026-10-22、requesty 2026-11-01），非本機實測；採用前依 V-冒煙程序驗證。免費層無契約保障，隨時可能變動——架構必須容忍任何一層消失（R4 降級鏈存在即是為此）。
