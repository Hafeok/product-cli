# Build dispatch to spark-cli

**Status:** ✅ implemented (canonical wire). `product` (the specification pillar)
hands frozen work to [`spark-cli`](https://github.com/Hafeok/spark-cli) (the
execution pillar) over the canonical [contract](https://github.com/Hafeok/ai-development-contracts)
encoding. The bridge is [`scripts/dispatch-to-spark.sh`](../scripts/dispatch-to-spark.sh).

> This is the Two-Pillars swap the [contracts tier](https://github.com/Hafeok/ai-development-contracts)
> exists to enable: replace the built-in `product build run` worker with spark's
> executor loop, without either side reaching into the other. They share exactly
> two data shapes — **WorkUnit** out, **VerdictEvent** back — and nothing else.

## Verified loop

```
$ scripts/dispatch-to-spark.sh <deliverable> --product-root <repo>
→ emit canonical WorkUnits          product build <del> --emit-seam
→ admit into spark                  spark admit <unit.json>   (homogeneity guard)
→ drain                             spark run   (or --serve for isolated production)
→ reconcile emitted verdicts        product verdict <event.json>
✓ dispatch complete
```

What now holds, tool-to-tool, both **independently built**:

- `product build --emit-seam` emits the canonical kebab-case WorkUnit — `unit-ref`,
  `tier`, `ladder-position`, `artifact-delivery`, `spmc-bundle.{model.binding,
  context-pool.fragments}`, sealed `cell-graph.cells` — and it **validates against
  the contract's own normative `work-unit.schema.json`** plus the two harness
  invariants (no cross-unit edges; every `context-refs` id resolves in-unit).
- `spark admit` parses that exact JSON (its interior maps it to spark's model at
  the seam) and `spark run`/`serve` emits a canonical kebab-case VerdictEvent
  (`tier-ran`, `cell-results`, `next-consequence`).
- `product verdict` reconciles it by `unit-ref` + `bundle-hash`.

The model binding is pinned from `.product/role-bindings.yaml` (`role → { tier,
capability-tag, provider, model-id, quantization, invocation }`); with no entry a
reported placeholder binding is emitted so the unit stays canonically valid —
**pin a served binding before real dispatch.**

## The pipeline

```
product build emit --seam DL-xxx        # freeze + emit WorkUnit JSON (one file per unit)
        │  units[]  (by value, bundle-hash identity)
        ▼
spark admit <unit.json>                 # homogeneity + structural guard, queue
spark serve                             # sandbox + brokered creds + worker + protected oracle gate
        │  VerdictEvent  (fire-and-forget)
        ▼
spark stream                            # durable, append-only verdict log (.spark/verdicts.jsonl)
        │  events[]
        ▼
product build verdict --event <e.json>  # reconcile: unit → work-unit state → deliverable done
```

The producer never calls the executor and the executor never reaches back into the
graph. Dispatch's last act is the emit; reconciliation is just reading the verdict
stream. `bundle-hash` closes the loop — every verdict is attributable to the exact
frozen unit it ran against.

## The blocker that was resolved: three JSON dialects of one contract

*(Historical — resolved by adopting the canonical wire below.)* The contract is
deliberately *field-set-and-meaning, not a wire format*, so each tool started with
its own projection — and they were **not the same concrete JSON**, so a unit
emitted by `product` would not `spark admit` unmodified:

| | Field naming | `spmc-bundle` | `cell-graph` | Extras |
|---|---|---|---|---|
| **contract 0.1.0** (canonical) | kebab-case | `model.binding{provider,model-id,quantization,invocation}` + `context-pool.fragments[]` | `{cells:[{id,requires,schema{shape-language,document},prompt{content},context-refs,output}]}` | closed (`additionalProperties:false`) |
| **product-cli** `--emit-seam` | snake_case | *pre-1.7.0 envelope* | — | one opaque `executor_extension` slot |
| **spark-cli** `admit` | snake_case | top-level `model_binding{model,quantization,params}` + `context_pool` **map** | bare **array**; `cell_id`, `depends_on`, per-cell `binding`, `prompt` string, `schema` value | `environment`, `credential_grant`, `tool_grants` |

`product-framework` (the standard) was aligned to contract 0.1.0 in v1.7.0; the two
**tools** have not caught up. Until they share one wire, dispatch cannot round-trip.

## Recommended approach — one canonical wire (contract kebab-case JSON)

Bring both tools onto the contract's normative JSON schema. Then the bridge is just
files + shell, and *any* conformant producer could drive spark later.

### product-cli changes
- **`product build emit --seam`** emits the 1.7.0 shape: kebab-case envelope with
  `tier`, `ladder-position`, `artifact-delivery`, `spmc-bundle.{model.binding,
  context-pool.fragments}`, and a sealed `cell-graph.cells[]`. Pin a full model
  binding from the deliverable's resolved role/capability (`.product/capabilities.yaml`,
  `role-bindings.yaml`).
- **`product build verdict`** validates the new VerdictEvent (`tier-ran`,
  `cell-results[]`, `next-consequence`) instead of the old `executor_extension`
  envelope, and reconciles by `unit-ref`/`bundle-hash`.
- **New `product build dispatch --executor spark DL-xxx`** (thin bridge): emit units
  → write each to a temp file → `spark mode set queue` → `spark admit <file>` per unit
  → `spark serve` → read `.spark/verdicts.jsonl` (or `spark stream`) → feed each event
  to the verdict reconciler. Honor `acceptance-class`: `auto-commit-if-green` commits
  on `accepted`; `needs-verdict` surfaces for a human.

### spark-cli changes
- Make the `interface` crate deserialize the **canonical kebab-case JSON**: add
  `#[serde(rename_all = "kebab-case")]` to `WorkUnit`, `Cell`, `ModelBinding`,
  `CellOutput`, `VerdictEvent`, and restructure to the nested shape —
  `spmc-bundle.model.binding`, `spmc-bundle.context-pool.fragments[]`,
  `cell-graph.cells[]`, `cell.requires` (rename of `depends_on`),
  `cell.schema.{shape-language,document}`, `cell.prompt.content`.
- Keep `environment` / `credential_grant` / `tool_grants` as `#[serde(default)]`
  Execution-Contract additions: absent in a bare canonical unit, defaulted to a
  conformant floor (spark already does this). They ride *outside* the closed contract
  envelope — carried in a side companion file or a spark-local extension, never inside
  the hashed `spmc-bundle`.

### Exact field map (canonical ↔ spark today)

| contract (canonical) | spark (current) |
|---|---|
| `unit-ref` / `parent-deliverable` / `bundle-hash` | `unit_ref` / `parent_deliverable` / `bundle_hash` |
| `acceptance-class` / `ladder-position` / `artifact-delivery` | `acceptance_class` / `ladder_position` / `artifact_delivery` |
| `spmc-bundle.model.binding.model-id` | `model_binding.model` |
| `spmc-bundle.model.binding.quantization` | `model_binding.quantization` |
| `spmc-bundle.model.binding.invocation` | `model_binding.params` |
| `spmc-bundle.model.binding.provider` | *(new — add)* |
| `spmc-bundle.context-pool.fragments[]{id,content,role}` | `context_pool` map `{id → {content,provenance}}` |
| `cell-graph.cells[]` | `cell_graph[]` |
| `cell.id` / `cell.requires` / `cell.context-refs` | `cell_id` / `depends_on` / `context_refs` |
| `cell.schema.{shape-language,document}` | `cell.schema` (raw value) |
| `cell.prompt.content` | `cell.prompt` (string) |
| `cell.output.{artifact-id,media-type,path}` | `cell.output.{artifact_id,media_type,path}` |
| VerdictEvent `event-id`/`emitted-at`/`tier-ran`/`cell-results`/`next-consequence` | `event_id`/`emitted_at`/`tier_ran`/`cell_results`/`next_consequence` |

Same fields throughout — the work is case + three structural moves (unit binding
nested, context pool map→array, cell-graph array→`{cells}`), not new semantics.

## Fallback — adapter (fastest to a working demo)

If aligning both crates is too much up front, keep spark's snake_case dialect and put
a **one-way mapper** in `product build dispatch`: frozen unit → spark's exact JSON →
`spark admit`. Verdicts come back in spark's snake_case; the reconciler lower-cases
the five field names. Gets a green `emit → admit → serve → verdict` loop quickly; the
mapping is spark-specific and lives on the producer side (a coupling the canonical
approach avoids).

## Phased plan — status

1. ✅ **Wire chosen** — canonical (contract kebab-case JSON).
2. ✅ **`product build --emit-seam` + `product verdict`** rewritten to the 1.7.0
   canonical shape (`product-core/src/pf/build_seam*.rs`).
3. ✅ **spark `admit`** parses the canonical WorkUnit and emits canonical
   VerdictEvents (`spark-cli/crates/interface`).
4. ✅ **Bridge** — [`scripts/dispatch-to-spark.sh`](../scripts/dispatch-to-spark.sh),
   proven end to end against a demo deliverable.
5. ⬜ **Remaining polish** — see below.

## Remaining polish / open questions

- **`--serve` in production.** The bridge defaults to the in-memory `spark run`
  demo path; `--serve` drains isolated (sandbox + brokered creds + worker +
  protected oracle) but needs `SPARK_ORACLE_CMD` and a worker configured per
  spark's `docs/running-on-spark.md`.
- **Real model bindings.** Emission floors to a reported placeholder binding when a
  role has no entry in `.product/role-bindings.yaml`. Pin served bindings there
  before dispatching to a real executor.
- **`emitted-at` format.** spark's demo `run` clock stamps a non-RFC3339
  `emitted-at`; `product verdict` is lenient, but spark's `serve` path should emit
  RFC3339 for strict `format: date-time` conformance. *(spark follow-up.)*
- **Deeper context inlining.** `--emit-seam` currently inlines a unit's declared
  `derived_from` / `applies` / `trace` ids as fragments; inlining the *resolved*
  What/How content from the graph (the full "by value" ideal) is a follow-up.
- **A first-class `product build dispatch --executor spark` subcommand** could
  replace the shell bridge once the loop stabilises.
- **Default executor.** Whether `product build run` keeps its built-in worker as an
  alternative, or spark becomes the default dispatch target.
