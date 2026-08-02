# Predicate Entry Format — v1

Schema for filing a predicate instance in the catalog. An entry instantiates the definition in
`predicate-definition.md`; every required element there appears as a required field here.

Entries are definitional objects. **They carry no status field.** Closure findings are claims,
filed separately, referenced by id. This split is enforced structurally, not by convention.

---

## Schema

```yaml
predicate:
  id:                  # stable identifier: pred/<domain>/<name>
  format: 1            # entry format version, independent of content
  name:                # short human name
  statement:           # the acceptance relation in one sentence:
                       #   accept c iff <condition over c and G>

  artifact_class:      # what c ranges over — exactly one of:
                       #   code | configuration | execution | interface | data | composition

  ground:              # what the check must see; each item carries provenance
    - fact:            #   the required information
      provenance:      #   controlled | observed | inferred | institutional

  tolerance:           # the declared bound; without this the entry is invalid
  rejection_witness:   # what a reject produces — evidence a violation exists

  depends_on: []       # predicate ids this one presupposes (catalog graph edges)

  closure_claims: []   # claim ids only — closure lives in the claim graph, with
                       #   status and falsifiers, never in this entry

  proxy_gap:           # optional; for gameable predicates: the value this
                       #   predicate proxies and how they can diverge

  notes:               # optional
```

## Field rules

- **id** is permanent. Renames create a new entry with a supersedes note.
- **statement** names both `c` and `G`. A statement that mentions no ground is suspect.
- **artifact_class** disambiguates same-named predicates. Race-freedom over `execution` and
  race-freedom over `code` are two entries.
- **ground** with an `observed` or `inferred` item cannot be closed at build time by any
  arrangement. This follows from the signature; no claim is needed to establish it.
- **depends_on** is acyclic. Composites do not get entries; they decompose into edges.
- **closure_claims** may be empty. An entry with no closure claims is a defined predicate
  nobody has yet made a finding about — valid, and honest.

---

## Example

```yaml
predicate:
  id: pred/data/shape-conformance
  format: 1
  name: Data-shape conformance
  statement: >
    Accept payload c iff c conforms exactly to the declared schema in G,
    with no missing, extra, or type-mismatched fields.

  artifact_class: data

  ground:
    - fact: the declared schema for the boundary
      provenance: controlled
    - fact: the payload as received at the boundary
      provenance: observed

  tolerance: exact structural match; no coercion, no unknown-field passthrough

  rejection_witness: the first violating field path and the violated constraint

  depends_on:
    - pred/data/well-formedness    # payload must parse before shape can be judged

  closure_claims:
    - DDD-cat-01   # "TypeScript strict mode operationally closes this at compile
                   #  time for values that never cross a serialisation boundary"
    - DDD-cat-02   # "runtime schema validation operationally closes this at the
                   #  boundary, at per-request cost"

  proxy_gap: >
    Conformance proxies validity. A payload can conform to the schema and still be
    semantically invalid; that residue belongs to domain-invariant predicates.

  notes: >
    The observed-ground item is why no static arrangement closes this for external
    input: the payload does not exist at build time.
```
