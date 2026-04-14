---
id: FT-020
title: Migration Path
phase: 1
status: in-progress
depends-on: []
adrs:
- ADR-017
- ADR-014
tests:
- TC-060
- TC-061
- TC-062
- TC-063
- TC-064
- TC-065
- TC-080
- TC-081
- TC-082
- TC-083
- TC-084
- TC-085
- TC-162
- TC-275
domains: []
domains-acknowledged: {}
---

Migration is a two-phase extract-then-confirm process. See ADR-017 for full heuristic specification.

```bash
# Dry run — see what would be created
product migrate from-adrs picloud-adrs.md --validate
product migrate from-prd picloud-prd.md --validate

# Execute — write files, skip existing
product migrate from-adrs picloud-adrs.md --execute
product migrate from-prd picloud-prd.md --execute

# Interactive — review each artifact before writing
product migrate from-prd picloud-prd.md --interactive

# Post-migration: fill in link gaps and generate checklist
product graph check
product checklist generate
```

The migration parser uses heading structure to detect artifact boundaries and extracts phase references, status markers, and test criteria from subsections. It does not infer `depends-on` edges or feature→ADR links — those require human review and are filled in via `product feature link` commands after migration.

The source document is never modified. Migration can be re-run safely.

---