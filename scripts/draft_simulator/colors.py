"""Terminal color support using AYU Mirage palette.

Provides 24-bit ANSI true color output with automatic TTY detection
and NO_COLOR convention support. Stdlib-only, no external dependencies.
"""

import os
import sys

_USE_COLOR = sys.stdout.isatty() and os.environ.get("NO_COLOR") is None

PALETTE: dict[str, tuple[int, int, int]] = {
    "accent": (0xFF, 0xCC, 0x66),
    "entity": (0x73, 0xD0, 0xFF),
    "keyword": (0xFF, 0xAD, 0x66),
    "constant": (0xDF, 0xBF, 0xFF),
    "tag": (0x5C, 0xCF, 0xE6),
    "comment": (0x6C, 0x7A, 0x8B),
    "vcs_added": (0x87, 0xD9, 0x6C),
    "error": (0xFF, 0x66, 0x66),
    "string": (0xD5, 0xFF, 0x80),
    "ui": (0x70, 0x7A, 0x8C),
    "warning": (0xFF, 0xA7, 0x59),
    "special": (0xFF, 0xDF, 0xB3),
    "func": (0xFF, 0xD1, 0x73),
    "operator": (0xF2, 0x9E, 0x74),
    "markup": (0xF2, 0x87, 0x79),
}


def c(text: object, role: str, bold: bool = False) -> str:
    """Wrap text in ANSI escape for the given palette role."""
    s = str(text)
    if not _USE_COLOR:
        return s
    r, g, b = PALETTE[role]
    bold_prefix = "\033[1m" if bold else ""
    return f"{bold_prefix}\033[38;2;{r};{g};{b}m{s}\033[0m"


def header(text: object) -> str:
    """Bold accent — titles."""
    return c(text, "accent", bold=True)


def section(text: object) -> str:
    """Bold entity — section headings."""
    return c(text, "entity", bold=True)


def label(text: object) -> str:
    """Keyword — labels/keys."""
    return c(text, "keyword")


def num(text: object) -> str:
    """Constant — numeric values."""
    return c(text, "constant")


def ok(text: object) -> str:
    """Green — PASS status."""
    return c(text, "vcs_added")


def fail(text: object) -> str:
    """Bold red — FAIL status."""
    return c(text, "error", bold=True)


def warn(text: object) -> str:
    """Warning orange — WARN status."""
    return c(text, "warning", bold=True)


def filepath(text: object) -> str:
    """String green — file paths."""
    return c(text, "string")


def dim(text: object) -> str:
    """Comment grey — secondary text."""
    return c(text, "comment")


def card(text: object) -> str:
    """Func gold — card names."""
    return c(text, "func")


def format_progress_bar(
    done: int,
    total: int,
    width: int = 40,
    use_color: bool | None = None,
    label: str = "runs complete",
) -> str:
    """Return a formatted progress bar string."""
    color = _USE_COLOR if use_color is None else use_color
    filled = int(width * done / total) if total > 0 else width
    fill = "=" * filled
    empty = " " * (width - filled)
    if color:
        r, g, b = PALETTE["accent"]
        dr, dg, db = PALETTE["comment"]
        fill_s = f"\033[38;2;{r};{g};{b}m{fill}\033[0m"
        empty_s = f"\033[38;2;{dr};{dg};{db}m{empty}\033[0m"
    else:
        fill_s = fill
        empty_s = empty
    return f"\r[{fill_s}{empty_s}] {done}/{total} {label}"
