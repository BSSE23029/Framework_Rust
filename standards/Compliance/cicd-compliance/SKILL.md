---
name: cicd-compliance
description: Audits the delivery pipeline as its own compliance surface — GitHub Actions workflows, Docker image build, Azure Container Registry and Container Apps deployment, secrets and cloud credentials, change control, provenance and SBOM, environment separation. Use for any compliance question about build, release or deployment.
---

# CI/CD and deployment compliance

Surface: `.github/workflows/`, `backend/.github/workflows/`, `backend/Dockerfile`,
`frontend/Dockerfile`, and the Azure target (resource group `framework-rg`, registry
`frameworkacr123`, container apps `web-dev-hms` / `api-dev-hms`). Graded **separately**.

Auditors treat the pipeline as a production system, because it can deploy anything to production
and it holds credentials to the cloud tenancy. Most teams grade it as an afterthought and lose
points on change management, not on code.

## What to check

### C-01 Cloud credential model
`Deploay-DEV.yml` authenticates with `azure/login@v1` using `secrets.AZURE_CREDENTIALS` — a
long-lived service-principal secret. **Workload identity federation (OIDC) is the current
expectation**: short-lived, no stored secret. Grade credential type, scope (is the SP scoped to
`framework-rg` or to the subscription?), and rotation.
Sources: `VENDOR-GITHUB-ACTIONS`, `SA-NCA-ECC`, `SOC2-TSC` CC6, `ISO-27002`.

### C-02 Action pinning and supply chain
`actions/checkout@v4`, `azure/login@v1`, `appleboy/ssh-action@v1.0.3` are referenced by tag. A tag
is mutable; pin third-party actions by commit SHA. This is squarely `OWASP-TOP10` A03:2025
Software Supply Chain Failures. Sources: `SLSA`, `NIST-SSDF` PW/PS, `VENDOR-GITHUB-ACTIONS`.

### C-03 Pipeline security gates
Inventory what runs before an image reaches production. Currently the dev pipeline builds, pushes
and deploys with no test, lint, SAST, dependency scan, secret scan, container scan, SBOM or
provenance step. Each missing gate is its own row — do not collapse them.
Sources: `NIST-SSDF` (PO, PS, PW, RV), `SOC2-TSC` CC8, `PCI-DSS` 6, `ISO-27002`, `SA-NCA-ECC`.

### C-04 Provenance and SBOM
Is an SBOM produced per image and retained? Are build attestations generated and verified at
deploy? SLSA Build L1 is basically free; L3 needs a hardened builder with non-falsifiable
provenance, reachable via GitHub OIDC + Sigstore/cosign. Sources: `SLSA`, `NIST-SSDF`.

### C-05 Change control and approvals
Push to `dev` deploys automatically. For anything guest-facing or production, check branch
protection, required reviews, required status checks, environment protection rules and manual
approval, plus whether the deployment is recorded (who, what commit, when) in a way an auditor can
read. Sources: `SOC2-TSC` CC8, `ISO-27002`, `SA-NCA-ECC`, `PCI-DSS` 6.5.

### C-06 The SSH deploy path
`backend/.github/workflows/deploy.yml` SSHes to an Azure VM with a stored private key and runs
`sudo git fetch` / `sudo git reset --hard` / `sudo npm ci` / `sudo pm2 restart`. Deploying by
resetting a working tree on a host, as root, with an unpinned dependency install, is a separate and
weaker change-control path than the container pipeline. Grade it on its own row and establish
whether it still targets anything live. Sources: `SOC2-TSC` CC8, `SA-NCA-ECC`, `NIST-SSDF`.

### C-07 Secrets in build arguments
The frontend image receives `VITE_SECRET_KEY`, `VITE_PLATFORM_KEY` and friends as `--build-arg`.
Build args are recorded in image history and the values are compiled into public JS. Cross-reference
`frontend-compliance` F-01. Also check for secrets echoed into workflow logs.
Sources: `OWASP-ASVS`, `SA-NCA-ECC`, `VENDOR-GITHUB-ACTIONS`.

### C-08 Container hardening
`backend/Dockerfile` runs as **root**, `COPY . .` copies the whole context (check `.dockerignore`
exists so `sample_env`, `Uploads/`, `data/`, `.git` never land in the image), has no healthcheck and
no pinned base digest. `frontend/Dockerfile` builds with `npm install --legacy-peer-deps` and
`CI=false` — non-reproducible, and it suppresses build failures. Nginx serves with no security
headers. Sources: `SA-NCA-CCC`, `ISO-27017`, `NIST-SSDF`, `PCI-DSS` 2.

### C-09 Registry and image lifecycle
ACR access control, whether images are scanned in the registry, immutability of tags, and retention
of old images. Tags are `dev-hms-<sha>` which is good for traceability. Sources: `SA-NCA-CCC`,
`ISO-27017`, `SOC2-TSC` CC7.

### C-10 Environment separation and data residency
Is there a real separation between dev, staging and production — separate resource groups,
registries, databases, secrets? Does any non-production environment hold real guest data? That is
a PDPL finding, not a hygiene one. Then: which **Azure region** do the container apps and data
run in, and does that satisfy the tenant's residency position? A Saudi Arabia Azure region was
announced for 2026 — verify current availability before promising in-Kingdom hosting.
Sources: `SA-PDPL-TRANSFER`, `SA-CST-CCRF`, `SA-NCA-CCC`, `VENDOR-AZURE`.

### C-11 AI agents in the pipeline
`backend/.github/workflows/` contains `ai-agent.yml`, `ai-handler.yml`, `ai-review.yml`. An AI
workflow with repository write access and a model in the loop is an agentic system inside the
supply chain: check what triggers it, whether untrusted input (issue/PR text from outside) can
reach the prompt, what token scope it holds, and whether it can push or deploy.
Sources: `OWASP-LLM-TOP10` (incl. the Agentic list), `SLSA`, `VENDOR-GITHUB-ACTIONS`.

### C-12 Logging, monitoring and incident response for the pipeline
Are deployment and workflow logs retained, is failed-deploy alerting in place, and does the incident
response plan cover a compromised pipeline or leaked cloud credential?
Sources: `SOC2-TSC` CC7, `SA-NCA-ECC`, `ISO-27002`, `NIST-CSF` (DETECT/RESPOND).

### C-13 Business continuity of delivery
Can the product be rebuilt and redeployed from source if the registry, the runner or a maintainer's
access disappears? Documented, and tested. Sources: `ISO-22301`, `SOC2-TSC` A1.

## Grading notes

- Pipeline rows are almost always `n/a` for tax and guest-data obligations — record the `n/a` with
  the reason, except for C-10 where non-production copies of guest data make PDPL apply directly.
- Evidence is a workflow path plus line, e.g. `.github/workflows/Deploay-DEV.yml:41`.
- Two pipelines exist with different security postures. Never grade them as one.
