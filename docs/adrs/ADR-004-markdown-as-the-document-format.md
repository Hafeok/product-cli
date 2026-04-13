---
id: ADR-004
title: Markdown as the Document Format
status: accepted
features: []
supersedes: []
superseded-by: []
domains: []
scope: domain
---

**Status:** Accepted

**Context:** Artifact files must be human-readable, diffable in git, renderable on GitHub and GitLab, and directly injectable into LLM context windows without transformation. The format choice affects authoring ergonomics, tooling availability, and the cost of the context bundle assembly step.

**Decision:** All artifact files are CommonMark markdown with YAML front-matter. No other format is supported.

**Rationale:**
- Markdown renders natively on every git hosting platform — no separate documentation pipeline required
- Markdown is the native input format for LLM context injection; no conversion step needed in context bundle assembly
- `pulldown-cmark` provides a robust, spec-compliant Rust parser
- GitHub Copilot, Cursor, and most LLM-assisted editors have first-class markdown support
- Front-matter stripping (removing the `---` block before injection) is a trivial string operation
- Code blocks, tables, and headings are all expressible in markdown — sufficient for the content patterns in features, ADRs, and test criteria

**Rejected alternatives:**
- **AsciiDoc** — more expressive than markdown, better tooling for long documents. Rejected because it does not render on GitHub by default, and LLM context injection requires an extra conversion step.
- **TOML/structured data** — fully machine-readable, no parsing ambiguity. Rejected because ADRs and features contain substantial prose (rationale, context, rejected alternatives) that is not natural to express in structured data.
- **Org-mode** — excellent for Emacs users. Rejected due to minimal tooling outside Emacs and no native renderer on git hosting platforms.