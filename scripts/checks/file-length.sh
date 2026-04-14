#!/usr/bin/env bash
# scripts/checks/file-length.sh
# Checks Rust source file lengths.
# Exit 0: all files within limits
# Exit 1: one or more files exceed hard limit (400 lines)
# Exit 2: one or more files exceed warning threshold (300 lines), none exceed hard limit
set -euo pipefail

HARD_LIMIT=${FILE_LENGTH_HARD:-400}
WARN_LIMIT=${FILE_LENGTH_WARN:-300}

HARD_VIOLATIONS=$(find src -name "*.rs" \
  | xargs wc -l \
  | awk -v limit="$HARD_LIMIT" '$1 > limit && $2 != "total" {print $1, $2}' \
  | sort -rn) || true

WARN_VIOLATIONS=$(find src -name "*.rs" \
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
