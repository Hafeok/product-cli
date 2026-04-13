The documentation is ready to write to `docs/guide/FT-023-agent-orchestration.md`. It covers all five Diataxis sections (~195 lines):

- **Overview** — what agent orchestration is and why it exists (ADR-021)
- **Tutorial** — step-by-step: first implementation, dry-run preview, verify, TC runner configuration
- **How-to Guide** — six task recipes: dry-run inspection, skip verify, headless mode, fixing E009 gap gate failures, adding runner config, checking TC status after verify
- **Reference** — exact CLI syntax and all three flags (`--dry-run`, `--no-verify`, `--headless`), pipeline step table, temp file paths, error codes, supported runners table, TC front-matter fields read/written by verify, status update rules, agent invocation commands
- **Explanation** — four design rationale sections: hard gap gate, advisory drift, file-based context, unrunnable TC handling, and the relationship between implement/verify and other commands

All commands, flags, and behavior are verified against the actual clap definitions and `implement.rs` source. Could you approve the file write?
