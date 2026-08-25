---
name: nist-csf
description: NIST Cybersecurity Framework 2.0 as the risk-management spine and cross-walk hub, plus SP 800-53 as a control catalogue. Use for structuring a security programme, prioritising remediation, or mapping between ISO, SOC 2, NCA ECC and other frameworks.
---

# NIST CSF 2.0

Registry: `NIST-CSF`, `NIST-80053`.

Released February 2024; current as of August 2026. **SP 800-53 Rev. 5** is still the current
revision (latest release 5.2); there is no Rev 6.

CSF is not a compliance obligation for this product. Its value is as **structure and translation**:
it organises the work into six functions and it has published mappings to nearly everything else,
which makes it the cheapest way to answer "does our ISO work cover what this buyer's NIST-based
questionnaire is asking".

## The six functions

| Function | What it means here |
|---|---|
| **GOVERN** (new in 2.0) | Who owns security, risk appetite, policy, supply-chain risk management, roles. The function most products skip and most auditors probe |
| **IDENTIFY** | Asset and data inventory, third-party inventory, risk assessment. For this product: the module inventory and the processor list |
| **PROTECT** | Access control, data security, platform hardening, awareness, resilience |
| **DETECT** | Logging, monitoring, anomaly detection |
| **RESPOND** | Incident response, analysis, mitigation, reporting |
| **RECOVER** | Recovery planning, restoration, communications |

GOVERN's addition in 2.0 is the substantive change: cybersecurity supply chain risk management
(C-SCRM) is elevated, which lines up with `OWASP-TOP10` A03:2025 and the CI/CD surface.

## Using CSF as the cross-walk hub

NIST publishes Online Informative References mapping CSF 2.0 to other frameworks. As of 2026 these
include SP 800-53 Rev 5.1.1 and, usefully for this product, a mapping to the **OWASP LLM Top 10
v2.0** — which gives a defensible way to bring the AI integrations into the same risk register as
everything else rather than treating them as a special case.

Practical use: pick CSF as the internal organising vocabulary, implement against `ISO-27002` and
`SA-NCA-ECC` controls, and use the mappings to answer questionnaires in whichever dialect the buyer
speaks.

## Profiles and tiers

Two concepts worth using rather than ignoring:

- **Profiles** — a Current Profile and a Target Profile, with the gap between them as the roadmap.
  This is a better shape for this product than a flat pass/fail grid, and it converts directly into
  the `gap` rows with effort estimates.
- **Tiers** (Partial → Risk Informed → Repeatable → Adaptive) — a maturity statement about how the
  organisation manages risk, not a score. Useful for setting expectations with a buyer about where
  the product is today.

## SP 800-53

Use as a **control catalogue for depth** when a control needs specifying precisely, not as an
obligation. It is a US federal catalogue; nothing here requires it. Cite it only where it adds
specificity that ISO 27002 lacks.
