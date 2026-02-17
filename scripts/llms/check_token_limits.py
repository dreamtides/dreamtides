#!/usr/bin/env python3
"""Validate that key documentation files stay within token budgets.

Reuses the tiktoken venv created by count_tokens.py. Exits non-zero if
any file exceeds its limit.

Rules:
  - AGENTS.md (repo root): max 1000 tokens
  - docs/**/*.md: max 10000 tokens each
"""

import glob
import os
import subprocess
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent.parent
VENV_DIR = REPO_ROOT / "scripts" / "llms" / ".venv"

AGENTS_TOKEN_LIMIT = 1000
DOCS_TOKEN_LIMIT = 10000


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
    os.execv(venv_python, [venv_python, __file__] + sys.argv[1:])


def main() -> int:
    ensure_venv()
    reexec_in_venv()

    import tiktoken

    enc = tiktoken.get_encoding("cl100k_base")
    failures: list[str] = []

    agents_path = REPO_ROOT / "AGENTS.md"
    if agents_path.exists():
        tokens = len(enc.encode(agents_path.read_text()))
        if tokens > AGENTS_TOKEN_LIMIT:
            failures.append(f"AGENTS.md: {tokens} tokens (limit {AGENTS_TOKEN_LIMIT})")

    for md_path in sorted(glob.glob("docs/**/*.md", root_dir=REPO_ROOT, recursive=True)):
        full_path = REPO_ROOT / md_path
        tokens = len(enc.encode(full_path.read_text()))
        if tokens > DOCS_TOKEN_LIMIT:
            failures.append(f"{md_path}: {tokens} tokens (limit {DOCS_TOKEN_LIMIT})")

    if failures:
        print("Token limit violations:")
        for failure in failures:
            print(f"  {failure}")
        return 1

    print("Token limits OK")
    return 0


if __name__ == "__main__":
    sys.exit(main())
