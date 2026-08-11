# DDD Research Protocol — instruments, experiments, datasets

Split out of the PRD per `dec/ddd/prd-split`. This document holds the
research content: what is **predicted** and what has been **observed**
are strictly separated — a prediction lives here with its pre-registration
pointer; an outcome lives in the experiment report and the graph, and this
document only points at it. (This resolves the reviewed PRD's temporal
contradiction, where confirmed M6 numbers sat inside a table describing
M6 as future.)

## 1. The correspondence dataset

**Question:** does the structural cost of an interface (fan-in/out,
contract size, reference count at creation) correspond to the demand a
seam actually absorbs (`I(V;S)` vs. real interface cost)?

**Collection:** every classified interception outcome logs one row under
`.ddd/seams/events/` (schema: `ddd-core/src/seam_event.rs` — symbol,
kind, change, visibility, signature, reference count, policy rule and its
claim, mode, outcome, linked declaration). Rows accumulate as a side
effect of normal governed work; the tool never computes
information-theoretic quantities itself — analysis is offline.

**Stratification:** rows are stratified before comparison
(`dec/ddd/correspondence-rows-are-stratified`) — language, artifact
class, mode, and outcome are not poolable.

**Honest limits:** every row is an edit that was *volunteered* through
the governed path (`DDD-arch-08`; M6 report §5.3). The dataset describes
governed-path edits, not the repository's edit population; the M8
repository-diff layer is what would let bypassed changes appear at all.

## 2. The adapter-cost experiment (M6, re-tested at M7)

**Pre-registered prediction** (filed before any adapter code, commit
`a8871da`, claim `DDD-adapter-01` at `projected`): a new language costs
approximately a policy table plus LSP wiring; material excess means
language knowledge leaked into the core.

**Protocol:** build the third language adapter (Rust, rust-analyzer
host); count core changes forced; classify each as predicted or leakage.
Re-test at the falsifier: a fourth artifact class that is *not*
LSP-shaped (the M7 HTML+CSS pair, a hostless two-file producer).

**Observed — pointers, not restatement:**

- M6: [`ddd-m6-experiment.md`](ddd-m6-experiment.md); `DDD-adapter-01`
  moved `projected → reported` (leakage 2 instances / 114 lines, both
  host-layer, both predicted); `DDD-adapter-04` filed the false half —
  host lifecycle is bespoke per server and budgeted per server.
- M7: [`ddd-m7-experiment.md`](ddd-m7-experiment.md); `DDD-adapter-01`
  moved `reported → established` (zero contract-surface leakage from a
  non-LSP producer); `DDD-adapter-05` priced the producer-shape boundary
  (~58 lines / 2 assumptions at the serve layer, no surface-vocabulary
  change).

The consequences for architecture (the capability decomposition) are
recorded in [`ddd-adrs.md`](ddd-adrs.md) §5 and specified in
[`ddd-v1-spec.md`](ddd-v1-spec.md) §4.

## 3. The hand-labelled classifier corpus (named future instrument)

Not yet built. A fixture set of real changes per language, hand-labelled
contract-surface / not, measuring:

- **recall** — contract changes the classifier catches;
- **false-demand rate** — non-contract changes it demands declarations
  for (the friction number).

It is the instrument behind spec success criteria 1–2 and the acceptance
harness for policy-table amendments (a row change must not silently trade
recall for friction). Build it no later than the M8 repository-diff work,
which needs the same corpus to validate the shared classifier over diffs.

## 4. Standing failure signals

Pre-registered, unchanged: interception overridden to `off` within a week
of dogfood (friction claim falsified); `why` unused by agents
(governance-as-ground weakened); manifest diff producing only noise
(warning-swamp claim needs revision). The basis-quality audit after typed
basis shipped is filed as `DDD-method-06`'s falsifier.
