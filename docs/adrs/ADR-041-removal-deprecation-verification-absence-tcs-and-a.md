---
id: ADR-041
title: Removal & Deprecation Verification — Absence TCs and ADR `removes`/`deprecates` Fields
status: proposed
features:
- FT-047
- FT-048
supersedes: []
superseded-by: []
domains:
- api
- data-model
- error-handling
scope: cross-cutting
---

**Status:** Proposed

**Context:** ADRs frequently mandate the *removal* or *deprecation* of something —
a dependency, a CLI command, a class, a configuration key, a front-matter field.
The current TC model can only express positive assertions: "the system does X". It
cannot express the negative assertion that drives every removal decision: "the
system no longer does X" or "X is no longer present anywhere in the codebase".

As a consequence, an ADR that says "Replace AutoMapper with manual mapping",
"Migrate from EF Core 6 to EF Core 8", or "`source-files` in ADR front-matter is
deprecated in favour of git tags" produces no machine-checkable artefact. The
decision is recorded in prose. Whether the decision was *enforced* in the code is
invisible to the graph. Drift detection (ADR-036) catches divergence between spec
and code at the file level, but it does not understand semantic absence — it
cannot say "the AutoMapper NuGet package is still referenced in this `.csproj`
file even though ADR-019 declares it removed".

Three concrete failure modes follow:

1. **Untracked re-introduction.** A library is removed, six months later a
   developer reintroduces it via a transitive dependency. Nothing in the spec
   layer notices.
2. **Declared-but-unenforced removals.** An ADR lists removals in its prose with
   no automated check that the removal occurred. Code review is the only gate, and
   code review forgets.
3. **Silent deprecation.** A front-matter field is deprecated by an accepted ADR
   but Product continues to read it without telling the user the field is on its
   way out. Migration never happens because nothing reminds the user.

The standalone specification at `docs/product-removal-deprecation-spec.md`
describes the design in detail, including runner patterns for several language
ecosystems. This ADR pins the architectural decisions; the spec covers the
operational detail.

---

**Decision:** Add a new TC type `absence`, add two new ADR front-matter fields
`removes` and `deprecates`, and add three new validation codes (G009, W022, W023)
to enforce the contract that every declared removal or deprecation has a linked
absence TC. Product makes no judgement about *what* is being removed — the
`removes`/`deprecates` strings are freeform — but it does enforce that the
enforcement TC exists.

---

### 1. New TC type: `absence`

Same front-matter structure as every other TC type. Same runner model. The
semantic difference is that the assertion is negative: the runner exits 0 when
the thing is *gone* and non-zero when the thing is still present. Absence TCs run
via `product verify --platform` because they assert facts about the whole
codebase, not about a single feature's behaviour.

An absence TC sets `validates.adrs` to the governing ADR(s) and leaves
`validates.features` empty — this is the structural marker that distinguishes a
cross-cutting absence assertion from a feature-scoped scenario assertion. The
`tc-type` enum gains `absence` alongside the existing `scenario | invariant |
chaos | exit-criteria` set.

### 2. New ADR fields: `removes` and `deprecates`

Both are arrays of freeform strings. They describe what the ADR mandates be
removed or deprecated, in human-readable form. Product never parses or
interprets them — they exist for two purposes:

- To make the ADR self-documenting about what it eliminates.
- To drive G009 / W022 enforcement.

```yaml
removes:
  - AutoMapper NuGet package
  - IMapper interface usage
  - CreateMap configuration calls
deprecates:
  - source-files          # ADR front-matter field, replaced by git tags
```

The fields default to empty arrays. An ADR with non-empty `removes` or
`deprecates` and no linked TC of `tc-type: absence` triggers G009 (gap analysis,
structural) and W022 (graph check, structural).

### 3. New codes

| Code | Tier | Severity | Condition |
|---|---|---|---|
| G009 | Gap | high | ADR has `removes` or `deprecates` entries with no linked `absence` TC |
| W022 | Validation | warning | Same condition as G009, surfaced by `product graph check` |
| W023 | Validation | warning | A front-matter field declared `deprecated` by an accepted ADR is encountered during graph construction |

G009 and W022 are the same structural rule surfaced through two interfaces.
`product gap check` exists for spec-quality review; `product graph check`
exists for structural validity. Both audiences need to be told. The rule is
computed once, surfaced twice.

W023 is informational and never blocks. The deprecated field is still parsed
and the graph still builds — backward compatibility is preserved. The warning's
job is to remind the user to migrate, not to break their repo.

### 4. Migration lifecycle

An absence TC progresses through the standard TC status machine, with the
addition that during a migration period, two TCs typically coexist:

```
Phase 1 (migration in progress):
  TC for deprecation warning  →  passing (warning emitted)
  TC for absence              →  failing (thing still present)

Phase 2 (migration complete):
  TC for deprecation warning  →  unrunnable (acknowledged, superseded)
  TC for absence              →  passing (thing gone)
```

Product does not encode this transition automatically. The author marks the
phase-1 TC as `unrunnable` with a reason when the phase-2 TC begins to pass.

---

