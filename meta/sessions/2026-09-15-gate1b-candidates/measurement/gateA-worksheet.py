#!/usr/bin/env python3
"""Build the ratification worksheet for A from the committed Gate A outputs (CG-R-114, CG-R-115).

Reads measurement/gateA-A-candidates.json (the instrument's set), ground-truth-A-entry-points.yaml
(the hand enumeration, for the three not-visible integration points) and, for the 23 candidates
whose identity is incomplete, the route string as written in the source file the inventory names
for the type — labelled a source reading for confirmation, never a supplied route. Every decision
field is left for Emil. Pre-filled only where a ruling already decided: CG-R-112 (health probes)
and CG-R-115's default for candidates outside the signal-bearing subset.
"""
import json, re, sys, yaml
from pathlib import Path

D = Path(__file__).resolve().parent.parent
SRC = Path(sys.argv[1]) if len(sys.argv) > 1 else None  # the eShopOnWeb checkout (optional)
r = json.load(open(D / "measurement/gateA-r3-A-candidates.json"))
gt = yaml.safe_load(open(D / "ground-truth-A-entry-points.yaml"))

signal = set()
for p in r["overlaps"]["cross_type_pairs"]:
    signal.add(p["a"]); signal.add(p["b"])

ROUTE_CALL = re.compile(r'^\s*(Get|Post|Put|Delete|Patch|Head)\("([^"]*)"\)')

def source_reading(c):
    """The route string in the type's file, by line — a reading of source, for confirmation."""
    if SRC is None or c["kind"] != "fastendpoints":
        return None
    # the inventory's file field is on the type; find it through the id's type name
    tfile = next((t for t in types if t["id"] == c["type_id"]), None)
    if not tfile:
        return None
    path = SRC / tfile["file"]
    if not path.exists():
        return {"file": tfile["file"], "note": "file not found in the checkout"}
    for n, line in enumerate(path.read_text().splitlines(), 1):
        m = ROUTE_CALL.match(line)
        if m:
            return {"verb": m.group(1).upper(), "route": m.group(2), "file": f"{tfile['file']}:{n}"}
    return {"file": tfile["file"], "note": "no route call found on one line"}

types = json.load(open(sys.argv[2]))["types"] if len(sys.argv) > 2 else []

def row(c):
    incomplete = c["identity"] != "complete"
    e = {
        "id": c["id"],
        "kind": c["kind"],
        "method": c["method"],
        "route": None if incomplete else c["path"],
        "identity": c["identity"],
        "signal_bearing": c["id"] in signal,
        "observed": {
            "authorisation": c["observed"]["authorisation"],
            "argument_symbols": c["observed"]["argument_symbols"],
            "identity_checks": len(c["observed"]["identity_checks"]),
        },
        "facts_by_proxy": {"read": c["facts"]["read"], "written": c["facts"]["written"], "possibly_written": c["facts"]["possibly_written"]},
        # --- Emil's fields (CG-R-106, CG-R-108, CG-R-109, CG-R-107) ---
        "decision": None,
        "act": None,
        "settles": None,
        "principal": None,
        "expected_actor_kinds": None,
        "population_order_of_magnitude": None,
        "rate_order_of_magnitude": None,
        "channel": None,
        "supported_throughput": {"sustained": None, "peak": None, "window": None, "behaviour_above_limit": None},
        "reject_reason": None,
    }
    if incomplete:
        e["route_supplied_at_ratification"] = None
        sr = source_reading(c)
        if sr:
            e["route_source_reading_for_confirmation"] = sr
    if c["id"] not in signal:
        e["default_if_unnamed"] = "reject — not ratified; no intent-holder available (CG-R-115)"
    return e

rows = [row(c) for c in r["candidates"]]
for e in gt.get("not_visible", []):
    h = {
        "id": f"{e['kind']}:{e['member']}#{e['method']} ({e['path']})",
        "kind": e["kind"],
        "method": e["method"],
        "route": e["path"],
        "identity": f"hand-supplied — not derivable by the instrument ({e.get('why', '')}); enters under CG-R-112",
        "signal_bearing": False,
        "decision": None, "act": None, "settles": None, "principal": None,
        "expected_actor_kinds": None, "population_order_of_magnitude": None, "rate_order_of_magnitude": None, "channel": None,
        "supported_throughput": {"sustained": None, "peak": None, "window": None, "behaviour_above_limit": None},
        "reject_reason": None,
    }
    if e["kind"] == "health-check":
        h["decision"] = "reject"
        h["reject_reason"] = "supplies no determination; liveness signal only"
        h["ruled_by"] = "CG-R-112"
    rows.append(h)

doc = {
    "worksheet": "ratification of A's candidate set — Gate 1b, Gate A",
    "vocabulary_grade": "stand-in (CG-R-115): no intent-holder for eShopOnWeb; every figure computed against the accepted subset carries this grade",
    "ratifier": "Emil, as stand-in",
    "rules": [
        "accepting a candidate supplies its act name, what it settles and the principal (CG-R-106); the session fills none of these",
        "expected actor kinds reference Layer 1 kinds; population and rate are orders of magnitude (CG-R-108)",
        "supported throughput is a determination: sustained, peak, window, behaviour above the limit (CG-R-109)",
        "channel is a named extent axis (CG-R-107); the Blazor client is an actor kind, not a candidate (CG-R-113)",
        "a candidate with identity incomplete is accepted only with its route supplied by hand from the source (CG-R-114); route_source_reading_for_confirmation is the session's reading, not a supplied route",
        "a rejection carries a reason (CG-R-106); candidates outside the signal-bearing subset default to the CG-R-115 reason unless named",
    ],
    "counts": {"derived": len(r["candidates"]), "signal_bearing": len(signal), "hand_supplied": len(gt.get("not_visible", [])), "identity_incomplete": sum(1 for c in r["candidates"] if c["identity"] != "complete")},
    "candidates": rows,
}
out = D / "ratification-A.yaml"
out.write_text(yaml.safe_dump(doc, sort_keys=False, allow_unicode=True, width=120))
print(f"{out}: {len(rows)} rows, {len(signal)} signal-bearing, {doc['counts']['identity_incomplete']} identity incomplete")
