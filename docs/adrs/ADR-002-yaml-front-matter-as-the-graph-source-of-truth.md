---
id: ADR-002
title: YAML Front-Matter as the Graph Source of Truth
status: accepted
features: []
supersedes: []
superseded-by: []
domains: []
scope: domain
---

**Status:** Accepted

**Context:** The knowledge graph linking features, ADRs, and test criteria must be maintained somehow. The options are: (a) a separate graph file hand-maintained alongside the markdown documents, (b) inline declarations within the markdown prose, or (c) YAML front-matter in each document that declares its identity and outgoing edges.

Option (a) creates a synchronisation problem — the graph file and the document files diverge. Option (b) is ambiguous to parse and fragile as document content changes. Option (c) keeps each document self-describing. The front-matter is the contract between the document and the graph.

**Decision:** YAML front-matter in every artifact file is the sole source of truth for graph relationships. The graph is always derived from front-matter on demand; there is no persistent graph store.

**Rationale:**
- Each file is self-describing — open any file and immediately understand its place in the graph
- Git diffs on front-matter are clean and reviewable: adding a link to an ADR is a one-line change
- No synchronisation problem: the graph cannot drift from the documents because it is always recomputed from them
- YAML front-matter is a well-understood convention (Jekyll, Hugo, Obsidian, academic tools); contributors arrive with prior knowledge
- The `serde_yaml` crate deserialises front-matter into typed Rust structs in one call
- Front-matter fields are strictly typed and validated on parse — `product graph check` reports malformed declarations

**Rejected alternatives:**
- **Separate `links.toml` graph file** — clean separation of concerns, but introduces a synchronisation requirement. Every time a new artifact is added, two files must be updated. In practice, contributors update the document and forget the graph file.
- **RDF/Turtle as the primary source** — philosophically consistent with PiCloud, but Turtle is not a natural authoring format for humans writing markdown documents. It would require a separate editor workflow or tooling that does not exist yet.
- **Inline markdown annotations** — `<!-- links: ADR-002, ADR-003 -->` style comments. Fragile, non-standard, and invisible in rendered output. Harder to validate programmatically.