⟦Γ:Invariants⟧{
  every_adr_with_nonempty_removes_has_a_linked_absence_tc
  every_adr_with_nonempty_deprecates_has_a_linked_absence_tc
  absence_tc_validates_adrs_set_features_empty_set
  absence_tc_passes_when_runner_exits_zero
  absence_tc_fails_when_runner_exits_nonzero
  w023_emitted_for_every_field_in_an_accepted_adr_deprecates_list
  w023_never_blocks_field_is_still_processed
  absence_tcs_run_under_product_verify_platform
}

⟦Ε⟧⟨δ≜1.0;φ≜100;τ≜◊⁺⟩

**Evidence TCs:** TC-586 (absence passes), TC-587 (absence fails), TC-588
(platform verify runs absence TCs), TC-589 (removes parses), TC-590
(deprecates parses), TC-591 (G009 fires), TC-592 (W022 fires), TC-593 (G009
clears when TC linked), TC-594 (W023 fires), TC-595 (deprecated field still
processed), TC-596 (W023 names ADR), TC-597 (phase-1 deprecation passes),
TC-598 (phase-2 absence passes), TC-599 (phase-2 phase-1 unrunnable does not
block), TC-600 (consolidated exit criteria).

---

**Rationale:**

- **Negative assertions are first-class architectural facts.** Half of every
  non-trivial migration is "make sure the old thing is gone". Encoding only
  positive assertions in the spec layer leaves the other half invisible. Adding
  a TC type for absence elevates removal to the same epistemic status as
  behaviour.
- **Freeform `removes`/`deprecates` strings keep the schema simple.** Product
  never has to model "what is a NuGet package", "what is a CLI command", "what
  is a front-matter field". The user knows what they're removing; Product just
  records the declaration and enforces that an enforcement TC exists. This is
  the same shape as `domains-acknowledged`: Product does not interpret the
  reasoning string, it only enforces that one is present.
- **Two codes for the same condition is correct, not redundant.** G009 fires
  in the spec-quality pipeline (`product gap check`), where the audience is
  the author. W022 fires in the structural pipeline (`product graph check`),
  where the audience is anyone running `product verify`. Suppressing W022
  because G009 covers the same condition would silence half the audience.
- **W023 as a non-blocking warning is the only safe default.** Hard-failing
  on a deprecated field would break every existing repo on Product upgrade.
  Silently ignoring deprecation gives the user no signal to migrate. The
  middle path — process the field, emit a warning that names the deprecating
  ADR — preserves compatibility while creating the migration prompt.
- **Absence TCs share the runner model with every other TC.** No new runner
  type, no new pipeline, no new prerequisites. The runner is a shell command
  that exits 0 if the thing is gone. Bash is the lowest-common-denominator
  verifier for "is this dependency referenced anywhere", "does this file
  exist", "does this CLI subcommand error out". Language-agnostic by design.
- **Run via `product verify --platform`.** Absence TCs are cross-cutting
  assertions about the whole codebase; they do not belong to any one feature.
  The platform verify pipeline is where cross-cutting assertions live (ADR-040,
  stage 6 of the unified verify pipeline). Routing absence TCs through the
  same pipeline keeps the operational surface stable.

**Rejected alternatives:**

- **Encode `removes`/`deprecates` as structured objects** (e.g., `{type: nuget,
  name: AutoMapper}`). Rejected because the value space is unbounded across
  ecosystems (NuGet, npm, cargo, pip, Maven, gem, Go module, system package,
  file path, CLI command, env var, header, kernel module, ...) and Product
  cannot model them all. Freeform strings push the semantics to the runner
  script, which is the correct place for ecosystem-specific knowledge.
- **Reuse `tc-type: scenario` with a convention** (e.g., title prefix
  "absence:"). Rejected because conventions are invisible to validation.
  G009/W022 need a structural marker they can pin without parsing prose.
  A new `tc-type` enum value is the cheapest unambiguous marker.
- **Make `removes`/`deprecates` a single field with a discriminator
  substring** (e.g., `removes: ["DEPRECATED:source-files", ...]`). Rejected
  for the same reason as the previous alternative — overloads one field with
  two semantics, makes the W023 lookup ambiguous, and obscures intent.
- **Compute G009 only on demand, no graph-check warning.** Rejected because
  gap check is opt-in for many teams; the structural graph check runs in CI
  on every commit. The condition is structural and cheap; the cost of double
  reporting is zero, the cost of single reporting is silent drift.
- **Block ADR acceptance until at least one absence TC exists for every
  `removes` entry.** Rejected because it conflates authoring with
  verification. The author may legitimately accept the ADR before the TC is
  written — G009/W022 then drive the work to completion. Blocking acceptance
  would create a chicken-and-egg situation where the author cannot record the
  decision until they have proven the removal, but proving the removal often
  requires the decision to be accepted first.
- **Hard-fail on encountering a deprecated front-matter field (W023 → E\*).**
  Rejected because it breaks backward compatibility on Product upgrade. The
  deprecation lifecycle is: ADR accepted → W023 surfaces → user migrates →
  eventually a follow-on ADR removes the field reader entirely (which is
  itself an absence TC). Hard-fail short-circuits the lifecycle.

**Test coverage:** TC-586 through TC-600 (FT-047). See the standalone spec at
`docs/product-removal-deprecation-spec.md` for runner pattern reference and
session-test-to-TC mapping.
