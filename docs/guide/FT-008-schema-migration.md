The file has been written to `docs/guide/FT-008-schema-migration.md`. Here's a summary of what's covered (~170 lines):

- **Overview**: What schema migration is and why it exists
- **Tutorial**: Step-by-step walkthrough from checking `schema-version` through dry-run, execution, and verification
- **How-to Guide**: Five recipes — previewing, upgrading from any version, suppressing W007, handling E008, and recovering from partial failures
- **Reference**: Exact command syntax, `product.toml` config keys, diagnostic codes (E008/E010/W007), output formats, migration behaviour details, and the known v0→v1 migration
- **Explanation**: Design rationale for integer versioning vs semver, hard-error on forward incompatibility, unknown field preservation, atomic writes + advisory locking, why `product.toml` updates last, and the relationship to the knowledge graph — all linking back to ADR-002, ADR-003, and ADR-014
