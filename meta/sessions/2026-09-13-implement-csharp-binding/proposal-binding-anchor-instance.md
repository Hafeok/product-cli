# Candidate schema change — the anchor instance of a pattern-level determination

**Target:** the domain state change binding, `schema/determination.schema.json` (filed here at
sha256 `4df4db09…`). **Status: `[PROPOSED]`.** Not applied; the schema is used unchanged in this
session. Filed at Emil's direction (CG-R-53) with this session as its evidence.

## The defect, as found by use

`$defs/address` requires `act_instance`. A determination that applies to every command slice — a
profile, `DSC-0002`'s payload-validation rule — therefore addresses one instance and declares
`extent.axes.slice-type: travels-to, region: all-command-slices`. That is the designed pattern and
it works.

What it leaves privileged is the anchor. `DSC-0002` is addressed to `PlaceOrder` for no reason
that `PlaceOrder` has and `EmptyCart` lacks. Delete the `PlaceOrder` slice from the event model
and `check_resolution.py` reports `DSC-0002` as an ADDRESS failure — a determination about every
command slice is orphaned by the removal of one it was never specifically about. The address
space then says the determination is dangling when its subject is intact.

This is the first schema gap found by use rather than by analysis. It surfaced when this session
tried to file a stack profile "addressed to the act type", as the C# binding PRD §3 says, and
found that the schema has no such address.

## What would close it — three shapes, none chosen here

1. **A type-level address.** `act_instance` becomes optional when `scale` is `slice` or above and
   the extent travels. Cheapest change; weakens PR-1's "address needs act type and instance"
   construct, which the conformance manifest cites by name. That citation would need re-issuing.
2. **A sentinel instance.** `act_instance: "*"` (or `all`) permitted only with a `travels-to`
   extent on `slice-type`. Keeps `act_instance` required; the sentinel is checked by the resolution
   validator rather than by the schema, exactly as instance names are today.
3. **Anchor as a listed set.** `act_instance` accepts an array when travelling; deletion of one
   member does not orphan the record. Most expressive, largest change, and it makes "which
   instances" an authored list that goes stale as slices are added.

Shape 2 is the smallest change that preserves the manifest's PR-1 citation. It is not
recommended here; it is the one this session would test first.

## What is not a defect

A determination genuinely about one instance that travels to others (a rule found at `PlaceOrder`
and generalised) is correctly anchored where it was found. The defect is only the case where no
instance is the origin — a pattern authored as a pattern.

## Evidence

- `inputs/domain-state-change-binding/examples/place-order.determinations.yaml`, `DSC-0002`.
- `inputs/domain-state-change-binding/tools/check_resolution.py`, the ADDRESS check.
- This session's `gate0-report.md` P-4, P-5, and `rulings-gate0.md` CG-R-53.
