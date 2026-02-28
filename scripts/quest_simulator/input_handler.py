"""Raw terminal input handling for arrow-key navigation.

Provides four input modes for site interactions: single select, multi select,
confirm/decline, and continue. Uses stdlib tty/termios for raw mode with
robust terminal state restoration.
"""

import atexit
import select
import signal
import sys
from typing import Callable, Optional

# Terminal control is Unix-only; guard imports for type checking on all platforms
try:
    import termios
    import tty

    _HAS_TERMIOS = True
except ImportError:
    _HAS_TERMIOS = False

# Semantic key constants
KEY_UP = "up"
KEY_DOWN = "down"
KEY_LEFT = "left"
KEY_RIGHT = "right"
KEY_ENTER = "enter"
KEY_SPACE = "space"
KEY_QUIT = "quit"
KEY_UNKNOWN = "unknown"

# Saved terminal state for restoration
_saved_termios: Optional[list[object]] = None


def _is_interactive() -> bool:
    """Check if stdout is a TTY and termios is available."""
    return _HAS_TERMIOS and sys.stdout.isatty() and sys.stdin.isatty()


def _save_termios() -> None:
    """Save current terminal state if not already saved."""
    global _saved_termios
    if _HAS_TERMIOS and _saved_termios is None:
        try:
            _saved_termios = termios.tcgetattr(sys.stdin.fileno())
        except termios.error:
            _saved_termios = None


def _restore_termios() -> None:
    """Restore saved terminal state without clearing it.

    Keeps _saved_termios so that re-entering raw mode and restoring again
    still works correctly in menu redraw loops.
    """
    if _HAS_TERMIOS and _saved_termios is not None:
        try:
            termios.tcsetattr(
                sys.stdin.fileno(), termios.TCSADRAIN, _saved_termios
            )
        except termios.error:
            pass


def _release_termios() -> None:
    """Restore saved terminal state and clear it.

    Called at the end of an interaction to fully release the saved state.
    """
    global _saved_termios
    _restore_termios()
    _saved_termios = None


def _sigint_handler(signum: int, frame: object) -> None:
    """Handle Ctrl+C by restoring terminal state before exiting."""
    _release_termios()
    print()
    sys.exit(130)


# Register atexit fallback for terminal restoration
atexit.register(_release_termios)


def _read_key() -> str:
    """Read a single keypress and return a semantic key name.

    Detects arrow escape sequences using select() with a 50ms timeout
    to distinguish bare Escape from the start of an arrow sequence.
    """
    ch = sys.stdin.read(1)
    if not ch:
        return KEY_QUIT

    if ch == "\x03":
        return KEY_QUIT

    if ch in ("\r", "\n"):
        return KEY_ENTER

    if ch == " ":
        return KEY_SPACE

    if ch == "\x1b":
        # Check if more bytes are available (arrow sequence)
        ready, _, _ = select.select([sys.stdin], [], [], 0.05)
        if ready:
            seq = sys.stdin.read(1)
            if seq == "[":
                code = sys.stdin.read(1)
                if code == "A":
                    return KEY_UP
                elif code == "B":
                    return KEY_DOWN
                elif code == "C":
                    return KEY_RIGHT
                elif code == "D":
                    return KEY_LEFT
        return KEY_QUIT

    return KEY_UNKNOWN


def _clear_menu(line_count: int) -> None:
    """Move cursor up N lines and clear to end of screen."""
    if line_count > 0:
        sys.stdout.write(f"\x1b[{line_count}A")
    sys.stdout.write("\x1b[J")
    sys.stdout.flush()


def _render_single_menu(
    options: list[str],
    cursor: int,
    render_fn: Optional[Callable[[int, str, bool], str]],
) -> int:
    """Render a single-select menu and return the number of lines printed.

    If render_fn is provided, it is called as render_fn(index, option, selected)
    and its return value is printed. Otherwise a default format is used.
    """
    line_count = 0
    for i, option in enumerate(options):
        is_selected = i == cursor
        if render_fn is not None:
            line = render_fn(i, option, is_selected)
        else:
            marker = ">" if is_selected else " "
            line = f"  {marker} {option}"
        print(line)
        line_count += line.count("\n") + 1
    return line_count


def _render_multi_menu(
    options: list[str],
    cursor: int,
    toggled: set[int],
    render_fn: Optional[Callable[[int, str, bool, bool], str]],
) -> int:
    """Render a multi-select menu and return the number of lines printed.

    If render_fn is provided, it is called as
    render_fn(index, option, is_highlighted, is_checked) and its return value
    is printed. Otherwise a default format is used.
    """
    line_count = 0
    for i, option in enumerate(options):
        is_highlighted = i == cursor
        is_checked = i in toggled
        if render_fn is not None:
            line = render_fn(i, option, is_highlighted, is_checked)
        else:
            marker = ">" if is_highlighted else " "
            check = "[x]" if is_checked else "[ ]"
            line = f"  {marker} {check} {option}"
        print(line)
        line_count += line.count("\n") + 1
    return line_count


