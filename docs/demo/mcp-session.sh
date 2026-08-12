#!/usr/bin/env bash
# docs/demo/mcp-session.sh <tool> <json-args>
# Warm the language hosts, THEN make one tools/call — both in one `ddd serve`
# process, because the host dies with the process. Needed for the Rust path.
set -euo pipefail
DDD="${DDD_BIN:-./target/release/ddd}"
WAIT="${DDD_WAIT_MS:-180000}"
{ jq -cn --argjson w "$WAIT" \
    '{jsonrpc:"2.0",id:1,method:"tools/call",params:{name:"ddd_warmup",arguments:{wait_ms:$w}}}'
  jq -cn --arg n "$1" --argjson a "$2" \
    '{jsonrpc:"2.0",id:2,method:"tools/call",params:{name:$n,arguments:$a}}'
} | "$DDD" serve 2>/dev/null \
  | jq -r 'select(.id==2) | .result.content[0].text // .error.message // .'
