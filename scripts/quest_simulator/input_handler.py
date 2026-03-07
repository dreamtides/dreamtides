"""Raw terminal input handling for arrow-key navigation.

Provides four input modes for site interactions: single select, multi select,
confirm/decline, and continue. Uses stdlib tty/termios for raw mode with
robust terminal state restoration.

When AI mode is enabled, decision points use a file-based turn protocol
instead of terminal I/O: a JSON prompt file is written, then the handler
polls for a JSON response file before continuing.
"""

import atexit
import io
import json
import os
import queue
import re
import select
import signal
import sys
import time
from pathlib import Path
from typing import Any, Callable, Optional

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
_saved_termios: Optional[list[Any]] = None

# --- AI mode state ---
_ai_mode: bool = False
_prompt_counter: int = 0
_PROMPT_PATH = Path(".logs/quest_ai_prompt.json")
_RESPONSE_PATH = Path(".logs/quest_ai_response.json")
_AI_POLL_INTERVAL = 0.3
_AI_TIMEOUT = 300  # 5 minutes


def set_ai_mode(enabled: bool) -> None:
    """Enable or disable AI turn protocol mode."""
    global _ai_mode
    _ai_mode = enabled


def get_ai_mode() -> bool:
    """Return whether AI mode is active."""
    return _ai_mode


# --- Web mode state ---
_web_mode: bool = False
_web_prompt_queue: Optional[queue.Queue] = None
_web_response_queue: Optional[queue.Queue] = None
_web_state_callback: Optional[Callable[[], dict]] = None


def set_web_mode(
    prompt_queue: queue.Queue,
    response_queue: queue.Queue,
    state_callback: Callable[[], dict],
) -> None:
    """Enable web mode with the given queues and state callback."""
    global _web_mode, _web_prompt_queue, _web_response_queue, _web_state_callback
    _web_mode = True
    _web_prompt_queue = prompt_queue
    _web_response_queue = response_queue
    _web_state_callback = state_callback


def _web_send_prompt(
    prompt_type: str,
    options: list[str],
    max_selections: Optional[int] = None,
) -> Any:
    """Send a web prompt and block until the browser responds."""
    context = _output_capture.flush_buffer() if _output_capture else ""
    state = _web_state_callback() if _web_state_callback else {}
    prompt = {
        "type": prompt_type,
        "options": options,
        "context": context,
        "max_selections": max_selections,
        "state": state,
    }
    pq = _web_prompt_queue
    rq = _web_response_queue
    assert pq is not None
    assert rq is not None
    pq.put(prompt)
    return rq.get()


class _OutputCapture(io.TextIOWrapper):
    """Wraps stdout to capture plain text while passing writes through."""

    def __init__(self, real_stdout: Any) -> None:
        self._real_stdout = real_stdout
        self._buffer: list[str] = []
        self._ansi_re = re.compile(r"\x1b\[[0-9;]*[A-Za-z]|\x1b\].*?\x07")

    def write(self, s: str) -> int:
        self._buffer.append(self._ansi_re.sub("", s))
        return self._real_stdout.write(s)

    def flush(self) -> None:
        self._real_stdout.flush()

    def flush_buffer(self) -> str:
        """Retrieve and clear all captured plain text."""
        text = "".join(self._buffer)
        self._buffer.clear()
        return text

    def isatty(self) -> bool:
        return False

    def fileno(self) -> int:
        return self._real_stdout.fileno()

    @property
    def encoding(self) -> str:
        return self._real_stdout.encoding

    @property
    def errors(self) -> Optional[str]:
        return self._real_stdout.errors


_output_capture: Optional[_OutputCapture] = None


def install_output_capture() -> None:
    """Install the output capture wrapper on sys.stdout."""
    global _output_capture
    _output_capture = _OutputCapture(sys.stdout)
    sys.stdout = _output_capture  # type: ignore[assignment]


def _write_ai_prompt(
    prompt_type: str,
    options: list[str],
    max_selections: Optional[int] = None,
) -> None:
    """Write a JSON prompt file for the AI sub-agent."""
    global _prompt_counter
    _prompt_counter += 1
    context = _output_capture.flush_buffer() if _output_capture else ""
    prompt = {
        "type": prompt_type,
        "prompt_id": _prompt_counter,
        "context": context,
        "options": options,
        "max_selections": max_selections,
    }
    _PROMPT_PATH.parent.mkdir(parents=True, exist_ok=True)
    tmp_path = _PROMPT_PATH.with_suffix(".tmp")
    tmp_path.write_text(json.dumps(prompt, indent=2))
    tmp_path.rename(_PROMPT_PATH)


def _read_ai_response() -> Any:
    """Poll for and read the AI response file, then clean up."""
    deadline = time.monotonic() + _AI_TIMEOUT
    while time.monotonic() < deadline:
        if _RESPONSE_PATH.exists():
            try:
                data = json.loads(_RESPONSE_PATH.read_text())
            except (json.JSONDecodeError, OSError):
                time.sleep(_AI_POLL_INTERVAL)
                continue
            if data.get("prompt_id") != _prompt_counter:
                time.sleep(_AI_POLL_INTERVAL)
                continue
            # Clean up files
            try:
                _RESPONSE_PATH.unlink()
            except OSError:
                pass
            try:
                _PROMPT_PATH.unlink()
            except OSError:
                pass
            return data["choice"]
        time.sleep(_AI_POLL_INTERVAL)
    raise TimeoutError("AI response timed out after 5 minutes")


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
    saved = _saved_termios
    if _HAS_TERMIOS and saved is not None:
        try:
            termios.tcsetattr(sys.stdin.fileno(), termios.TCSADRAIN, saved)
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
    print("  Quest abandoned.")
    sys.exit(130)


