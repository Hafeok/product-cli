#!/usr/bin/env bash
# docs/demo/who-did-what.sh — who FILED decisions vs who SIGNED them.
set -euo pipefail
LEDGER="${LEDGER_BIN:-./target/release/ledger}"
printf "%-34s %7s %7s\n" "identity" "FILED" "SIGNED"
"$LEDGER" log | awk '
  /^cs:/ { split($0, a, " by "); who = a[2] }
  /decision\(s\)/ {
    n = $1 + 0; acc = 0
    for (i = 1; i <= NF; i++) if ($(i+1) ~ /^acceptance/) acc = $i + 0
    if (n  > 0) filed[who] += n
    if (acc > 0) signed[who] += acc
  }
  END {
    for (w in filed) seen[w] = 1
    for (w in signed) seen[w] = 1
    for (w in seen) printf "%-34s %7d %7d\n", w, filed[w], signed[w]
  }' | sort -k2 -rn
