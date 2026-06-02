#!/usr/bin/env bash
# scripts/checks/file-length.sh
# Checks Rust source file lengths.
# Exit 0: all files within limits
# Exit 1: one or more files exceed hard limit (400 lines)
# Exit 2: one or more files exceed warning threshold (300 lines), none exceed hard limit
set -euo pipefail

HARD_LIMIT=${FILE_LENGTH_HARD:-400}
WARN_LIMIT=${FILE_LENGTH_WARN:-300}

# Discover which src/ trees to walk: workspace members after FT-107, with a
# `src/` fallback so the script also works inside tempdirs created by the
# `code_quality_tests` suite (which fake a single-crate layout).
DIRS=()
for d in product-core/src product-mcp/src product-cli/src; do
  [ -d "$d" ] && DIRS+=("$d")
done
if [ ${#DIRS[@]} -eq 0 ] && [ -d src ]; then
  DIRS=(src)
fi
if [ ${#DIRS[@]} -eq 0 ]; then
  echo "OK: no source directories to scan"
  exit 0
fi

HARD_VIOLATIONS=$(find "${DIRS[@]}" -name "*.rs" \
  | xargs wc -l \
  | awk -v limit="$HARD_LIMIT" '$1 > limit && $2 != "total" {print $1, $2}' \
  | sort -rn) || true

WARN_VIOLATIONS=$(find "${DIRS[@]}" -name "*.rs" \
  | xargs wc -l \
  | awk -v wl="$WARN_LIMIT" -v hl="$HARD_LIMIT" \
    '$1 > wl && $1 <= hl && $2 != "total" {print $1, $2}' \
  | sort -rn) || true

if [ -n "$HARD_VIOLATIONS" ]; then
  echo "ERROR: files exceeding hard limit ($HARD_LIMIT lines):"
  echo "$HARD_VIOLATIONS" | while read -r count file; do
    echo "  $file: $count lines (limit: $HARD_LIMIT)"
  done
  exit 1
fi

if [ -n "$WARN_VIOLATIONS" ]; then
  echo "WARNING: files approaching limit ($WARN_LIMIT–$HARD_LIMIT lines):"
  echo "$WARN_VIOLATIONS" | while read -r count file; do
    echo "  $file: $count lines (warn at: $WARN_LIMIT)"
  done
  exit 2
fi

echo "OK: all source files within limits"
exit 0
