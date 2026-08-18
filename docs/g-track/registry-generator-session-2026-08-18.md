# The registry generator — session report (G-track, 2026-08-18)

**Repository:** `Hafeok/product-cli`, branch `claude/registry-generator-mechanism-wf1dsr`.
**Gates:** 1 design · 2 implementation · 3 template changes · 4 close. All four ratified by
emil@okkels-klein.dk. **Design document:** `docs/g-track/registry-generator-design-2026-08-18.md`.

**Status: the generator is the mechanism; G0 regenerates through it.**

---

## 1. Why this session existed

The G0 session generated a registry instance with a shell one-liner and hit a **silent corruption of
the base IRI**: a sigil in a parameter value was re-read as code by the substitution engine, and the
wrong value propagated into every file including the birth provenance. A human reading the output
caught it. The base IRI is the one registry parameter with no supersession path — a changed IRI
orphans identifiers rather than superseding them (`g-dec-03`).

Two faults, not one. String substitution re-reads values as code in every engine that does it, and a
scratchpad script cannot be tested. Registry generation mints identity, records birth provenance, and
creates the artefact a founding decision is filed into. The remedy is a subcommand with fixtures.

---

## 2. The finding: the generator paid for itself before it minted anything

`graphs/canonical/_exemplar.ttl` carried its provenance attribution as
`<https://REGISTRY-HOST.example/agent/g1-session>` — a **second occurrence of the placeholder host**,
outside the `/ns#` base every other file used. Substituting the base IRI alone would have left that
IRI pointing at `REGISTRY-HOST.example` in every generated instance's data file: a corrupted host in
ratified content, of exactly the class that produced the G0 bug, and invisible to anyone reading the
diff for the parameter they supplied.

The gate's Check B found it at Gate 2, before any tree was written. **Correction applied:** the
attribution now sits in the instance's own namespace, `reg:agent-g1-session`, leaving one host token
in the template. This is the claim made for building the mechanism, demonstrated rather than argued.

---

## 3. The subcommand surface

```
product registry generate --owner <org> --repo <name> --ratifier <person> \
    --display-name <name> --base-iri <iri> --date <YYYY-MM-DD> \
    --generated-by <who> --out <dir>

product registry check <dir>
```

Slice + adapter, as the workspace does everywhere: the pure slice is `product-core/src/registry/`
(`params` · `template` · `substitute` · `verify` · `plan` · `apply` · `check/`), the thin adapter is
`product-cli/src/commands/registry.rs`. No MCP mirror: the command mutates no `.product` graph in any
repository, and repository minting does not belong inside a phase-gated authoring session.

The template stays at `docs/g-track/registry-template/` — the path the PRD and every instance's
provenance pin — and is embedded by `include_str!` with a manifest-drift test, so a template file
added without wiring fails a test rather than vanishing from generated instances.

### The public surface, as the governance gate read it

The DDD contract-surface gate failed the first CI run with **151 undischarged changes** — a fair
reading of what had been published: every internal type of the renderer, the gate and the shapes
reader stood as public API, plus a test-support module that is scaffolding rather than surface.

Two things were wrong, and both were fixed. The slice's public surface is now its **re-export list** —
the acts (`plan_generation`, `apply_generation`, `check_instance`, `evaluate`) with the types they
carry — and the modules beneath are crate-internal, so a caller depends on the acts and never on how
the template is rendered or how the gate is spelled. That took the governed surface from 151 events
to **59**, which is the surface actually intended. Those 59 are discharged by ten seam declarations
under `.ddd/seams/`, each carrying what a caller learns at that boundary *and what the boundary
cannot do* — the apply seam states that publishing is not among its powers, the check seam states its
own limit as a second reader of pySHACL's rules, the params seam states that validation judges
meaning and never character safety.

One lint surfaced with the narrowing rather than being introduced by it: clippy's
`enum_variant_names` does not fire on public enums, so the shapes reader's constraint enum was
renamed `Kind` → `Rule` to keep its variants mirroring SHACL's own constraint-component names.

### The generate / publish split

**Generation is local and offline.** It renders in memory, gates, refuses a target that is not empty,
writes only beneath `--out`, then `git init` and **one** birth commit with inline identity, no system
or global git config, and author/committer dates pinned to the mint date — so the same parameters
through the same generator produce the same tree bytes and the same commit id.

**Publication is a separate act and is not a subcommand.** No remote is configured, no network call is
made, no repository is created anywhere. The two commands that would publish are printed for a human
to run. The boundary is where it is because minting identity and publishing it answer to different
authorities, and because only the local half can be a fixture — if publication rode inside `generate`,
the untested half would ride on the tested half's back, which is how G0's one-liner shipped a
corrupted base IRI with a green-looking run.

---

## 4. The substitution mechanism

Typed arguments; no `--set KEY=VALUE` table, no parameter file. The renderer walks the template **once**:
at each character boundary it matches the longest token from a closed table, copies the parameter's
bytes to the output, records the span it wrote, and advances the cursor past the token. The cursor
never moves backwards and the output is never re-scanned.

