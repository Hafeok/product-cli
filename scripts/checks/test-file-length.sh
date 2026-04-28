#!/usr/bin/env bash
# scripts/checks/test-file-length.sh
# Checks Rust integration-test file lengths under tests/.
# Tests are inherently verbose (setup + assertions), so the hard limit is
# higher than for src/. Anything over the hard limit signals a topic file
# that should be split further.
# Exit 0: all files within limits
# Exit 1: one or more files exceed hard limit
# Exit 2: one or more files exceed warning threshold, none exceed hard limit
set -euo pipefail

HARD_LIMIT=${TEST_FILE_LENGTH_HARD:-1600}
WARN_LIMIT=${TEST_FILE_LENGTH_WARN:-800}

HARD_VIOLATIONS=$(find tests -name "*.rs" \
  | xargs wc -l \
  | awk -v limit="$HARD_LIMIT" '$1 > limit && $2 != "total" {print $1, $2}' \
  | sort -rn) || true

WARN_VIOLATIONS=$(find tests -name "*.rs" \
  | xargs wc -l \
  | awk -v wl="$WARN_LIMIT" -v hl="$HARD_LIMIT" \
    '$1 > wl && $1 <= hl && $2 != "total" {print $1, $2}' \
  | sort -rn) || true

if [ -n "$HARD_VIOLATIONS" ]; then
  echo "ERROR: test files exceeding hard limit ($HARD_LIMIT lines):"
  echo "$HARD_VIOLATIONS" | while read -r count file; do
    echo "  $file: $count lines (limit: $HARD_LIMIT)"
  done
  exit 1
fi

if [ -n "$WARN_VIOLATIONS" ]; then
  echo "WARNING: test files approaching limit ($WARN_LIMIT–$HARD_LIMIT lines):"
  echo "$WARN_VIOLATIONS" | while read -r count file; do
    echo "  $file: $count lines (warn at: $WARN_LIMIT)"
  done
  exit 2
fi

echo "OK: all test files within limits"
exit 0
