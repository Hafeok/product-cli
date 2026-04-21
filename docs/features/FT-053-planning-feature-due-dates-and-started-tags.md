---
id: FT-053
title: Planning — Feature Due Dates and Started Tags
phase: 5
status: complete
depends-on:
- FT-036
- FT-037
- FT-041
adrs:
- ADR-036
- ADR-038
- ADR-045
tests:
- TC-636
- TC-637
- TC-638
- TC-639
- TC-640
- TC-641
- TC-642
- TC-643
- TC-644
domains:
- observability
- scheduling
domains-acknowledged:
  ADR-043: Implementation adds to existing slices (`src/verify/` for the W-class stage, `src/tags/` for the started-tag helper, `src/status/` for the due-date cell) following the plan-then-apply pattern; no monolithic handlers introduced.
  ADR-041: No absence TCs or ADR removes/deprecates interaction — `due-date` is a feature front-matter addition and the started tag extends ADR-036's existing implementation-tracking namespace.
  ADR-040: W028/W029 fire in the existing verify stage 2 (graph structure) alongside W002/W010; the planning warnings reuse the existing W-class channel and exit-code contract without adding a new verify stage or LLM-boundary hook.
  ADR-042: The exit-criteria TC (TC-644) uses the existing `exit-criteria` structural type from ADR-042 unchanged; no new TC types are introduced and the scenario/invariant partition is not touched.
---

## Description

Product tracks features with a `status` field and emits a
completion tag (`product/FT-XXX/complete`) when all TCs pass.
That answers "is it done?" but not "when is it supposed to be
done?" or "how long did it take?". Both questions want to live
next to the feature rather than in an external tracker, and
together they feed the forecasting model in
`docs/product-forecasting-spec.md`.

This feature adds one optional front-matter field (`due-date`)
and one new automatically-created git tag
(`product/FT-XXX/started`). Both are advisory: due dates never
block verification or phase completion, and the started tag
degrades gracefully when git is unavailable — the same posture
the completion tag (ADR-036) takes today.

The full spec is
[`docs/product-planning-due-date-spec.md`](/docs/product-planning-due-date-spec.md);
the pinned decisions live in the governing ADR.

---

## Depends on

- **FT-036** — tag-based implementation tracking; the started
  tag extends the `product/FT-XXX/{event}` namespace established
  by the completion tag.
- **FT-037** — drift detection / tag listing; `product tags
  list` and `product tags show` gain the started-tag surface.
- **FT-041** — request interface; the only path to set or
  delete `due-date` (and the apply hook that creates the started
  tag on first `planned → in-progress` transition).

---

## Scope of this feature

### In

1. **`due-date` feature front-matter field.** Optional ISO 8601
   date. Parser accepts `YYYY-MM-DD` exactly; any other shape
   produces E006 with the expected format. Set via
   `product request` change mutations; there is no dedicated
   CLI shortcut.
2. **`product/FT-XXX/started` annotated tag.** Created by
   `product request apply` on the first `planned → in-progress`
   transition (or at apply-time for features created directly
   with `status: in-progress`). Created once, never overwritten.
   Tag message: `FT-XXX started: status changed to in-progress`.
   Skipped with a W-class warning when git is unavailable.
3. **W028 — due-date-passed.** Fires when `due-date < today AND
   status != complete`. Reported in `product verify` stage 2 and
   in `product status`. Exit 2.
4. **W029 — due-date-approaching.** Fires when `due-date` is
   within the configured warning window AND
   `status != complete`. Default window 3 days, configurable via
   `[planning].due-date-warning-days`. Setting 0 disables W029.
   Reported alongside W028. Exit 2.
5. **`product status` due-date column.** Features with
   `due-date` render their date next to status; overdue features
   are flagged with a visible marker. Features without
   `due-date` render no date column.
6. **`product tags list --type started`.** The existing tag
   listing gains a `started` filter and the default listing
   includes started tags in the table.
7. **`[planning]` config section.** New `due-date-warning-days`
   key (default 3). Validated as a non-negative integer.
8. **Unit + integration tests** covering the W028/W029
   conditions, the tag creation semantics (first transition,
   replan, direct-in-progress), the `product status` rendering,
   and the change-request set/delete path.

### Out

- **New CLI shortcut for setting `due-date`** (`product feature
  due-date FT-XXX 2026-05-01`). All planning fields are set via
  requests; adding a shortcut expands the granular-tool surface
  for no new capability. Deferred.
- **Stored `started-at` / `completed-at` in YAML.** The tag
   timestamp is the authority (ADR-036 precedent). No new
   front-matter fields.
