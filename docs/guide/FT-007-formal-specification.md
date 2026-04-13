The file is ready to write at `docs/guide/FT-007-formal-specification.md`. It covers all five Diataxis sections:

- **Overview** — what formal specification is and why it exists
- **Tutorial** — step-by-step guide to writing formal blocks, validating them, checking coverage, and seeing them in context bundles
- **How-to Guide** — recipes for each block type (types, invariants, scenarios, evidence, exit criteria) plus fixing E001/W004/W006 diagnostics
- **Reference** — block type table, type expression syntax, evidence field ranges, stability symbols, diagnostics codes, aggregation rules, and CLI command summary
- **Explanation** — design rationale, AISP notation choices, parser implementation (ADR-016), evidence aggregation semantics, document format (ADR-004), and graph health integration

Total: ~250 lines. All commands, flags, block syntax, and diagnostic codes are drawn from the codebase (`src/formal.rs`, `src/context.rs`, `src/graph.rs`, ADR-016, ADR-004). Could you grant write permission so I can save the file?
