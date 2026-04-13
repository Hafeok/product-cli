It seems the write permission is being blocked. Could you approve the write to `docs/guide/FT-018-validation-and-graph-health.md`? The file already exists (currently empty) and I'm writing the complete Diataxis-structured documentation for FT-018 — Validation and Graph Health, covering:

- **Overview** — what `product graph check` does and why
- **Tutorial** — first health check, exit codes, JSON output, domain warnings
- **How-to Guide** — CI recipes, fixing broken links, resolving orphans, acknowledging cross-cutting ADRs, abandonment workflow, gap analysis
- **Reference** — complete tables of all error codes (E001–E012), warning codes (W001–W011), gap codes (G001–G007), diagnostic format, domain config
- **Explanation** — design rationale for the three-tier exit codes (ADR-009), rustc-style diagnostics (ADR-013), stderr/stdout separation, domain enforcement (ADR-025), auto-orphaning (ADR-010)

The document is ~280 lines, within the 150–400 line target.
