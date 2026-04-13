---
id: ADR-005
title: Numeric ID Scheme (FT-XXX, ADR-XXX, TC-XXX)
status: accepted
features: []
supersedes: []
superseded-by: []
domains: []
scope: domain
---

**Status:** Accepted

**Context:** Artifacts need stable, human-readable, machine-parseable identifiers. These IDs appear in front-matter links, CLI commands, filenames, and LLM context bundles. They must be: short enough to type, unambiguous, sortable, and stable after assignment.

**Decision:** Use prefixed zero-padded numeric IDs: `FT-001`, `ADR-001`, `TC-001`. IDs are assigned sequentially by `product feature/adr/test new`. Once assigned, IDs are permanent — artifacts are never renumbered. Retired artifacts are marked `status: abandoned`, not deleted.

**Rationale:**
- Sequential numeric IDs are common convention in engineering (JIRA, ADR numbering, RFC numbering) — contributors arrive with prior knowledge
- Prefixes (`FT`, `ADR`, `TC`) make the artifact type visible in any context where the ID appears
- Zero-padding ensures correct alphabetical sort in file listings and git diffs
- Permanent IDs mean that external references (comments in code, commit messages, slack messages) remain valid indefinitely
- The prefix is configurable in `product.toml` — teams that prefer `FEAT`, `DEC`, `TEST` can use those instead

**Rejected alternatives:**
- **Slug-based IDs** (e.g., `cluster-foundation`) — human-readable but not stable if the title changes. Two artifacts with similar titles produce collision-prone slugs.
- **UUIDs** — globally unique, collision-free. Rejected because UUIDs are unreadable in context. `FT-001` in a commit message is meaningful; `3f2504e0-4f89-11d3-9a0c-0305e82c3301` is not.
- **Semantic versioning** — expressive for API-like artifacts. Rejected because it implies a release lifecycle that does not map cleanly to features and decisions.