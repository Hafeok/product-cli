#!/usr/bin/env bash
# Example implementation harness. Copy and modify for your workflow.
# Product is a knowledge tool — this script is not part of Product.
set -euo pipefail

FEATURE=${1:?Usage: implement.sh FT-XXX}

echo "=== Pre-flight ==="
product preflight "$FEATURE" || {
  echo "Pre-flight failed. Run: product preflight $FEATURE"
  exit 1
}

echo "=== Gap check ==="
product gap check "$FEATURE" --severity high --format json | tee /tmp/gaps.json
if jq -e '.findings | length > 0' /tmp/gaps.json > /dev/null; then
  echo "High-severity gaps found. Resolve before implementing."
  exit 1
fi

echo "=== Drift check ==="
product drift check --phase "$(product feature show "$FEATURE" --field phase)"
# Drift is advisory — continue regardless

echo "=== Context bundle ==="
BUNDLE_FILE=$(mktemp /tmp/product-context-XXXX.md)
product context "$FEATURE" --depth 2 --measure > "$BUNDLE_FILE"
echo "Bundle written to: $BUNDLE_FILE"

echo "=== Agent invocation ==="
# Replace this with your agent of choice:
#   claude --print --context-file "$BUNDLE_FILE"
#   cursor --context "$BUNDLE_FILE"
#   cat "$BUNDLE_FILE" | your-agent
echo "Pass $BUNDLE_FILE to your agent, then run:"
echo "  product verify $FEATURE"
