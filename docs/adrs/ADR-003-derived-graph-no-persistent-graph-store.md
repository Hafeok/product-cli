---
id: ADR-003
title: Derived Graph — No Persistent Graph Store
status: accepted
features: []
supersedes: []
superseded-by: []
domains: []
scope: domain
content-hash: sha256:e74a536f8d71c990df0286a70ed5e056f6e830e20ab0a5278e622d752649f271
---

**Status:** Accepted

**Context:** The knowledge graph must be queryable by the CLI. The choices are: persist the graph to a file (SQLite, RDF store, TOML index), regenerate it on every command invocation, or keep it in a daemon process.

**Decision:** Rebuild the in-memory graph from front-matter on every command invocation. The graph is never persisted. `index.ttl` is an export artefact for external tooling, never read by Product.

**Rationale:**
- A developer repository for a project like PiCloud will have on the order of 50–200 artifact files. Reading and parsing all of them takes < 50ms on any modern hardware. There is no performance case for caching.
- A persistent graph store introduces a synchronisation invariant: the graph must always match the files. This invariant is impossible to enforce perfectly (files can be edited outside Product, git operations change files without invoking the CLI). A derived graph is always correct by construction.
- No migration strategy is needed when the schema changes. Old front-matter that Product can no longer parse is reported as a warning; it does not corrupt a stored graph.
- The `index.ttl` export is a snapshot. If it is stale, `product graph rebuild` regenerates it. The CLI never depends on it being fresh.

**Rejected alternatives:**
- **SQLite index** — fast random access, good for large repositories. Rejected because the target scale (< 200 files) does not justify the added complexity of cache invalidation, migration, and the possibility of a corrupted or stale index.
- **Daemon process** — the graph stays hot in memory; file watching keeps it current. Rejected as massively over-engineered for a developer CLI tool. Daemons have startup costs, crash modes, and version skew problems.
- **`index.ttl` as read source** — `product graph rebuild` generates it; CLI reads from it. Rejected because stale `index.ttl` would silently produce wrong answers. The graph must always reflect the current file state.