- **The forecasting model itself.** This feature emits the raw
   anchors (started tag, due-date field); the forecasting model
   consumes them. Separate feature work.
- **Due dates on ADRs, TCs, or DEPs.** Commitment dates apply to
   features (the unit of scope stakeholders care about). Other
   artifact types do not gain the field.
- **Blocking behaviour on missed dates.** W028/W029 are W-class
   only. No new E-class code; no phase-gate integration.

---

## Commands

No new CLI subcommands. Surfaces through:

- `product request apply` — reads `due-date` on incoming
  changes, creates the started tag on status transitions.
- `product verify` — stage 2 gains W028/W029 on features with
  `due-date`.
- `product status` — renders due-date column and overdue flag.
- `product tags list` — includes started tags and accepts
  `--type started`.

Setting a due date:
```
product request new change
product request add target FT-009
# field: due-date, value: 2026-05-01
product request submit
```

Removing a due date:
```yaml
type: change
reason: "Remove due date — commitment moved to FT-012"
changes:
  - target: FT-009
    mutations:
      - { op: delete, field: due-date }
```

---

## Implementation notes

- **`src/types.rs` — Feature struct.** Add `due_date:
  Option<chrono::NaiveDate>`. Serde attribute renames to
  `due-date` for YAML compatibility.
- **`src/parser.rs`.** Extend feature front-matter parser to
  accept the field; parse with `NaiveDate::parse_from_str(s,
  "%Y-%m-%d")` and raise E006 on failure with an
  "expected YYYY-MM-DD" hint.
- **`src/verify/` (stage 2 — graph structure).** Emit W028 and
  W029 by comparing `feature.due_date` to
  `chrono::Local::now().date_naive()` plus the configured
  warning window.
- **`src/request/apply.rs` — status transition hook.** On every
  applied `change` mutation that sets `status: in-progress` on a
  feature whose prior value is `planned` (or where the feature
  is being created with `status: in-progress`), call a new
  `tags::create_started_tag(feature_id)` helper. The helper
  checks for pre-existing tag and skips with no error.
- **`src/tags/mod.rs` (or `src/git_tags.rs`).** New
  `create_started_tag` mirroring the existing
  `create_complete_tag`. Shared git availability detection and
  W-class warning emission.
- **`src/status/` — project summary.** Extend the feature
   rendering to emit the due-date cell when set; append a marker
   glyph for overdue features.
- **`src/config.rs`.** New `[planning]` TOML section with
  `due-date-warning-days: u32` default 3.
- **File-length budget.** Every touched file must remain under
  400 lines; plan to split the tags module if the started-tag
  helpers push it over.
- **Dependencies.** `chrono` is already a direct dependency; no
  new crates required.

---

## Acceptance criteria

A developer running on a clean repository can:

1. Apply a change request that sets `due-date: 2026-05-01` on
   `FT-009` and observe the YAML parsed back round-trips the
   field exactly.
2. Apply a change request with `due-date: "not-a-date"` and
   observe E006 with an `expected YYYY-MM-DD` message; no write
   occurs.
3. Advance the clock (in fixture) past an
   `in-progress` feature's due date and run
   `product verify FT-009`; observe W028, exit code 2, and the
   feature status unchanged.
4. Set a feature's `due-date` two days in the future with
   default `due-date-warning-days = 3`; run `product status`
   and observe W029 plus the due-date column rendered.
5. Set `[planning].due-date-warning-days = 0`; observe W029 is
   not emitted regardless of the date window.
6. Apply a request that sets a feature's `status` from
   `planned` to `in-progress`; observe
   `product/FT-009/started` is created and
   `product tags list --feature FT-009` surfaces it.
7. Revert a feature to `planned` then back to `in-progress`;
   observe the original started tag's timestamp is preserved
   (no new started tag created, no overwrite).
8. Run `product verify` on a feature with no `due-date`;
   observe neither W028 nor W029 fire.
9. Run `cargo test`, `cargo clippy -- -D warnings -D
   clippy::unwrap_used`, and `cargo build` and observe all pass.

See the exit-criteria TC for the consolidated check-list.

---

## Follow-on work

- **CLI shortcut `product feature due-date FT-XXX DATE`.** If
   request-based editing proves too verbose for the common
   single-field set. Deferred until usage data exists.
- **`due-date` on phases.** A phase-level commitment date would
   feed forecasting's "will phase N ship on time?" question.
   Separate feature.
- **Burndown rendering in `product status`.** Extended output
   showing per-phase due-date distribution and overdue count.
   Pure surface — no new fields required. Deferred.
