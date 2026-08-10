# M5 — Curation: closing report

The milestone's product is accepted content. Every entry below was proposed
in-session, presented with its ground, and accepted per entry by the
principal before the next batch was drafted. Nothing was generated in bulk.

**Store:** 24 predicates · 68 claims · 33 decisions (126 entries with
manifests and config). **This branch:** 85 entries added, 22 amended, across
17 content batches plus one format bump.

---

## 1. Coverage by phase

### Phase 1 — dogfood closure

`ddd report escapes` now prints clean in all three sections **with no
not-checkable set anywhere**, which is what the M3+M4 session left standing.

| Finding | Disposition |
|---|---|
| 7 unpinned `based_on` edges | **Resolved.** Decision-time status recovered from git (`136aa55`); none of the seven claims had moved since, so the pins are recovered rather than assumed. No late-pin marker needed. |
| 15 live claims with no `revalidate_by` | **Resolved.** Cadence set per claim from drift rate × consequence — 6 months on claims resting on a moving spec or a market read, 24 months on corpus-settled methodology. |
| 24 undeclared What boundaries | **Priced, not paid.** `dec/ddd/what-boundaries-priced-not-paid`, exposure stated and verified, review by 2027-02-07. `ddd what --strict` still exits 1, deliberately. |
| M3/M4 flagged choices (4) | **Settled.** PRD §14 q5; enforce-mode matching granularity; correspondence-row stratification; the C# enum-member policy-table gap. |

`changed` was deliberately not touched when adding cadences: a cadence is
metadata about when to recheck a finding, not a change to the finding, and
bumping it would have fired basis loss on seven decisions — reporting that
the ground moved when it had not.

### Phase 2 — the C# closure-claim seed

Three predicates, 17 claims, **every status earned by compilation rather than
recall**. Each hole builds clean under `Nullable enable` +
`TreatWarningsAsErrors` and then faults at runtime.

- `pred/code/null-dereference-safety` — 7 claims
- `pred/code/static-type-conformance` — 5 claims
- `pred/code/deterministic-disposal` — 6 claims

Two of the kickoff's assumed boundary clauses came back **corrected**, which
is the main argument that the discipline was worth its cost:

- **EF scalar materialisation is not a null-dereference hole.** A NULL column
  against a non-nullable property throws `InvalidOperationException` from the
  provider's reader before the entity is returned. The static gap is real;
  the runtime closure is the provider's. (`DDD-cs-nrt-06`)
- **The real EF hole is prescribed by EF's own documentation.** `= null!` on
  a required navigation is the documented remedy for CS8618; unloaded, it
  faults at the dereference. (`DDD-cs-nrt-05`)
- **Async disposal is open in one shape only.** Implementing *only*
  `IAsyncDisposable` under a sync `using` is `CS8418`, and the compiler names
  the fix. The gap is the ambiguity of implementing both. (`DDD-cs-disp-03`)

### Phase 3 — checklist conversion

All 19 items routed with ground attached. §6.2's eleven and §6.3's nine, one
already covered.

| Routing | Count | Examples |
|---|---|---|
| Analyzer-closable | 6 | CA1305/1307/1309, CA1031, CA2007, CA2016, `use-recent-api-versions` |
| Assertion-closable | 2 | AutoMapper coverage, EF concurrency tokens |
| Composition-closable | 1 | authorization fallback policy |
| Review over a complete worklist | 2 | Bicep parameter defaults, sizing values |
| Open | 7 | ordering, `async void`, transaction extent, RBAC scope, … |
| Already covered | 1 | `!` → `DDD-cs-nrt-04` |

Item 9 (async shape) **decomposed onto predicates already filed** rather than
needing its own: ConfigureAwait and CancellationToken are policy arguments,
HttpClient lifetime is a disposal proxy-gap instance. Only `async void`
needed a new predicate. That is the catalog composing rather than growing
linearly, and it is the first evidence that the predicate set is the right
shape.

Item 10 (concurrency) was predicted to resist the `P(c, G)` form and does
not: token presence is readable from the built model, so only *which*
entities need one stays with judgment.

---

## 2. The five closure modes

Not planned; they emerged from the routing and they are not interchangeable.
Recording them because the middle three are routinely described as "we have a
check for that" when their failure modes differ completely.

1. **Analyzer-closed** — build time, unconditional, no way to skip.
2. **Assertion-closed** — startup or test time, **conditional on the
   assertion running**. Most likely to be recorded as closed while inert.
   `dec/cs/startup-assertions-required` exists for this reason.
3. **Snapshot-closed** — an approved output shape (DAD §4.2). Closes
   serialisation contracts, which no analyzer reaches.
4. **Composition-closed** — an arrangement in the composition root. One line
   to add, one line to delete; warrants an architecture test.
5. **Review-closed over a mechanically complete worklist** — the judgment is
   human, the *enumeration* is not. DAD §11 names enumeration completeness as
   the load-bearing gap with no mechanical check; for Bicep parameter
   defaults and sizing values, there is one.

