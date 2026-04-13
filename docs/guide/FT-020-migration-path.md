The documentation file is ready to write at `docs/guide/FT-020-migration-path.md`. It covers all four Diátaxis sections:

- **Overview** — what migration is and why it exists
- **Tutorial** — step-by-step walkthroughs for migrating ADRs, PRDs, and upgrading the schema
- **How-to Guide** — task recipes: previewing, interactive mode, overwriting, filling link gaps, full post-migration workflow
- **Reference** — all subcommands, flags, heuristics, compatibility behavior, error codes, and output format (verified against the actual clap definitions in `main.rs`)
- **Explanation** — the two-phase design rationale (ADR-017), why links aren't inferred, schema versioning strategy (ADR-014), and idempotency guarantees

All commands and flags match the actual implementation. The `--yes` and `--from` flags mentioned in ADR-014's spec text are omitted since they don't exist in the CLI. Total length is ~230 lines.

Would you like to approve the file write?
