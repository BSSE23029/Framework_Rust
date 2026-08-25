---
name: management-systems
description: ISO 9001 quality management and ISO 22301 business continuity — including the imminent ISO 9001:2026 revision — plus the disaster-recovery and continuity controls other frameworks require. Use for quality-system, continuity, backup/DR or availability questions.
---

# Quality and continuity management systems

Registry: `ISO-9001`, `ISO-22301`.

Both are organisational management-system standards. Neither is a code audit, and both produce
findings that engineering cannot close alone — mark those rows `organisational` in the grid.

## ISO 9001 — check the date before you start

| | |
|---|---|
| Current edition | **2015** |
| Next edition | **published 16 September 2026** — at FDIS now, weeks away |
| Transition deadline | **September 2029** (three years) |
| Nature of change | Evolutionary: quality culture, ethical behaviour, clearer risk management. Core requirements intact |

**Do not start a 9001 gap analysis against the 2015 text without recording this.** Anything begun
now should be built to accommodate the 2026 revision, and any statement to a customer about ISO 9001
should mention which edition.

For a software product, 9001 mostly formalises things a competent team already does: defined
processes, requirements traceability, defect management, customer feedback handling, supplier
control, competence records, and continual improvement. Its real value here is commercial — some
enterprise and government procurement in the region asks for it.

## ISO 22301 — business continuity

Current: **2019**, plus **Amendment 1:2024** (climate change consideration).

The structure that matters:

| ID | Requirement |
|---|---|
| BC-01 | Business impact analysis — which functions, how long can they be down, what does that cost |
| BC-02 | Risk assessment for disruption scenarios |
| BC-03 | Recovery objectives: **RTO** and **RPO** stated per service and agreed with the business |
| BC-04 | Continuity strategies and resources to meet them |
| BC-05 | Documented continuity and recovery plans |
| BC-06 | Incident response structure and communications, including to tenants |
| BC-07 | **Exercising and testing** — with records |
| BC-08 | Performance evaluation and improvement |

For a PMS the BIA is unusually concrete: if the system is down, a hotel cannot check guests in,
cannot produce a compliant invoice, and cannot transmit to Shomoos. That is a licensing exposure for
the property, not just a service inconvenience — which sets a much shorter tolerable outage than a
typical SaaS.

## The DR controls other frameworks demand

Backup and recovery appear in `SA-NCA-ECC`, `SOC2-TSC` (Availability), `ISO-27002` and `PCI-DSS`.
Grade these once and cite them all:

1. Backups exist, on a defined schedule, per data store (DB, S3/`Uploads/`, configuration, secrets)
2. Backups are **encrypted**, and the key is not stored with them
3. Backup **location and region** are consistent with the PDPL transfer position — a backup in
   another country is a transfer
4. Retention aligned to both tax minimums and privacy maximums
5. **Restoration tested**, with a date and a result. Untested backup = `gap`
6. RTO/RPO documented and actually achievable by the tested procedure
7. Rebuild-from-source capability if the registry, runners or a maintainer's access is lost
8. Tenant-facing communication path during an outage

## Grid guidance

These rows land mostly on the **backend** (data protection, restore) and **CI/CD** (rebuild,
environment recreation) surfaces, with the management-system requirements marked `organisational`.
Availability rows are also `SOC2-TSC` A-series evidence — cite both.
