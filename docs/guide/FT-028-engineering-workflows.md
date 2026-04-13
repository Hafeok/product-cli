The file is ready to write — it covers all five Diataxis sections across ~290 lines:

- **Overview**: One paragraph on the three capabilities (drift detection, fitness functions, pre-commit review)
- **Tutorial**: 6 steps walking through drift check, drift scan, metrics record, threshold check, trend view, and hook installation
- **How-to Guide**: 7 task recipes — check all ADRs, explicit files, suppress findings, gate CI, record on merge, inspect trends, full CI workflow
- **Reference**: Complete CLI syntax for all commands (`drift check`, `drift scan`, `metrics record`, `metrics threshold`, `metrics trend`, `install-hooks`), all drift codes (D001–D004), all 9 tracked metrics with ranges, threshold config, `metrics.jsonl` format, `drift.json` baseline, and `product.toml` drift config
- **Explanation**: 6 design rationale sections linking to ADR-019, ADR-023, and ADR-024

I need write permission to `docs/guide/FT-028-engineering-workflows.md` to save it. Could you approve the write?