The invariant, stated so a test can assert it: *the output is the concatenation of literal spans of
the template with byte-for-byte copies of parameter values.* Therefore `&`, `\1`, `$0`, a backslash, a
pipe, a backtick, `$(…)` are data — there is no replacement grammar to re-read them in — and
substitution order is irrelevant, because there is no order.

Sequential `str::replace` is the mechanism that would nearly have worked and does not: a value
inserted by an earlier replacement sits in the buffer that later replacements scan. The fixture
`sequential_replace_would_have_re_read_it` runs both on the same input and shows the naive one
producing `SECOND` where the renderer produces the value that was supplied. That is a bug this session
would have shipped, kept as a test rather than a comment.

**No templating dependency.** Every candidate engine brings an expression language and an escaping
mode — surface whose purpose is to interpret its inputs. The renderer is ~60 lines and closes the last
hole; auditing an engine's replacement semantics would cost more than the lines it replaced.

---

## 5. The verification gate

Runs on the in-memory tree **before the first byte reaches disk**. A failure is a refusal, not a
warning: no target directory is created at all.

- **Check A — round-trip.** The renderer's recorded spans are read back out of the rendered bytes and
  compared to the parameters; the per-parameter tally is cross-checked against a naive occurrence
  count over the template (two mechanisms counting the same thing); every parameter must reach the
  output at least once. This is what would have caught the G0 bug, loudly, at generation time.
- **Check B — no survivors.** No `{{IDENT}}` and no base-IRI sentinel may remain. A placeholder the
  template carries without a typed parameter therefore fails generation rather than shipping.
- **`TEMPLATE.md` is asserted, not exempted** — byte-identical to the template's own copy, because it
  travels verbatim as the instance's record of what its placeholders were.

### The value-span masking rule

Check B reads the template text that *survived* rendering — the rendered file with every recorded
value span masked out — not the raw output. The first run of the hostile-value fixture refused a
display name containing `{{OWNER_ORG}}`. Refusing it would have been fail-closed but wrong: **"values
are data" that stops holding once a value looks like a template is not a property at all.** Masking
keeps both guarantees whole — an unwired placeholder *outside* a value still aborts, and a value is
data end to end. The same reasoning applies to the base-IRI sentinel: a supplied IRI containing
`REGISTRY-HOST.example` is the parameter, not a survivor.

---

## 6. Fixtures — 70 new tests, all passing

52 unit tests in `product-core/src/registry/`, 17 generation fixtures in
`product-cli/tests/registry_generator.rs`, 1 divergence measure in
`product-cli/tests/registry_shacl_divergence.rs`.