def single_select(
    options: list[str],
    render_fn: Optional[Callable[[int, str, bool], str]] = None,
    initial: int = 0,
) -> int:
    """Display a single-select menu and return the chosen index.

    Arrow up/down moves the cursor. Enter confirms the selection.

    Args:
        options: List of option strings to display.
        render_fn: Optional callback(index, option, is_selected) -> str
            for custom rendering of each line.
        initial: Initial cursor position.

    Returns:
        The selected index (0-based).
    """
    if not options:
        return 0

    if not _is_interactive():
        return min(initial, len(options) - 1)

    cursor = min(initial, len(options) - 1)
    line_count = _render_single_menu(options, cursor, render_fn)

    _save_termios()
    prev_handler = signal.getsignal(signal.SIGINT)
    signal.signal(signal.SIGINT, _sigint_handler)
    try:
        tty.setraw(sys.stdin.fileno())
        while True:
            key = _read_key()
            if key == KEY_ENTER:
                break
            elif key == KEY_UP:
                cursor = (cursor - 1) % len(options)
            elif key == KEY_DOWN:
                cursor = (cursor + 1) % len(options)
            elif key == KEY_QUIT:
                _release_termios()
                print()
                sys.exit(130)
            else:
                continue

            _restore_termios()
            _clear_menu(line_count)
            line_count = _render_single_menu(options, cursor, render_fn)
            tty.setraw(sys.stdin.fileno())
    finally:
        _release_termios()
        signal.signal(signal.SIGINT, prev_handler)

    return cursor


def multi_select(
    options: list[str],
    render_fn: Optional[Callable[[int, str, bool, bool], str]] = None,
    max_selections: Optional[int] = None,
    initial_toggled: Optional[set[int]] = None,
) -> list[int]:
    """Display a multi-select menu and return the list of toggled indices.

    Arrow up/down moves the cursor. Space toggles the current item.
    Enter confirms all toggled items.

    Args:
        options: List of option strings to display.
        render_fn: Optional callback(index, option, is_highlighted, is_checked)
            -> str for custom rendering.
        max_selections: Maximum number of items that can be selected.
            If None, no limit.
        initial_toggled: Set of initially toggled indices.

    Returns:
        Sorted list of selected indices.
    """
    if not options:
        return []

    if not _is_interactive():
        if initial_toggled:
            return sorted(i for i in initial_toggled if 0 <= i < len(options))
        return []

    cursor = 0
    toggled: set[int] = set(initial_toggled) if initial_toggled else set()
    line_count = _render_multi_menu(options, cursor, toggled, render_fn)

    _save_termios()
    prev_handler = signal.getsignal(signal.SIGINT)
    signal.signal(signal.SIGINT, _sigint_handler)
    try:
        tty.setraw(sys.stdin.fileno())
        while True:
            key = _read_key()
            if key == KEY_ENTER:
                break
            elif key == KEY_UP:
                cursor = (cursor - 1) % len(options)
            elif key == KEY_DOWN:
                cursor = (cursor + 1) % len(options)
            elif key == KEY_SPACE:
                if cursor in toggled:
                    toggled.discard(cursor)
                elif max_selections is None or len(toggled) < max_selections:
                    toggled.add(cursor)
            elif key == KEY_QUIT:
                _release_termios()
                print()
                sys.exit(130)
            else:
                continue

            _restore_termios()
            _clear_menu(line_count)
            line_count = _render_multi_menu(
                options, cursor, toggled, render_fn
            )
            tty.setraw(sys.stdin.fileno())
    finally:
        _release_termios()
        signal.signal(signal.SIGINT, prev_handler)

    return sorted(toggled)


def confirm_decline(
    accept_label: str = "Accept",
    decline_label: str = "Decline",
    render_fn: Optional[Callable[[str, str, bool], str]] = None,
) -> bool:
    """Display a confirm/decline prompt and return the choice.

    Left/right arrows switch between the two options. Enter confirms.

    Args:
        accept_label: Label for the accept option.
        decline_label: Label for the decline option.
        render_fn: Optional callback(accept_label, decline_label, accepted)
            -> str for custom rendering. If None, a default format is used.

    Returns:
        True if accepted, False if declined.
    """
    if not _is_interactive():
        return True

    selected = 0  # 0 = accept, 1 = decline
    labels = [accept_label, decline_label]

    def _print_prompt() -> int:
        if render_fn is not None:
            line = render_fn(accept_label, decline_label, selected == 0)
        else:
            parts: list[str] = []
            for i, label in enumerate(labels):
                if i == selected:
                    parts.append(f"[> {label} <]")
                else:
                    parts.append(f"[  {label}  ]")
            line = "  " + "    ".join(parts)
        print(line)
        return 1

    line_count = _print_prompt()

    _save_termios()
    prev_handler = signal.getsignal(signal.SIGINT)
    signal.signal(signal.SIGINT, _sigint_handler)
    try:
        tty.setraw(sys.stdin.fileno())
        while True:
            key = _read_key()
            if key == KEY_ENTER:
                break
            elif key == KEY_LEFT:
                selected = 0
            elif key == KEY_RIGHT:
                selected = 1
            elif key == KEY_QUIT:
                _release_termios()
                print()
                sys.exit(130)
            else:
                continue

            _restore_termios()
            _clear_menu(line_count)
            line_count = _print_prompt()
            tty.setraw(sys.stdin.fileno())
    finally:
        _release_termios()
        signal.signal(signal.SIGINT, prev_handler)

    return selected == 0


def wait_for_continue(
    prompt: str = "  Press any key to continue...",
    render_fn: Optional[Callable[[], str]] = None,
) -> None:
    """Display a prompt and wait for any keypress.

    Used after information display to pause before proceeding.

    Args:
        prompt: Default prompt string if render_fn is not provided.
        render_fn: Optional callback() -> str for custom rendering of the
            prompt line.
    """
    if render_fn is not None:
        print(render_fn())
    else:
        print(prompt)

    if not _is_interactive():
        return

    _save_termios()
    prev_handler = signal.getsignal(signal.SIGINT)
    signal.signal(signal.SIGINT, _sigint_handler)
    try:
        tty.setraw(sys.stdin.fileno())
        key = _read_key()
        if key == KEY_QUIT:
            _release_termios()
            print()
            sys.exit(130)
    finally:
        _release_termios()
        signal.signal(signal.SIGINT, prev_handler)
