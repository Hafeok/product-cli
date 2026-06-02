#!/usr/bin/env bash
# scripts/checks/module-structure.sh
# Checks that required top-level modules exist and main.rs is within limits.
# Exit 0: all checks pass
# Exit 1: missing modules or main.rs exceeds limit
set -euo pipefail

MISSING=()

if [ -d product-core/src ] || [ -d product-cli/src ] || [ -d product-mcp/src ]; then
  # Workspace mode (FT-107): modules live under per-crate src/ trees.
  declare -A REQUIRED_PATHS=(
    [graph]=product-core/src/graph
    [parse]=product-core/src/parse
    [context]=product-core/src/context
    [verify]=product-core/src/verify
    [io]=product-core/src/io
    [commands]=product-cli/src/commands
    [mcp]=product-mcp/src
  )
  for mod in "${!REQUIRED_PATHS[@]}"; do
    if [ ! -d "${REQUIRED_PATHS[$mod]}" ]; then
      MISSING+=("${REQUIRED_PATHS[$mod]} (module: $mod)")
    fi
  done
  MAIN_PATH=product-cli/src/main.rs
else
  # Single-crate fallback — used by tempdir-based unit tests for this script.
  REQUIRED_MODULES=(graph parse context commands verify mcp io)
  for mod in "${REQUIRED_MODULES[@]}"; do
    if [ ! -d "src/$mod" ]; then
      MISSING+=("src/$mod/")
    fi
  done
  MAIN_PATH=src/main.rs
fi

if [ ${#MISSING[@]} -gt 0 ]; then
  echo "ERROR: missing required modules:"
  for m in "${MISSING[@]}"; do echo "  $m"; done
  exit 1
fi

if [ ! -f "$MAIN_PATH" ]; then
  echo "OK: module structure valid (no main.rs to check)"
  exit 0
fi

MAIN_LINES=$(wc -l < "$MAIN_PATH")
if [ "$MAIN_LINES" -gt 80 ]; then
  echo "ERROR: $MAIN_PATH has $MAIN_LINES lines (limit: 80)"
  echo "  main.rs must contain only CLI dispatch — no logic."
  exit 1
fi

echo "OK: module structure valid, main.rs: $MAIN_LINES lines"
exit 0
