#!/usr/bin/env python3
"""Estimate token consumption of CLAUDE.md and related agent instruction files.

Automatically creates a venv and installs tiktoken on first run. Uses the
cl100k_base encoding for accurate token counts.
"""

import os
import subprocess
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent.parent
VENV_DIR = REPO_ROOT / "scripts" / "llms" / ".venv"

# Files to measure, in display order.
TARGET_FILES = [
    "AGENTS.md",
    "AGENTS_OPUS.md",
    "AGENTS_GPT.md",
]


def ensure_venv() -> None:
    """Create venv and install tiktoken if not already present."""
    venv_python = VENV_DIR / "bin" / "python3"
    if venv_python.exists():
        return
    print("Setting up venv and installing tiktoken...")
    subprocess.check_call(
        [sys.executable, "-m", "venv", str(VENV_DIR)],
        stdout=subprocess.DEVNULL,
    )
    subprocess.check_call(
        [str(venv_python), "-m", "pip", "install", "-q", "tiktoken"],
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    )


def reexec_in_venv() -> None:
    """Re-execute this script using the venv python if not already in it."""
    venv_python = str(VENV_DIR / "bin" / "python3")
    if sys.executable == venv_python or os.environ.get("_IN_VENV"):
        return
    os.environ["_IN_VENV"] = "1"
    os.execv(venv_python, [venv_python, __file__])


def main() -> None:
    ensure_venv()
    reexec_in_venv()

    import tiktoken

    enc = tiktoken.get_encoding("cl100k_base")
    results: list[tuple[str, int, int]] = []

    for name in TARGET_FILES:
        path = REPO_ROOT / name
        if not path.exists():
            continue
        text = path.resolve().read_text()
        results.append((name, text.count("\n"), len(enc.encode(text))))

    if not results:
        print("No agent instruction files found.", file=sys.stderr)
        sys.exit(1)

    claude_md = REPO_ROOT / "CLAUDE.md"
    symlink_note = ""
    if claude_md.is_symlink():
        symlink_note = f"  (CLAUDE.md -> {claude_md.resolve().name})"

    name_width = max(len(r[0]) for r in results)
    print(f"  {'File':<{name_width}}  {'Lines':>5}  {'Tokens':>6}")
    print(f"  {'-' * name_width}  {'-----':>5}  {'------':>6}")
    for name, lines, tokens in results:
        print(f"  {name:<{name_width}}  {lines:>5}  {tokens:>6}")
    if symlink_note:
        print()
        print(symlink_note)


if __name__ == "__main__":
    main()
