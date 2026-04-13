It looks like file write permissions haven't been granted yet. Could you approve the write permission so I can save the documentation file to `docs/guide/FT-017-generated-checklist.md`?

The document is ready — it covers all five Diátaxis sections:

- **Overview**: What the generated checklist is and why it exists (ADR-007)
- **Tutorial**: Step-by-step first generation, status update + regenerate, and auto-regeneration via `product verify`
- **How-to Guide**: Bulk status changes, CI integration, changing output path, idempotency verification
- **Reference**: Command syntax, `product.toml` config, file format, status marker table (all 12 markers from the source code), ordering rules, write behavior, and automatic regeneration
- **Explanation**: Design rationale (ADR-007), idempotency (TC-159), CI exit code relationship (ADR-009), topological ordering
