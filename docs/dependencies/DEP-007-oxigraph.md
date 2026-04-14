---
id: DEP-007
title: oxigraph
type: library
source: "https://crates.io/crates/oxigraph"
version: "0.4"
status: active
features:
  - FT-016
  - FT-024
adrs:
  - ADR-008
availability-check: "cargo check"
breaking-change-risk: medium
---

# oxigraph

Embedded RDF store and SPARQL query engine. Powers the `product graph ttl` export and SPARQL query execution in `rdf.rs`. Loads the knowledge graph into an in-memory RDF store for graph queries. Used with `default-features = false`.
