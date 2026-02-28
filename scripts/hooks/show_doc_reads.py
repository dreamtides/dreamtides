#!/usr/bin/env python3
"""Print docs/ files sorted by Claude Code read count."""

import json
import os
import sys


def main():
    project_dir = os.path.dirname(
        os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    )
    counts_file = os.path.join(project_dir, ".claude", "doc-read-counts.json")

    if not os.path.exists(counts_file):
        print("No doc reads recorded yet.")
        return

    with open(counts_file, "r") as f:
        try:
            counts = json.load(f)
        except json.JSONDecodeError:
            print("No doc reads recorded yet.")
            return

    if not counts:
        print("No doc reads recorded yet.")
        return

    sorted_docs = sorted(counts.items(), key=lambda x: x[1], reverse=True)
    max_path_len = max(len(path) for path, _ in sorted_docs)

    print(f"{'Document':<{max_path_len}}  Reads")
    print(f"{'─' * max_path_len}  ─────")
    for path, count in sorted_docs:
        print(f"{path:<{max_path_len}}  {count:>5}")

    print(f"\nTotal: {sum(counts.values())} reads across {len(counts)} documents")


if __name__ == "__main__":
    main()
