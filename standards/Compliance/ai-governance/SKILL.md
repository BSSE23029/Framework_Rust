---
name: ai-governance
description: Governance and security for the product's AI usage — OpenAI and LM Studio integrations, AI workflows in CI, and any AI-driven decisioning — under ISO/IEC 42001, the EU AI Act, SDAIA's Saudi AI guidance, the OWASP LLM and Agentic top tens, and the CCPA ADMT rules. Use whenever an AI feature or AI tooling is in scope.
---

# AI governance

Registry: `ISO-42001`, `EU-AI-ACT`, `SA-SDAIA-AI`, `OWASP-LLM-TOP10`, `US-CCPA` (ADMT).

This product already uses AI in two places, and they carry different risk:

1. **Product AI** — `Services/Integrations/AI/OpenAi/queryOpenAi.js` and
   `Services/Integrations/AI/LMStudio/` (local and network).
2. **Pipeline AI** — `ai-agent.yml`, `ai-handler.yml`, `ai-review.yml` in
   `backend/.github/workflows/`, which are agentic systems with repository access.

## The privacy question comes first

Before any AI-governance framework applies, answer the data question: **what personal data goes
into the prompt, and where is it processed?** An OpenAI call from a Saudi deployment carrying guest
data is a cross-border transfer to the US and needs a route under `SA-PDPL-TRANSFER` — with a
transfer risk assessment if it is continuous or involves sensitive data. LM Studio running locally
is materially different, and that difference is worth designing around.

Then: is the data used for training, is there a DPA and a zero-retention arrangement, and is the
prompt minimised to what the feature needs.

## EU AI Act — `EU-AI-ACT`

| Obligation | Status |
|---|---|
| Prohibited practices | In force since 2 Feb 2025 |
| GPAI obligations | In force since 2 Aug 2025 |
| **Transparency duties (Art. 50)** | **In force since 2 Aug 2026** |
| Annex III high-risk | Deferred to **2 Dec 2027** by the omnibus |
| Annex I embedded high-risk | Deferred to **2 Aug 2028** |

Article 50 is the live one, and it is the one a hotel product trips: if a guest interacts with an AI
system, they must be told; AI-generated content needs marking. Deferral of the high-risk deadlines
does not touch it.

Applies where the system is placed on the EU market or its output is used in the EU. Assess per
deployment, like everything else.

## Saudi guidance — `SA-SDAIA-AI`

SDAIA's AI Ethics Principles (2023), Generative AI Guidelines (Jan 2024) and the AI Adoption
Framework. Largely non-binding for private-sector SaaS, but it is the yardstick Saudi enterprise and
government-adjacent buyers apply, and Saudi declared 2026 its "Year of AI" with further instruments
landing. *Registry note: sources disagree on whether the Adoption Framework is Sept 2024 or Nov 2025
— verify before citing a date.*

## ISO/IEC 42001 — `ISO-42001`

The certifiable AI management system standard (published Dec 2023; adopted in Europe as
EN ISO/IEC 42001:2026, CEN approval 13 Mar 2026). Same management-system shape as ISO 27001: scope,
policy, roles, AI risk assessment, impact assessment, lifecycle controls, monitoring, improvement.
Worth pursuing only once there is real AI functionality in the product — but the **AI system
inventory** it demands is worth building now, because nothing else forces you to write down every
place a model is called.

## OWASP LLM and Agentic top tens — `OWASP-LLM-TOP10`

The security layer. For the product AI: prompt injection, sensitive information disclosure, supply
chain, insecure output handling, excessive agency, system prompt leakage, unbounded consumption.

For the **CI AI workflows** specifically, use the Agentic list (2026). The questions that matter:

- Can untrusted text — an issue title, a PR body, a commit message from outside — reach the prompt?
- What is the token scope, and can the agent push, merge or deploy?
- Is model output ever executed or trusted without validation?
- Is there a spend and rate ceiling?
- Is there an audit trail of what the agent did?

An agent with write access and an injectable prompt is a supply-chain compromise waiting to be
written up. Grade these in the **CI/CD** grid, not the backend one.

## Automated decisioning

If AI ever drives pricing, guest screening, credit, refunds or anything decision-like about a
person, two more regimes engage: **CCPA ADMT** (notice, access, opt-out; compliance by 1 Jan 2027
for pre-existing uses) and the AI Act's high-risk analysis. Recommendation engines and dynamic
pricing sit closer to this line than teams expect — assess explicitly rather than assuming not.

## Disclosure

Anywhere a guest interacts with AI, disclose it. Required by `EU-AI-ACT` Art. 50 for EU-facing
deployments, expected by `SA-SDAIA-AI`, and cheap. Frontend surface.
