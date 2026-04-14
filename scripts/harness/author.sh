#!/usr/bin/env bash
# Example authoring harness. Copy and modify for your workflow.
# Loads the appropriate system prompt and starts Product MCP.
set -euo pipefail

SESSION_TYPE=${1:?Usage: author.sh feature|adr|review}
PROMPTS_DIR=${PRODUCT_PROMPTS_DIR:-"$(product config get paths.prompts)"}
PROMPT_FILE="$PROMPTS_DIR/author-${SESSION_TYPE}-v1.md"

if [ ! -f "$PROMPT_FILE" ]; then
  echo "Prompt file not found: $PROMPT_FILE"
  echo "Run: product prompts init"
  exit 1
fi

echo "System prompt: $PROMPT_FILE"
echo "Product MCP: stdio (Claude Code will connect automatically)"
echo ""
echo "Open Claude Code in this directory. The .mcp.json will load Product MCP."
echo "Paste the contents of $PROMPT_FILE as your first message or system prompt."
echo ""
echo "When complete, run: product graph check && product gap check --changed"
