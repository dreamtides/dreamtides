#!/bin/bash
# Enforce that Claude runs just fmt, just review, and commits before stopping.
# Exit 0 = allow stop, Exit 2 = block stop.
#
# Uses a marker file (.validation_marker) touched by mark_validation.sh
# whenever just fmt or just review runs. Blocks only if the marker is
# missing or older than 60 seconds AND there are uncommitted changes.

INPUT=$(cat)
STOP_HOOK_ACTIVE=$(echo "$INPUT" | jq -r '.stop_hook_active // false')

# If the hook already fired once, allow Claude to stop (prevent infinite loop)
if [ "$STOP_HOOK_ACTIVE" = "true" ]; then
  exit 0
fi

cd "$CLAUDE_PROJECT_DIR" || exit 0

# No uncommitted changes to tracked files — nothing to do
if git diff --quiet && git diff --cached --quiet; then
  exit 0
fi

MARKER="$CLAUDE_PROJECT_DIR/.validation_marker"

# If marker exists and is recent (< 60 seconds old), allow stop
if [ -f "$MARKER" ]; then
  NOW=$(date +%s)
  MARKER_TIME=$(stat -f %m "$MARKER" 2>/dev/null || stat -c %Y "$MARKER" 2>/dev/null || echo 0)
  AGE=$(( NOW - MARKER_TIME ))
  if [ "$AGE" -lt 60 ]; then
    exit 0
  fi
fi

# Marker is missing or stale — block Claude from stopping
cat >&2 <<'EOF'
You have uncommitted changes and have not recently run validation. Before stopping, please:

1. Run 'just fmt' to format code & docs
2. Run 'just review' to validate changes
3. Commit your work with a detailed git commit message
EOF
exit 2
