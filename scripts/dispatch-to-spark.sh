#!/usr/bin/env bash
# Build dispatch to the Spark executor — the §5.1 seam, realised as a process
# bridge. The two pillars share no live surface: `product` emits frozen canonical
# WorkUnits by value, `spark` admits/serves them and emits VerdictEvents to its
# stream, and `product verdict` reconciles them. Fire-and-forget: the producer
# holds no knowledge of the consumer.
#
# Usage:
#   scripts/dispatch-to-spark.sh <deliverable> [--product-root DIR] [--serve]
#
# Env:
#   PRODUCT_BIN   path to the `product` binary   (default: product on PATH)
#   SPARK_BIN     path to the `spark` binary      (default: spark on PATH)
#   --serve       drain isolated via `spark serve` (sandbox+creds+worker+oracle+
#                 durable log) instead of the in-memory `spark run` demo path;
#                 requires SPARK_ORACLE_CMD (and a worker) per spark's docs.
set -euo pipefail

DELIVERABLE="${1:?usage: dispatch-to-spark.sh <deliverable> [--product-root DIR] [--serve]}"; shift || true
PRODUCT_BIN="${PRODUCT_BIN:-product}"
SPARK_BIN="${SPARK_BIN:-spark}"
ROOT="."
DRAIN="run"
while [ $# -gt 0 ]; do
  case "$1" in
    --product-root) ROOT="$2"; shift 2 ;;
    --serve) DRAIN="serve"; shift ;;
    *) echo "unknown arg: $1" >&2; exit 2 ;;
  esac
done

WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT
SEAM="$WORK/${DELIVERABLE}.seam.json"

# Both tools resolve their state from the working dir (`product` walks up to
# `.product/`; `spark` writes `.spark/` here), so run from the product root.
cd "$ROOT"

echo "→ emit canonical WorkUnits for '$DELIVERABLE'"
"$PRODUCT_BIN" build "$DELIVERABLE" --emit-seam --out "$SEAM"

echo "→ split into per-unit files"
python3 - "$SEAM" "$WORK" <<'PY'
import json, sys
units = json.load(open(sys.argv[1])); out = sys.argv[2]
for i, u in enumerate(units):
    json.dump(u, open(f"{out}/unit-{i}.json", "w"))
print(f"  {len(units)} unit(s)")
PY

echo "→ admit into spark (homogeneity guard) and drain via '$DRAIN'"
"$SPARK_BIN" mode set queue >/dev/null
for f in "$WORK"/unit-*.json; do "$SPARK_BIN" admit "$f"; done
"$SPARK_BIN" "$DRAIN"

echo "→ reconcile emitted verdicts"
# Prefer the durable log (serve); fall back to the in-memory stream (run).
python3 - "$WORK" <<'PY'
import json, os, sys
out = sys.argv[1]
events = []
if os.path.exists(".spark/verdicts.jsonl"):
    events = [json.loads(l) for l in open(".spark/verdicts.jsonl") if l.strip()]
elif os.path.exists(".spark/state.json"):
    events = json.load(open(".spark/state.json")).get("stream", [])
for i, ev in enumerate(events):
    json.dump(ev, open(f"{out}/verdict-{i}.json", "w"))
print(f"  {len(events)} verdict(s)")
PY
for f in "$WORK"/verdict-*.json; do [ -e "$f" ] && "$PRODUCT_BIN" verdict "$f"; done

echo "✓ dispatch complete"
