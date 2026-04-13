#!/usr/bin/env bash
set -euo pipefail

# implement-all.sh — Iterate through all features and implement them headlessly.
# Uses `product feature next` to pick the next ready feature and
# `product implement --headless` to run each one without human interaction.
# Commits and pushes after each successful feature.

BRANCH=${BRANCH:-main}
MAX_FAILURES=${MAX_FAILURES:-3}
MAX_ITERATIONS=${MAX_ITERATIONS:-100}

failures=0
iteration=0

while true; do
    iteration=$((iteration + 1))

    if [ "$iteration" -gt "$MAX_ITERATIONS" ]; then
        echo "Reached MAX_ITERATIONS ($MAX_ITERATIONS). Stopping."
        exit 1
    fi

    # Ask the knowledge graph what to implement next
    next_output=$(product feature next 2>&1) || true

    # Stop if nothing left
    if echo "$next_output" | grep -qi "all features are complete"; then
        echo "All features are complete."
        exit 0
    fi

    if echo "$next_output" | grep -qi "incomplete dependencies"; then
        echo "Remaining features have incomplete dependencies. Stopping."
        echo "  $next_output"
        exit 1
    fi

    # Extract feature ID (e.g. "FT-008" from "FT-008 — Schema Migration (phase 2, planned)")
    feature_id=$(echo "$next_output" | grep -oE 'FT-[0-9]+' | head -1)

    if [ -z "$feature_id" ]; then
        echo "Could not parse feature ID from: $next_output"
        exit 1
    fi

    # Extract title for the commit message (text between " — " and " (")
    feature_title=$(echo "$next_output" | sed 's/^[^ ]* — //;s/ (.*$//')

    echo "============================================================"
    echo "  Iteration $iteration: implementing $feature_id — $feature_title"
    echo "  $(date)"
    echo "============================================================"

    if product implement "$feature_id" --headless; then
        echo "$feature_id completed successfully."
        failures=0

        # Commit and push
        if [ -n "$(git status --porcelain)" ]; then
            git add -A
            git commit -m "Implement $feature_id: $feature_title

Automated headless implementation via product implement --headless."
            echo "Committed $feature_id."

            git push origin "$BRANCH"
            echo "Pushed $feature_id to origin/$BRANCH."
        else
            echo "No changes to commit for $feature_id."
        fi
    else
        failures=$((failures + 1))
        echo "WARNING: $feature_id failed (consecutive failures: $failures/$MAX_FAILURES)"

        # Discard partial work so the next iteration starts clean
        git checkout -- . 2>/dev/null || true
        git clean -fd 2>/dev/null || true

        if [ "$failures" -ge "$MAX_FAILURES" ]; then
            echo "Hit $MAX_FAILURES consecutive failures. Stopping."
            exit 1
        fi

        echo "Skipping to next feature..."
    fi

    echo ""
done
