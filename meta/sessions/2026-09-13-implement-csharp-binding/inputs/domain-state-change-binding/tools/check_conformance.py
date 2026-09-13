#!/usr/bin/env python3
"""Check a candidate specification language against the conformance criterion.

Reads a candidate manifest (YAML) and reports, per requirement, whether the
evidence the candidate published satisfies it.

WHAT THIS CHECKS
    PR-1..PR-3  a prohibition is declared AND names an enforcing construct
                (schema constraint, required field, grammar production).
                A prohibition whose only enforcement is prose FAILS: a rule
                held in prose is not enforced.
    SR-1        the build-time and act-time encodings have identical structure.
    DP-1..DP-7  both members are present and the encodings differ.

WHAT THIS DOES NOT CHECK
    That two differing encodings differ FOR THE STATED REASON. A candidate can
    satisfy every pair with encodings that differ arbitrarily. That residual is
    checked by a person reading the published evidence, and no instrument
    replaces it.

    That the act vocabulary is right, that any determination is true, or that a
    specification written in the language is complete or assured.

A green run means the notation can say what needs saying and cannot say what
must not be said. It means nothing else.

Usage:  python check_conformance.py candidate.yaml [--strict]
Exit:   0 conformant   1 non-conformant   2 malformed manifest
"""

import sys
import json
import argparse

try:
    import yaml
except ImportError:
    sys.exit("PyYAML required: pip install pyyaml --break-system-packages")

PROHIBITIONS = {
    "PR-1": "a determination with no address",
    "PR-2": "a determination with no allocation",
    "PR-3": "a coverage claim with no stated uncovered set",
}

PAIRS = {
    "DP-1": ("silent on axis", "does not travel to axis"),
    "DP-2": ("decidable in principle", "runnable at act time"),
    "DP-3": ("covered by a check", "outside its reach"),
    "DP-4": ("settled in advance", "discretion carried by an actor"),
    "DP-5": ("proxy for another predicate", "direct predicate"),
    "DP-6": ("object in read position", "object in write position"),
    "DP-7": ("external ground", "internally produced ground"),
}

# Enforcement kinds that count. Prose does not appear here by design.
ENFORCING = {"schema", "required-field", "grammar", "validator", "type"}


def shape(obj):
    """Structural signature: keys and nesting, values discarded."""
    if isinstance(obj, dict):
        return {k: shape(v) for k, v in sorted(obj.items())}
    if isinstance(obj, list):
        return [shape(v) for v in obj[:1]]  # element shape, not length
    return type(obj).__name__


def norm(obj):
    return json.dumps(obj, sort_keys=True, default=str)


def check(manifest, strict=False):
    results = []

    # --- Prohibitions -----------------------------------------------------
    declared = manifest.get("prohibitions", {})
    for pid, desc in PROHIBITIONS.items():
        entry = declared.get(pid)
        if not entry:
            results.append((pid, "FAIL", f"not declared — {desc}"))
            continue
        kind = (entry.get("enforced_by") or "").strip().lower()
        if kind not in ENFORCING:
            results.append((pid, "FAIL",
                            f"enforcement is '{kind or 'unstated'}'; prose does not enforce"))
        elif not entry.get("construct"):
            results.append((pid, "FAIL", "no enforcing construct named"))
        else:
            results.append((pid, "PASS", f"{kind}: {entry['construct']}"))

    # --- Sameness ---------------------------------------------------------
    sr = manifest.get("sameness", {}).get("SR-1")
    if not sr or "build_time" not in sr or "act_time" not in sr:
        results.append(("SR-1", "FAIL", "both encodings not published"))
    elif shape(sr["build_time"]) != shape(sr["act_time"]):
        results.append(("SR-1", "FAIL",
                        "build-time and act-time encodings have different structure"))
    else:
        results.append(("SR-1", "PASS", "identical structure"))

    # --- Discrimination pairs --------------------------------------------
    pairs = manifest.get("pairs", {})
    for pid, (a_desc, b_desc) in PAIRS.items():
        entry = pairs.get(pid)
        if not entry:
            results.append((pid, "FAIL", f"not published — {a_desc} vs {b_desc}"))
            continue
        if "a" not in entry or "b" not in entry:
            results.append((pid, "FAIL", "both members not published"))
            continue
        if entry["a"] is None or entry["b"] is None:
            results.append((pid, "FAIL", "a member is null — the distinction is unrepresentable"))
            continue
        if norm(entry["a"]) == norm(entry["b"]):
            results.append((pid, "FAIL", "encodings are identical — distinction not made"))
        else:
            results.append((pid, "PASS", "encodings differ"))

    return results


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("manifest")
    ap.add_argument("--strict", action="store_true",
                    help="reserved; no effect yet")
    args = ap.parse_args()

    try:
        with open(args.manifest) as f:
            manifest = yaml.safe_load(f)
    except FileNotFoundError:
        print(f"REFUSED: {args.manifest} not found", file=sys.stderr)
        return 2
    if not isinstance(manifest, dict):
        print("REFUSED: manifest is not a mapping", file=sys.stderr)
        return 2

    name = manifest.get("candidate", "(unnamed)")
    results = check(manifest, args.strict)

    width = max(len(r[0]) for r in results)
    print(f"\nCandidate: {name}\n")
    for rid, verdict, detail in results:
        mark = "  ok " if verdict == "PASS" else "FAIL"
        print(f"  [{mark}] {rid:<{width}}  {detail}")

    failed = [r for r in results if r[1] == "FAIL"]
    print(f"\n  {len(results) - len(failed)}/{len(results)} requirements satisfied")

    if failed:
        print(f"\n  NON-CONFORMANT. {len(failed)} requirement(s) unsatisfied.")
    else:
        print("\n  CONFORMANT on every published requirement.")

    print("\n  This checks that encodings differ. It does not check that they")
    print("  differ for the stated reason — read the published evidence.\n")

    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
