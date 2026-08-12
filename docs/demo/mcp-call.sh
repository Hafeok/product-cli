#!/usr/bin/env bash
# docs/demo/mcp-call.sh <tool> <json-args>
# One JSON-RPC tools/call against `ddd serve` over stdio. Hostless artifact
# classes only (html-css) — a Rust call needs a warm host, see mcp-session.sh.
set -euo pipefail
DDD="${DDD_BIN:-./target/release/ddd}"
jq -cn --arg n "$1" --argjson a "$2" \
  '{jsonrpc:"2.0",id:1,method:"tools/call",params:{name:$n,arguments:$a}}' \
| "$DDD" serve 2>/dev/null \
| jq -r '.result.content[0].text // .error.message // .'
