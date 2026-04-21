---
id: ADR-045
title: Planning Annotations — Due Dates as Advisory Signals and Started Tags as Cycle-Time Anchors
status: proposed
features:
- FT-053
supersedes: []
superseded-by: []
domains:
- observability
- scheduling
scope: domain
---

**Status:** Proposed

**Context:** Product models features with a `status` field
(`planned` / `in-progress` / `complete` / `abandoned`) and emits
a completion tag (`product/FT-XXX/complete`, ADR-036) when
`product verify` confirms all TCs pass. That model answers
"is it done?" but not two questions that project execution
increasingly needs to answer:

1. **"When should it be done?"** — stakeholder commitments,
   customer-contracted dates, sprint boundaries. Currently
   recorded in external tools (issue trackers, spreadsheets) and
   disconnected from the feature artifact that carries the actual
   scope.
2. **"How long did it take?"** — cycle time from start to
   complete. The `complete` tag gives the end; there is no
   corresponding anchor for the start. ADR-036 deliberately
   excluded started-tags from its scope ("ADRs don't have a clear
   `implemented` moment; features have `product verify`"), leaving
   the cycle-time question open.

Both questions want to live next to the feature, in the graph,
not in a parallel system. Answer (1) informs prioritisation and
status reports; answer (2) is the raw input for the forecasting
model in `docs/product-forecasting-spec.md`. Neither should gate
implementation — a missed due date is a signal, not an error.

**Decision:** Add one optional feature front-matter field
(`due-date`) and one new automatically-created git tag
(`product/FT-XXX/started`). Both are advisory: `due-date` never
blocks verification, phase gates, or `graph check`; the started
tag is best-effort and degrades gracefully when git is
unavailable, mirroring ADR-036's posture.

### Decisions pinned by this ADR

1. **`due-date` is optional, ISO 8601 date (YYYY-MM-DD), never
   automatic.** Features without it work identically to today.
   Product never assigns or infers a due date; it is always set
   explicitly via a change request. Precision is the day — no
   datetime, no timezone.
2. **`due-date` is purely advisory.** It has no effect on phase
   gate evaluation (FT-036), feature completion, TC execution, or
   `graph check` exit codes. The only surfaces are W028, W029,
   and the `product status` rendering.
3. **W028 and W029 are W-class (exit 2).** Both fire in the graph
   structure stage of `product verify` and in `product status`.
   Missing a due date never produces exit 1. W028 fires when
   `due-date < today AND status != complete`. W029 fires when
   `due-date` is within the configured warning window AND
   `status != complete`.
4. **Warning window is configurable via `due-date-warning-days`
   in `[planning]`.** Default 3. Setting to 0 disables W029.
5. **`product/FT-XXX/started` is created at most once per
   feature.** On the first `planned → in-progress` transition
   detected by `product request apply`. Subsequent replans
   (`in-progress → planned → in-progress`) do not overwrite the
   original tag — the earliest start timestamp is the honest
   anchor for cycle-time.
6. **Features created directly with `status: in-progress` also
   get the tag at apply time.** There is no prior `planned`
   state to transition from, but the feature is already in flight
   and the tag must exist for the forecasting model to compute
   cycle-time.
7. **Tag creation is best-effort.** If the working directory is
   not a git repo, or git is unavailable, the started-tag
   creation is skipped with a W-class warning — identical to
   ADR-036's posture for the completion tag. Product never blocks
   apply on tag failure.
8. **Cycle-time is a derived quantity, not stored.** The tag
   timestamp (`git log -1 --format=%aI`) is the authority for
   `started-at`, matching ADR-036's treatment of `completed-at`.
   No new front-matter field for `started-at` or `completed-at`.
9. **Tag annotation includes the reason it was created.** Message:
   `FT-XXX started: status changed to in-progress`. Mirrors the
   completion tag's format.

### Tag namespace summary (extends ADR-036)

| Tag | Created by | When |
|---|---|---|
| `product/FT-XXX/started` | `product request apply` | First `in-progress` transition (this ADR) |
| `product/FT-XXX/complete` | `product verify FT-XXX` | All TCs passing (ADR-036) |
| `product/FT-XXX/complete-vN` | `product verify FT-XXX` | Re-verification (ADR-036) |

`product tags list` and `product tags show` are extended to
surface started tags; `--type started` and `--type complete`
filters match the new namespace.

**Rationale:**

- **Advisory, never gating.** The project lore is full of cases
  where due dates slip because scope changed, not because work
  was slow. A gating due date would push users to fudge the
  field; an advisory signal encourages honest recording.
- **One tag per feature.** Replans are common and often represent
   de-prioritisation, not start-over. Preserving the earliest
   start timestamp gives forecasting a stable baseline.
- **Best-effort tagging.** Matches ADR-036's contract: Product
   never requires git. Adopting a repo mid-flight (features
   already `in-progress`) must not emit errors about missing
   historical tags.
- **Derived timestamps.** Writing `started-at` to YAML would
   duplicate the tag's timestamp, introduce a source-of-truth
   question, and create a field that goes stale on rebases.
   Keeping the tag as the authority eliminates the class of bug.
- **W028 / W029 as the surface.** `product verify` already emits
   W-class findings for W002 (orphan), W010 (domain gap),
   W-path-absolute (FT-051). Planning warnings fit the same
   channel and exit-code contract (ADR-009).

**Rejected alternatives:**

- **Make `due-date` block phase completion.** Would turn
   missed-date into a build failure, which punishes honest
   recording and changes the contract that `product verify`
   reports implementation status, not scheduling status.
- **Store `started-at` and `completed-at` in feature
   front-matter.** Same rename/rebase problems that led ADR-036
   to avoid commit SHAs in YAML. Derived from tags is cleaner.
- **Auto-set `due-date` from a `due-in-days` shorthand.**
   Tempting for brevity but adds a time-dependent field whose
   meaning depends on when it was set. ISO dates are
   unambiguous.
- **Support datetime precision.** Day precision matches how
   humans actually commit to delivery ("by Friday",
   "by 2026-05-01"). Sub-day precision adds schema surface and
   timezone bugs for no real benefit.
- **Warn on the due-date field missing.** Rejected — optional
   means optional. A project that does not track commitments in
   Product must not be nagged.
- **Combine started and complete tags into a single
   `implementation` tag namespace with events.** Tempting for
   symmetry but diverges from ADR-036's chosen `{event}`
   suffix pattern. Keeping `product/FT-XXX/{started,complete}`
   preserves the existing tag browsing ergonomics.

### Test coverage

| Decision | Covered by TC (title) |
|---|---|
| `due-date` parses ISO 8601 | `due-date-field-parses-iso-8601` |
| `due-date` is advisory | `due-date-never-blocks-verify-or-phase-gate` |
| W028 on overdue | `w028-fires-when-overdue-not-complete` |
| W029 window + disable | `w029-fires-within-warning-window` |
| Started-tag first transition | `started-tag-created-on-in-progress-transition` |
| Started-tag not overwritten | `started-tag-not-recreated-on-replan` |
| `status` renders due date | `product-status-shows-due-date-column-and-overdue-flag` |
| Request sets/deletes field | `change-request-sets-and-deletes-due-date` |
