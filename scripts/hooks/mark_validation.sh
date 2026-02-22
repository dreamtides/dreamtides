#!/bin/bash
# PostToolUse hook for Bash: touch a marker file when just fmt or just review runs.

INPUT=$(cat)
COMMAND=$(echo "$INPUT" | jq -r '.tool_input.command // ""')

if echo "$COMMAND" | grep -qE '(just fmt|just review)'; then
  touch "$CLAUDE_PROJECT_DIR/.validation_marker"
fi

exit 0
