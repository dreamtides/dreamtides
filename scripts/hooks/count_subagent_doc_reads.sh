#!/usr/bin/env bash
# Claude Code hook: counts docs/ reads from subagent transcripts
# Triggered by SubagentStop; receives JSON on stdin with agent_transcript_path

set -euo pipefail

INPUT=$(cat)
TRANSCRIPT=$(echo "$INPUT" | python3 -c "import sys,json; print(json.load(sys.stdin).get('agent_transcript_path',''))")

if [ -z "$TRANSCRIPT" ] || [ ! -f "$TRANSCRIPT" ]; then
  exit 0
fi

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="${CLAUDE_PROJECT_DIR:-"$(cd "$SCRIPT_DIR/../.." && pwd)"}"
COUNTS_FILE="$PROJECT_DIR/.claude/doc-read-counts.json"

mkdir -p "$(dirname "$COUNTS_FILE")"

# Extract docs/ file paths from Read tool calls in the transcript, then update counts
python3 -c "
import json, os

transcript = '$TRANSCRIPT'
project_dir = '$PROJECT_DIR'
counts_file = '$COUNTS_FILE'
docs_dir = project_dir + '/docs/'

def extract_read_paths(obj):
    \"\"\"Recursively find Read tool calls and extract file_path from input.\"\"\"
    paths = []
    if isinstance(obj, dict):
        # Match content items with name=Read and input.file_path
        if obj.get('name') == 'Read' and isinstance(obj.get('input'), dict):
            fp = obj['input'].get('file_path', '')
            if fp.startswith(docs_dir):
                paths.append(fp[len(project_dir)+1:])
        for v in obj.values():
            paths.extend(extract_read_paths(v))
    elif isinstance(obj, list):
        for item in obj:
            paths.extend(extract_read_paths(item))
    return paths

doc_paths = []
with open(transcript, 'r') as f:
    for line in f:
        try:
            entry = json.loads(line)
        except json.JSONDecodeError:
            continue
        doc_paths.extend(extract_read_paths(entry))

if not doc_paths:
    exit()

counts = {}
if os.path.exists(counts_file):
    with open(counts_file, 'r') as f:
        try:
            counts = json.load(f)
        except json.JSONDecodeError:
            counts = {}

for rel_path in doc_paths:
    counts[rel_path] = counts.get(rel_path, 0) + 1

with open(counts_file, 'w') as f:
    json.dump(counts, f, indent=2, sort_keys=True)
"