---

## 3. Manifest candidates awaiting adoption

Nine decisions are filed and accepted but operative only where a repo's
manifest cites them. None applies to this workspace, which is Rust.

| Decision | Closes | Not bought |
|---|---|---|
| `dec/cs/policy-rules-at-error` | CA1305/1307/1309/1310 | the ambient clock |
| `dec/cs/general-catch-at-error` | CA1031 | the continuation under a narrowed type |
| `dec/cs/async-policy-rules-at-error` | CA2007, CA2016 | token *presence* |
| `dec/cs/disposal-rules-at-error` | CA1001, CA2000 | async, container hand-off, HttpClient |
| `dec/cs/startup-assertions-required` | mapper coverage, EF tokens | which entities need one |
| `dec/cs/authorization-fallback-deny` | unattributed endpoints | per-endpoint policy |
| `dec/cs/unsafe-reinterpretation-banned` | `Unsafe.As`, `Marshal.PtrToStructure` | interop exception path |
| `dec/bicep/linter-at-error` | API currency, secret rules | secrets not *named* like secrets |
| `dec/bicep/psrule-in-pr-stage` | network exposure, diagnostics | RBAC scope, `existing`, sizing |

**Two adoption traps, both measured:**

- **PSRule passes vacuously.** Without the standalone Bicep CLI on `PATH` and
  every parameter resolvable, expansion fails, PSRule reports *"no matching
  rules were found"* and zero failures — indistinguishable from a clean run
  in an exit code. Adoption must assert that expansion happened.
- **The Bicep secret chain is a name heuristic.** `adminPassword` drew
  findings from both tools; renamed to `seedValue`, still leaving through a
  deployment output, it drew nothing from either. One rename defeats two
  independent tools.

---

## 4. Evidence to upgrade later

Five claims are honest about resting on something weaker than a run. Each
says so in its own `evidence` field; none should be promoted without closing
the gap.

| Claim | What is cited rather than exercised |
|---|---|
| `DDD-cs-disp-06` | socket exhaustion from per-call HttpClient — Microsoft guidance, no load test run |
| `DDD-bicep-idem-01` | that a structural `what-if` assertion supplies the criterion — no Azure subscription available |
| `DDD-cs-pol-02` | Meziantou.Analyzer as the clock rule's home — package not exercised, rule id deliberately not cited |
| `DDD-cs-obs-01` | Threading.Analyzers as `async void`'s home — same |
| `DDD-bicep-diag-01` | retention *values* — the probe never declared a setting with a retention, so this is absence of an observation, not observation of an absence |

Thirteen claims remain `projected`. Two are worth a look: **`DDD-arch-02` and
`DDD-arch-03` were exercised by M3/M4** — the LSP surface classified real
edits on real Roslyn and the interceptor demanded declarations without
extending either server. They look like `reported` candidates. Status
promotion is the principal's; they were not promoted here.

---

## 5. What M6 needs from this catalog

M6 adds the Rust adapter and gives this workspace a governed surface of its
own. Three entries are waiting for it:

- **`dec/ddd/enforce-matching-tightens-to-symbol`** — enforce-mode matching
  drops the file arm. Its acceptance test is already written as the claim's
  falsifier: a correspondence row whose `linked_declaration` names a symbol
  other than its own should not occur in enforce mode.
- **`dec/ddd/enum-member-gap-priced`** — add `enum-member` to `DECL_KINDS`
  plus a policy row. Small, adapter-local, and the first live test of
  `dec/ddd/adapter-policy-tables`, which it passed.
- **`DDD-what-04` / the boundary deferral** — reviewed 2027-02-07, or sooner
  when M6 gives the workspace a real governed surface, whichever comes first.

Beyond that, M6 is the first opportunity to put **correspondence rows** in
the store. `.ddd/seams/events/` is still empty, so `DDD-arch-04`,
`DDD-arch-05`, `DDD-arch-06` and both friction claims have no ground in this
repo at all — their six-month cadences are a bet that M6 supplies it. If it
does not, the empty recheck is itself the finding.

---

## 6. Known limits of this milestone

- **One toolchain.** Every closure claim is indexed to C# 12 / SDK 8.0.413 /
  net8.0, or to Bicep CLI 0.37.4 / PSRule 1.47.0. Nothing was checked against
  .NET 9 or 10, both of which are installed here.
- **One provider.** Every EF observation is Sqlite. `DDD-cs-nrt-06` names
  this explicitly and makes another provider behaving differently its
  falsifier.
- **No engagement repo.** The catalog has never been inherited by a second
  repo, which is PRD §12's fourth success criterion and the whole basis of
  `dec/ddd/seed-lands-in-claims-not-shared`. `shared/` is still empty by
  decision, not by omission.
- **The committed PRD is an older revision than the working draft.** Claims
  cite §-numbers from the committed copy; a re-sync should check them.
