#!/usr/bin/env python3
"""Validate determination records against the binding's schema.

The schema IS the prohibition enforcement (PR-1..PR-3). A record omitting an
address, an allocation, or an uncovered set has no valid representation, so
this run is the evidence a conformance claim cites.
"""
import sys, json, yaml
from jsonschema import Draft202012Validator

schema = json.load(open("schema/determination.schema.json"))
v = Draft202012Validator(schema)
records = yaml.safe_load(open(sys.argv[1]))

bad = 0
for r in records:
    errs = sorted(v.iter_errors(r), key=lambda e: list(e.path))
    rid = r.get("id", "(no id)")
    if errs:
        bad += 1
        print(f"  FAIL {rid}")
        for e in errs:
            loc = "/".join(str(p) for p in e.path) or "<root>"
            print(f"       {loc}: {e.message[:90]}")
    else:
        print(f"  ok   {rid}  {r['provenance']['made_at']:<10} {r['allocation']['class']}")

print(f"\n  {len(records)-bad}/{len(records)} valid")
sys.exit(1 if bad else 0)
