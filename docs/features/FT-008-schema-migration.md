---
id: FT-008
title: Schema Migration
phase: 2
status: complete
depends-on:
- FT-003
adrs:
- ADR-002
- ADR-014
- ADR-016
tests:
- TC-060
- TC-061
- TC-062
- TC-063
- TC-064
- TC-065
- TC-179
- TC-186
- TC-187
- TC-188
- TC-189
- TC-255
- TC-256
- TC-257
- TC-258
- TC-259
- TC-260
- TC-266
- TC-267
- TC-268
- TC-269
- TC-270
- TC-271
- TC-272
- TC-273
- TC-274
domains: []
domains-acknowledged: {}
---

In-place schema upgrades for front-matter when the schema version changes.

```
product migrate schema --dry-run    # report what would change without writing
product migrate schema --execute    # update all files in place
```

The `schema-version` field in `product.toml` declares the current schema version. On startup, Product validates:
- E008 — forward incompatibility (file schema version > binary schema version)
- W007 — upgrade available (file schema version < binary schema version)

Migration functions are registered per version transition (e.g., v0→v1). Each migration function transforms front-matter in place while preserving unknown fields. Concurrent `product migrate schema` commands are prevented by advisory locking (E010).

### Exit Criteria

Run `product migrate schema` on a v0 repository — all files updated, `schema-version` bumped. Run two concurrent commands — one succeeds, one exits E010. No data corruption.
