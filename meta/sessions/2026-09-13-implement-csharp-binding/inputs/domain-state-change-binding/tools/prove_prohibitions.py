#!/usr/bin/env python3
"""Prove PR-1..PR-3 are enforced by construction.

Each case below is a determination the criterion says a conforming language
must be UNABLE to express. Each must be REJECTED by the schema. A case that
validates is a failed prohibition and the binding is non-conforming.
"""
import json, yaml, sys
from jsonschema import Draft202012Validator

v = Draft202012Validator(json.load(open("schema/determination.schema.json")))
base = yaml.safe_load(open("examples/place-order.determinations.yaml"))[1]

def strip(d, *path):
    import copy
    o = copy.deepcopy(d)
    t = o
    for k in path[:-1]:
        t = t[k]
    t.pop(path[-1], None)
    return o

cases = [
    ("PR-1", "determination with no address", strip(base, "address")),
    ("PR-2", "determination with no allocation", strip(base, "allocation")),
    ("PR-3", "coverage claim with no uncovered set",
             strip(base, "allocation", "acceptance", "does_not_cover")),
    ("PR-2b", "residual with no accountable principal", {
        **strip(base, "allocation"),
        "allocation": {"class": "residual", "carried_by": "machine"}}),
    ("PR-2c", "machine named as accountable principal", {
        **strip(base, "allocation"),
        "allocation": {"class": "residual", "carried_by": "machine",
                       "principal": {"kind": "machine", "identifier": "gpt"}}}),
    ("DP-5", "proxy with no stated divergence", {
        **base,
        "allocation": {**base["allocation"], "acceptance": {
            **base["allocation"]["acceptance"],
            "proxy": {"stands_in_for": "the real predicate"}}}}),
    ("DP-1", "silent axis carrying a region", {
        **base,
        "extent": {"axes": {"sector": {"state": "silent", "region": "healthcare"}}}}),
]

fails = 0
for pid, desc, doc in cases:
    errs = list(v.iter_errors(doc))
    if errs:
        print(f"  ok   {pid:<6} rejected — {desc}")
    else:
        fails += 1
        print(f"  FAIL {pid:<6} ACCEPTED — {desc}  (prohibition not enforced)")

print(f"\n  {len(cases)-fails}/{len(cases)} forbidden shapes are unrepresentable")
sys.exit(1 if fails else 0)
