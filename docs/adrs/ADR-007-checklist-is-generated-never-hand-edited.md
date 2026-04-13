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

**Context:** The existing workflow uses `checklist.md` as the source of truth for implementation status. Developers tick boxes in the checklist to mark work complete. This creates a problem: the checklist and the front-matter can diverge. If someone updates a feature's status in front-matter but forgets to tick the checklist (or vice versa), the two sources disagree.

**Decision:** `checklist.md` is a generated document. Implementation status is owned by the `status` field in each artifact's front-matter. `product checklist generate` regenerates `checklist.md` from the current front-matter state. The checklist file includes a warning header directing contributors not to edit it directly.

**Rationale:**
- Single source of truth: status lives in one place (front-matter), not two (front-matter + checklist)
- The checklist becomes a view, not a store. It can be regenerated at any time without loss of information
- Git history on individual feature files shows who changed the status of that feature and when — a much finer-grained audit trail than a single checklist file with many concurrent edits
- `product status FT-001 complete` updates front-matter and can regenerate the checklist in one command — the developer never needs to find and tick the right box

**Migration note:** The existing `checklist.md` in PiCloud's repository should be treated as the initial status snapshot. During migration, `product migrate` reads checked boxes in the existing checklist and populates `status` fields in the scaffolded feature files accordingly.

**Rejected alternatives:**
- **Checklist as source of truth, front-matter derived** — reverses the ownership. Markdown checkbox state is harder to parse programmatically than a YAML enum field. Also, checklist entries lack the structure to express the distinction between `planned`, `in-progress`, `complete`, and `abandoned`.
- **Both are sources of truth (sync on conflict)** — any two-source-of-truth design requires a merge strategy. Merge strategies for status fields have no correct answer when they diverge. Reject this entire class of design.