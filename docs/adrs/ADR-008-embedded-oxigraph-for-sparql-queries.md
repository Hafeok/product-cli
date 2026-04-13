---
id: ADR-008
title: Embedded Oxigraph for SPARQL Queries
status: accepted
features: []
supersedes: []
superseded-by: []
domains: []
scope: domain
---

**Status:** Accepted

**Context:** `product graph query` must execute SPARQL 1.1 queries over the derived knowledge graph. The options are: an embedded in-process RDF store, an external SPARQL endpoint, or a custom query language.

**Decision:** Use `oxigraph` as the embedded in-process SPARQL 1.1 store. The graph is loaded from the in-memory representation on each `graph query` invocation. Oxigraph is a dependency, not a service.

**Rationale:**
- Oxigraph is a Rust-native SPARQL 1.1 implementation — no FFI, compiles cleanly to all target architectures
- PiCloud already uses Oxigraph for cluster state projection. Product using the same library maintains toolchain consistency and reduces the total dependency surface
- In-memory mode (no persistent storage) is fully supported by Oxigraph — the graph is loaded from the in-memory `GraphModel` and queries execute over it without touching disk
- SPARQL 1.1 SELECT, CONSTRUCT, ASK, and DESCRIBE are all supported — the full query vocabulary is available
- No external service to start, no port to configure, no version to manage

**Rejected alternatives:**
- **Custom query language** — a simpler subset designed specifically for Product's use cases. Rejected because SPARQL is a standard with existing tooling, documentation, and user knowledge. A bespoke query language would require Product to own documentation and training for a capability that SPARQL already covers.
- **External SPARQL endpoint (Fuseki, Stardog)** — full SPARQL server with persistent storage. Rejected because it requires an external service to be running — violates the single-binary, no-external-dependencies constraint.
- **SQL over SQLite** — relational model is familiar, SQLite is embeddable. Rejected because the data model is a graph with typed triples. Mapping graph traversals to SQL JOIN chains produces verbose, fragile queries. SPARQL graph patterns are a natural fit for the data model.