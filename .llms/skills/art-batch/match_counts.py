#!/usr/bin/env python3
"""Shared utility: count rendered-text assignments across art-assigned.toml
and /tmp/art-batch-results/*.toml using proper TOML parsing."""

import re
import tomllib
from pathlib import Path

REPO_ROOT = Path(__file__).parent.parent.parent.parent
ASSIGNED = REPO_ROOT / "rules_engine" / "tabula" / "art-assigned.toml"
RESULTS_DIR = Path("/tmp/art-batch-results")


def normalize_text(text: str) -> str:
    """Normalize rendered-text so TOML multiline strings (with \\n) match
    the anonymized pool format (which uses double spaces for line breaks)."""
    return re.sub(r"\s*\n+\s*", "  ", text.strip())


def _count_from_toml(path: Path, counts: dict[str, int]) -> None:
    """Parse a TOML file and increment counts for each rendered-text entry."""
    with open(path, "rb") as f:
        data = tomllib.load(f)
    for card in data.get("cards", []):
        rt = card.get("rendered-text", "").strip()
        if rt:
            key = normalize_text(rt)
            counts[key] = counts.get(key, 0) + 1


def get_match_counts() -> dict[str, int]:
    """Return rendered-text -> total assignment count across all sources."""
    counts: dict[str, int] = {}
    if ASSIGNED.exists():
        _count_from_toml(ASSIGNED, counts)
    if RESULTS_DIR.exists():
        for f in sorted(RESULTS_DIR.iterdir()):
            if f.suffix == ".toml":
                _count_from_toml(f, counts)
    return counts
