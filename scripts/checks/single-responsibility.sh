#!/usr/bin/env bash
# scripts/checks/single-responsibility.sh
# Checks that every source file (except mod.rs, main.rs) begins with a
# single-responsibility //! doc comment that does not contain " and ".
# Exit 0: all files pass
# Exit 1: one or more files fail
set -euo pipefail

RESULTS_FILE=$(mktemp /tmp/sr-check-results.XXXXXX)
trap "rm -f $RESULTS_FILE" EXIT

FOUND_VIOLATION=0

while IFS= read -r file; do
  FIRST_LINE=$(head -1 "$file")
  if [[ ! "$FIRST_LINE" =~ ^//! ]]; then
    echo "ERROR: $file: missing single-responsibility doc comment (first line must be //! ...)" >> "$RESULTS_FILE"
    FOUND_VIOLATION=1
  elif [[ "$FIRST_LINE" =~ " and " ]]; then
    echo "ERROR: $file: responsibility doc comment contains 'and' — split this file" >> "$RESULTS_FILE"
    echo "  Found: $FIRST_LINE" >> "$RESULTS_FILE"
    FOUND_VIOLATION=1
  fi
done < <(find src -name "*.rs" ! -name "mod.rs" ! -name "main.rs" | sort)

if [ "$FOUND_VIOLATION" -eq 1 ]; then
  cat "$RESULTS_FILE"
  exit 1
fi

echo "OK: all files have single-responsibility doc comments"
exit 0
