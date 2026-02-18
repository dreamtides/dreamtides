#!/usr/bin/env bash
# Claude Code hook: counts reads of files under docs/
# Receives JSON on stdin with tool_input.file_path

set -euo pipefail

INPUT=$(cat)
FILE_PATH=$(echo "$INPUT" | python3 -c "import sys,json; print(json.load(sys.stdin).get('tool_input',{}).get('file_path',''))")

if [ -z "$FILE_PATH" ]; then
  exit 0
fi

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="${CLAUDE_PROJECT_DIR:-"$(cd "$SCRIPT_DIR/../.." && pwd)"}"
DOCS_DIR="$PROJECT_DIR/docs"
COUNTS_FILE="$PROJECT_DIR/.claude/doc-read-counts.json"

# Check if the file is under docs/
case "$FILE_PATH" in
  "$DOCS_DIR"/*)
    ;;
  *)
    exit 0
    ;;
esac

# Get relative path from project root
REL_PATH="${FILE_PATH#"$PROJECT_DIR"/}"

mkdir -p "$(dirname "$COUNTS_FILE")"

# Atomically update the counts file using Python for safe JSON handling
python3 -c "
import json, os, sys, fcntl

counts_file = '$COUNTS_FILE'
rel_path = '$REL_PATH'

# Read existing counts
counts = {}
if os.path.exists(counts_file):
    with open(counts_file, 'r') as f:
        try:
            counts = json.load(f)
        except json.JSONDecodeError:
            counts = {}

counts[rel_path] = counts.get(rel_path, 0) + 1

with open(counts_file, 'w') as f:
    json.dump(counts, f, indent=2, sort_keys=True)
"
