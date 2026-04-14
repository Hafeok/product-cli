#!/usr/bin/env bash
# scripts/checks/function-length.sh
# Uses awk to find fn definitions, then counts statement lines until closing brace.
# Exit 0: all functions within limits
# Exit 1: one or more functions exceed hard limit
# Exit 2: one or more functions exceed warning threshold, none exceed hard limit
set -euo pipefail

HARD_LIMIT=${FN_LENGTH_HARD:-40}
WARN_LIMIT=${FN_LENGTH_WARN:-30}

RESULTS_FILE=$(mktemp /tmp/fn-length-results.XXXXXX)
trap "rm -f $RESULTS_FILE" EXIT

find src -name "*.rs" | while read -r file; do
  awk -v hard="$HARD_LIMIT" -v warn="$WARN_LIMIT" -v fname="$file" '
    /^[[:space:]]*(pub |pub\(.*\) |async |pub async |pub(crate) )*fn / {
      fn_name = $0
      fn_line = NR
      brace_depth = 0
      stmt_count = 0
      in_fn = 1
    }
    in_fn {
      # Count opening and closing braces
      n = split($0, chars, "")
      for (i = 1; i <= n; i++) {
        if (chars[i] == "{") brace_depth++
        if (chars[i] == "}") brace_depth--
      }
      # Count non-blank, non-brace-only lines as statements
      stripped = $0
      gsub(/^[[:space:]]+/, "", stripped)
      gsub(/[[:space:]]+$/, "", stripped)
      if (length(stripped) > 0 && stripped != "{" && stripped != "}") {
        stmt_count++
      }
      if (brace_depth == 0 && fn_line != NR) {
        if (stmt_count > hard) {
          print "ERROR: " fname ":" fn_line ": function has " stmt_count \
                " statement lines (limit: " hard ")"
        } else if (stmt_count > warn) {
          print "WARN: " fname ":" fn_line ": function has " stmt_count \
                " statement lines (warn at: " warn ")"
        }
        in_fn = 0
        stmt_count = 0
      }
    }
  ' "$file"
done > "$RESULTS_FILE" 2>&1

if grep -q "^ERROR:" "$RESULTS_FILE" 2>/dev/null; then
  cat "$RESULTS_FILE"
  exit 1
elif grep -q "^WARN:" "$RESULTS_FILE" 2>/dev/null; then
  cat "$RESULTS_FILE"
  exit 2
fi
echo "OK: all functions within limits"
exit 0
