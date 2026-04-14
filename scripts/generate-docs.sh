#!/usr/bin/env bash
set -euo pipefail

# generate-docs.sh — Spawn a claude -p process per feature to write Diátaxis documentation.
# After each feature, commit and push so progress is saved incrementally.
# Uses the product CLI for feature listing and context bundle assembly.

REPO_ROOT="$(cd "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
DOCS_OUT="$REPO_ROOT/docs/guide"
PRODUCT="$REPO_ROOT/target/release/product"

# Build release binary once
echo "==> Building product (release)..."
cargo build --release --manifest-path "$REPO_ROOT/Cargo.toml" 2>&1

# Create output directory
mkdir -p "$DOCS_OUT"

# Extract feature list as JSON, then parse IDs and titles
FEATURE_JSON=$($PRODUCT feature list --format json 2>/dev/null)
TOTAL=$(echo "$FEATURE_JSON" | jq length)

DIATAXIS_PROMPT=$(cat <<'PROMPT_EOF'
You are a technical writer creating documentation for the Product CLI tool.
Product is a Rust CLI and MCP server that manages a file-based knowledge graph of features, ADRs, and test criteria.

Write documentation for the feature described in the context below, following the Diátaxis framework strictly.
Write the output as a single Markdown file to the path specified below. Use these sections (include only sections that apply):

## Overview
One paragraph explaining what this capability is and why it exists.

## Tutorial
Step-by-step walkthrough for a newcomer. Use concrete examples with actual CLI commands.
Assume the reader has Product installed and a repository already initialized.

## How-to Guide
Task-oriented recipes for common operations. Each recipe should have a heading and numbered steps.
Focus on "how do I accomplish X" — not explanation.

## Reference
Exact CLI syntax, flags, output formats, configuration keys, and edge cases.
Use code blocks for commands and tables for flag/option listings.

## Explanation
Deeper discussion of design decisions, trade-offs, and how this feature relates to the rest of the system.
Link to relevant ADRs by ID where appropriate.

Rules:
- Be precise and concrete. Use actual command names, flag names, and file paths from the context.
- Do NOT invent commands or flags that are not present in the context.
- Keep it concise — aim for 150-400 lines total.
- Use GitHub-flavored Markdown.
- Do not include a title heading — the build system adds that.
- Do NOT include YAML front-matter (no --- block at the top). Start the file directly with ## Overview.
- Do NOT output any commentary, explanations, or status messages — only write the documentation file.
PROMPT_EOF
)

for i in $(seq 0 $((TOTAL - 1))); do
  FT_ID=$(echo "$FEATURE_JSON" | jq -r ".[$i].id")
  FT_TITLE=$(echo "$FEATURE_JSON" | jq -r ".[$i].title")

  # Derive slug from the feature file on disk
  FT_FILE=$(ls "$REPO_ROOT/docs/features/${FT_ID}-"*.md 2>/dev/null | head -1)
  if [ -z "$FT_FILE" ]; then
    echo "[$((i+1))/$TOTAL] SKIP $FT_ID — no feature file found"
    continue
  fi
  FT_SLUG=$(basename "$FT_FILE" .md)
  OUT_FILE="$DOCS_OUT/${FT_SLUG}.md"

  # Skip if file already exists with substantial content (>20 lines)
  if [ -f "$OUT_FILE" ] && [ "$(wc -l < "$OUT_FILE")" -ge 20 ]; then
    echo "[$((i+1))/$TOTAL] SKIP $FT_ID — $OUT_FILE already has $(wc -l < "$OUT_FILE") lines"
    continue
  fi

  echo ""
  echo "============================================================"
  echo "[$((i+1))/$TOTAL] Generating docs for $FT_ID — $FT_TITLE"
  echo "============================================================"

  # Assemble context bundle via the product CLI
  CONTEXT=$($PRODUCT context "$FT_ID" --depth 2 2>/dev/null || echo "No context available for $FT_ID")

  # Spawn claude -p with context piped via stdin
  printf '%s\n\nWrite the file to: %s\n\nHere is the feature context (from the knowledge graph context bundle):\n---BEGIN CONTEXT---\n%s\n---END CONTEXT---\n\nWrite the documentation file now for feature %s — %s.' \
    "$DIATAXIS_PROMPT" "$OUT_FILE" "$CONTEXT" "$FT_ID" "$FT_TITLE" \
    | claude -p --allowedTools "Write,Read,Edit"

  # Verify the file was written with real content
  if [ ! -s "$OUT_FILE" ] || [ "$(wc -l < "$OUT_FILE")" -lt 20 ]; then
    echo "    WARNING: $OUT_FILE looks too short or empty — skipping commit"
    continue
  fi

  echo "    Written: $OUT_FILE ($(wc -l < "$OUT_FILE") lines)"

  # Commit and push
  cd "$REPO_ROOT"
  git add "$OUT_FILE"
  git commit -m "$(cat <<EOF
docs: add Diátaxis guide for $FT_ID — $FT_TITLE

Co-Authored-By: Claude Opus 4.6 (1M context) <noreply@anthropic.com>
EOF
)"
  git push
  echo "    Committed and pushed: $FT_ID"
done

echo ""
echo "============================================================"
echo "Done — $TOTAL feature docs generated in $DOCS_OUT/"
echo "============================================================"