def _sigtstp_handler(signum: int, frame: object) -> None:
    """Handle Ctrl+Z by restoring terminal before suspending.

    Restores terminal state so the shell is usable while suspended,
    then sends SIGSTOP to actually suspend the process.
    """
    _restore_termios()
    signal.signal(signal.SIGTSTP, signal.SIG_DFL)
    os.kill(os.getpid(), signal.SIGSTOP)


def ensure_terminal_restored() -> None:
    """Public API to force terminal state restoration.

    Called by external modules (e.g. quest_sim.py) as a safety net
    in exception handlers and cleanup paths.
    """
    _release_termios()


# Register atexit fallback for terminal restoration
atexit.register(_release_termios)


def _read_key() -> str:
    """Read a single keypress and return a semantic key name.

    Uses os.read() on the raw file descriptor to bypass Python's buffered
    TextIOWrapper. This is critical because select() checks the fd level,
    and Python's buffer can consume all bytes of an escape sequence at once,
    causing select() to report no data available for the remaining bytes.
    """
    fd = sys.stdin.fileno()
    raw = os.read(fd, 1)
    if not raw:
        return KEY_QUIT

    ch = raw[0]

    if ch == 0x03:  # Ctrl+C
        return KEY_QUIT

    if ch in (0x0D, 0x0A):  # \r, \n
        return KEY_ENTER

    if ch == 0x20:  # space
        return KEY_SPACE

    if ch == 0x1B:  # Escape
        # Check if more bytes are available (arrow sequence)
        ready, _, _ = select.select([fd], [], [], 0.05)
        if ready:
            seq = os.read(fd, 1)
            if seq == b"[":
                code = os.read(fd, 1)
                if code == b"A":
                    return KEY_UP
                elif code == b"B":
                    return KEY_DOWN
                elif code == b"C":
                    return KEY_RIGHT
                elif code == b"D":
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
    Counts actual terminal lines (including newlines within render_fn output).
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
    Counts actual terminal lines (including newlines within render_fn output).
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

    if _web_mode:
        choice = _web_send_prompt("single_select", options)
        return max(0, min(int(choice), len(options) - 1))

    if _ai_mode:
        _write_ai_prompt("single_select", options)
        choice = _read_ai_response()
        return max(0, min(int(choice), len(options) - 1))

    if not _is_interactive():
        return min(initial, len(options) - 1)

    cursor = min(initial, len(options) - 1)
    line_count = _render_single_menu(options, cursor, render_fn)

    _save_termios()
    prev_int = signal.getsignal(signal.SIGINT)
    prev_tstp = signal.getsignal(signal.SIGTSTP)
    signal.signal(signal.SIGINT, _sigint_handler)
    signal.signal(signal.SIGTSTP, _sigtstp_handler)
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
        signal.signal(signal.SIGINT, prev_int)
        signal.signal(signal.SIGTSTP, prev_tstp)

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

    if _web_mode:
        choice = _web_send_prompt(
            "multi_select", options, max_selections=max_selections
        )
        indices = [int(i) for i in choice if 0 <= int(i) < len(options)]
        if max_selections is not None:
            indices = indices[:max_selections]
        return sorted(set(indices))

    if _ai_mode:
        _write_ai_prompt("multi_select", options, max_selections=max_selections)
        choice = _read_ai_response()
        indices = [int(i) for i in choice if 0 <= int(i) < len(options)]
        if max_selections is not None:
            indices = indices[:max_selections]
        return sorted(set(indices))

    if not _is_interactive():
        if initial_toggled:
            return sorted(i for i in initial_toggled if 0 <= i < len(options))
        return []

    cursor = 0
    toggled: set[int] = set(initial_toggled) if initial_toggled else set()
    line_count = _render_multi_menu(options, cursor, toggled, render_fn)

    _save_termios()
    prev_int = signal.getsignal(signal.SIGINT)
    prev_tstp = signal.getsignal(signal.SIGTSTP)
    signal.signal(signal.SIGINT, _sigint_handler)
    signal.signal(signal.SIGTSTP, _sigtstp_handler)
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
            line_count = _render_multi_menu(options, cursor, toggled, render_fn)
            tty.setraw(sys.stdin.fileno())
    finally:
        _release_termios()
        signal.signal(signal.SIGINT, prev_int)
        signal.signal(signal.SIGTSTP, prev_tstp)

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
    if _web_mode:
        return bool(_web_send_prompt("confirm_decline", [accept_label, decline_label]))

    if _ai_mode:
        _write_ai_prompt("confirm_decline", [accept_label, decline_label])
        return bool(_read_ai_response())

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
    prev_int = signal.getsignal(signal.SIGINT)
    prev_tstp = signal.getsignal(signal.SIGTSTP)
    signal.signal(signal.SIGINT, _sigint_handler)
    signal.signal(signal.SIGTSTP, _sigtstp_handler)
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
        signal.signal(signal.SIGINT, prev_int)
        signal.signal(signal.SIGTSTP, prev_tstp)

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

    if _web_mode:
        _web_send_prompt("wait_for_continue", [])
        return

    if _ai_mode:
        return

    if not _is_interactive():
        return

    _save_termios()
    prev_int = signal.getsignal(signal.SIGINT)
    prev_tstp = signal.getsignal(signal.SIGTSTP)
    signal.signal(signal.SIGINT, _sigint_handler)
    signal.signal(signal.SIGTSTP, _sigtstp_handler)
    try:
        tty.setraw(sys.stdin.fileno())
        key = _read_key()
        if key == KEY_QUIT:
            _release_termios()
            print()
            sys.exit(130)
    finally:
        _release_termios()
        signal.signal(signal.SIGINT, prev_int)
        signal.signal(signal.SIGTSTP, prev_tstp)
