---
id: ADR-007
title: Checklist is Generated, Never Hand-Edited
status: accepted
features: []
supersedes: []
superseded-by: []
domains: []
scope: domain
---

**Status:** Accepted

**Context:** The original workflow used `checklist.md` as the source of truth for implementation status — developers ticked boxes to mark work complete. This design had a divergence problem: front-matter and checklist could disagree. Since then, the Product toolchain has matured: `product verify` updates TC and feature status directly in front-matter, `product status` renders phase gate state and exit criteria progress in the terminal, `product feature next` uses topological sort to determine what to implement next, and agents call `product_feature_list` rather than reading a file. Implementation status now lives entirely in front-matter. Agents no longer need checklist.md.

**Decision:** `checklist.md` is a generated human-readable view for stakeholders and GitHub rendering. It is not a data source, not an agent input, and not a source of truth. Implementation status is owned exclusively by feature and TC front-matter. `product checklist generate` produces `checklist.md` on demand. The file is listed in `.gitignore` by default — it is a local rendering, not a committed artifact, unless the project explicitly chooses to commit it for GitHub visibility.

**Rationale:**
- Front-matter is the single source of truth. Checklist.md is a projection of that truth, not a parallel record.
- Agents use `product_feature_list`, `product status`, and `product feature next` — none of these require checklist.md to exist. Removing checklist.md from the committed repository eliminates a file that can silently go stale.
- The legitimate remaining use case — "show a stakeholder what's been built without requiring Product to be installed" — is served by generating the file on demand and either sharing it or committing it deliberately. The default is not to commit it.
- GitHub renders markdown checkboxes natively. For projects that want GitHub visibility of implementation status, committing checklist.md remains valid — the project sets `checklist-in-gitignore = false` in `product.toml`.

**Migration note:** The existing `checklist.md` in PiCloud's repository should be treated as the initial status snapshot. During migration, `product migrate` reads checked boxes in the existing checklist and populates `status` fields in the scaffolded feature files accordingly. After migration, checklist.md is redundant as a data source.

**Rejected alternatives:**
- **Checklist as source of truth, front-matter derived** — reverses the ownership. Markdown checkbox state is harder to parse programmatically than a YAML enum field. Checklist entries cannot express the distinction between `planned`, `in-progress`, `complete`, and `abandoned`.
- **Both are sources of truth (sync on conflict)** — any two-source-of-truth design requires a merge strategy. Merge strategies for status fields have no correct answer when they diverge. Reject this entire class of design.
- **Remove checklist.md entirely** — loses the legitimate stakeholder and GitHub rendering use case. The file is genuinely useful as an occasional generated snapshot. Keeping it as an optional view rather than a required artifact is the right balance.