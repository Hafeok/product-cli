#!/usr/bin/env bash
# scripts/checks/module-structure.sh
# Checks that required top-level modules exist and main.rs is within limits.
# Exit 0: all checks pass
# Exit 1: missing modules or main.rs exceeds limit
set -euo pipefail

REQUIRED_MODULES=(graph parse context commands verify mcp io)
MISSING=()

for mod in "${REQUIRED_MODULES[@]}"; do
  if [ ! -d "src/$mod" ]; then
    MISSING+=("src/$mod/")
  fi
done

if [ ${#MISSING[@]} -gt 0 ]; then
  echo "ERROR: missing required modules:"
  for m in "${MISSING[@]}"; do echo "  $m"; done
  exit 1
fi

MAIN_LINES=$(wc -l < src/main.rs)
if [ "$MAIN_LINES" -gt 80 ]; then
  echo "ERROR: src/main.rs has $MAIN_LINES lines (limit: 80)"
  echo "  main.rs must contain only CLI dispatch — no logic."
  exit 1
fi

echo "OK: module structure valid, main.rs: $MAIN_LINES lines"
exit 0