| Fixture | Asserts | Result |
|---|---|---|
| `parameters_round_trip_byte_identically` | every parameter byte-identical at every site | pass |
| `g0_regression_a_sigil_in_the_ratifiers_address_survives` | strike out every whole value; **no fragment** of the address or base IRI remains anywhere — the shape the G0 corruption took | pass |
| `the_g0_handoff_parameters_mint_the_handed_over_instance` | the handoff's five parameters + mint date; base IRI `tag:emil@okkels-klein.dk,2026-08-17:ground/` byte for byte, in the prefix, the `reg:baseIri` literal, the shapes, the exemplars | pass |
| `hostile_parameter_values_are_data` | `&`, `\1`, `$0`, `` ` ``, `$(…)`, `|`, `/`, a literal `{{OWNER_ORG}}` | pass |
| `sequential_replace_would_have_re_read_it` | the bug that would have shipped, shown failing | pass |
| `no_placeholder_survives_the_generated_tree` | zero across the tree | pass |
| `template_md_travels_verbatim` | byte-identical to the template's copy, placeholders intact | pass |
| `the_birth_provenance_is_complete_as_rdf` | queried as RDF, not string-matched: both versions, every parameter, the agent | pass |
| `a_fresh_instance_passes_its_own_rules` | file rule + shapes clean on a fresh instance | pass |
| Negative shapes | institutional Reading without a trust decision · two assertions in one file · decision missing its falsifier · decision dated by a bare string · provenance outside the vocabulary · an empty file | all **fail** as required |
| `unsupported_shacl_construct_fails_closed` · `an_unreadable_shape_fails_closed_with_its_own_heading` | an unreadable shape fails, in its own section, with no violations section | pass |
| `a_refused_parameter_writes_nothing` | gate refuses → **no target directory** | pass |
| `generation_refuses_a_non_empty_target` | prior content untouched | pass |
| `generation_is_deterministic` | identical bytes **and** identical commit id | pass |
| `two_generations_in_one_run_do_not_interfere` · `generation_leaves_no_residue_outside_its_target` | teardown provable | pass |
| `generation_configures_no_remote` | no remote; publish commands printed, never run | pass |
| `embedded_template_matches_on_disk` · version-constant drift | manifest and header cannot drift | pass |

Workspace gates green at every hold: `cargo t` (1352 tests, 0 failed), `cargo clippy --workspace -- -D
warnings -D clippy::unwrap_used`, `cargo xtask check`.

---

## 7. The template at 0.2.0

One bump covering every change this session made — the template is not released between gates, so two
bumps would record a state that never existed.

- **`shapes/decision.ttl`** — title, resolution, region, falsifier, ratifier, status, date required;
  `basis` / `acceptedCost` / `revisitIf` deliberately **not**, because a decision may honestly have
  none and demanding them produces filler. `reg:made` is constrained to `xsd:date`. `reg:ratifiedBy`
  is left unconstrained as to node kind: whether a ratifier is named by literal or IRI depends on
  whether the owner has a durable identifier for people, which is an instance's ruling — the G0
  instance uses an email literal, an institutional instance might use an IRI.
- **`graphs/canonical/_exemplar-decision.ttl`** — a conforming example, so a ratifier learns the form
  from a file rather than from SHACL. It carries `{{RATIFIER}}` so the form looks like theirs, with a
  status stating plainly that it is not ground.
- **The CI file rule** — generalised from a filename exemption to the class rule it stood in for:
  exactly one assertion **or** exactly one decision per `graphs/**/*.ttl`, no exemptions. The
  zero-triple tolerance went with the empty founding-decision file, which is no longer shipped: the
  slot is a path the ratifier creates. A rule with an exception decays.
- **TEMPLATE.md step 1** no longer reads both ways — the file travels **verbatim**, unsubstituted.
- **The base IRI documents two routes** with their trade-offs: a host the owner controls durably
  (resolvable from day one, and a real commitment through rebrands and transfers), or
  location-independent minting per `g-dec-03` (`tag:`/`urn:`, HTTP form published later as a
  projection through a rebasing parameter, accepted cost an internal-to-published mapping). Neither is
  the default. The README records that **which route an instance took is a decision filed in that
  registry, not a fact recoverable from the IRI** — a `tag:` IRI records the mint date and authority
  but not whether the deferral was deliberate or a durable host was rejected.
- **`GENERATION.ttl` is parameterised**, filled by the same renderer under the same gate, recording
  the generator's version alongside the template's, and correcting the PROV: `prov:wasGeneratedBy` the
  act, `prov:wasAttributedTo` the agent.

---

## 8. The pySHACL divergence measure

The instance's own CI runs `pyshacl`; the fixtures run a fail-closed native reader that evaluates a
defined SHACL subset compiled to SPARQL over oxigraph, because a gate depending on a Python toolchain
becomes a habit the moment it is skipped when absent. Anything outside the subset is reported as
**unevaluable** and fails the check — never silently skipped, and reported distinctly from data
violating a shape.

Two readers of one rule set is a **standing finding**, so it is measured rather than assumed small.
`the_native_reader_agrees_with_pyshacl` runs both over nine cases whenever pySHACL is present and
fails on any disagreement; when absent it says on stderr that the divergence was not measured.

**Run in this session** — pySHACL 0.40.1, nine cases: shipped assertion exemplar, shipped decision
exemplar, conforming Reading, institutional without a trust decision, institutional with one,
provenance outside the vocabulary, assertion without a predicate, decision missing its falsifier,
decision dated by a bare string. **No disagreement.** Non-vacuity confirmed: pySHACL independently
reports `Conforms: False` on the negatives, and a fresh 0.2.0 instance was additionally validated by
the instance workflow's own two steps run verbatim (`pyshacl -a` over `shapes/`, and the workflow's
rdflib file-rule script) — both clean.

---

## 9. What G0 must do differently when it resumes

1. **Regenerate through the subcommand.** No shell substitution, no hand-editing of the tree. The
   invocation is in §3 with the handoff's parameters; the `the_g0_handoff_parameters_...` fixture
   pins it.
2. **The same five parameters and the same mint date, 2026-08-17.** The mint date records when the
   authority was demonstrably controlled, not when the tree was written; it does not move because the
   tree was rewritten.
3. **File the two ratified texts verbatim** — the founding decision and `g-dec-03`, from
   *Founding content, ratified as text*. They were ratified as text, not as the tree they were
   written into; a paraphrase at this handoff is the drift the founding decision's own
   non-graduation clause guards against.
4. **The founding decision creates its file.** `graphs/canonical/founding-decision.ttl` no longer
   ships as an empty slot; the ratifier's first merge creates it, conforming to
   `shapes/decision.ttl`.
5. **Do not re-apply the two template additions as instance-local.** `shapes/decision.ttl` and the
   generalised file rule are in the template at 0.2.0; an instance that re-adds them locally forks
   the template.
6. **The instance records template 0.2.0 and the generator's version.** Re-pinning is possible
   against either.
7. **Publishing stays a separate, explicit act.** `Hafeok/ground-registry-g0` is still empty; the
   generator prints the two commands and pushes nothing.

---

## 10. What this session did not touch

G0 itself; the extractor; any instance published to any remote (`Hafeok/ground-registry-g0` remains
empty); the PRD; canon. No repository was created under any organisation. The session pushed to its
own branch and nowhere else.
