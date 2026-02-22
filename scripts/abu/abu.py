#!/usr/bin/env python3
"""Control the Unity Editor and interact with a running game.

Provides subcommands for Unity Editor operations (refresh, play, test, cycle)
via Hammerspoon and for in-game interaction (snapshot, click, hover, drag,
screenshot) over TCP using the abu NDJSON protocol.

Editor command prerequisites:
  1. Hammerspoon.app installed and running
  2. ~/.hammerspoon/init.lua contains: require("hs.ipc")
  3. hs CLI installed (Hammerspoon > Preferences > Install CLI tool)
  4. Hammerspoon granted Accessibility in System Settings > Privacy & Security
"""

import argparse
import base64
import json
import os
import re
import shutil
import socket
import subprocess
import sys
import tempfile
import time
import uuid
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any

# Save builtins before shadowing with domain-specific subclasses.
builtins_ConnectionRefusedError = ConnectionRefusedError
builtins_TimeoutError = TimeoutError

UNITY_BUNDLE_ID = "com.unity3d.UnityEditor5.x"
EDITOR_LOG = Path.home() / "Library" / "Logs" / "Unity" / "Editor.log"
TIMEOUT_SECONDS = 120
TEST_TIMEOUT_SECONDS = 300
POLL_INTERVAL = 0.3
ABU_PORT = 9999
ABU_STATE_FILE = Path(__file__).resolve().parent.parent.parent / ".abu-state.json"


class AbuError(Exception):
    """Raised when an abu command fails."""


class ConnectionError(AbuError):
    """Raised when the CLI cannot connect to Unity."""


class TimeoutError(AbuError):
    """Raised when waiting for a Unity response times out."""


class EmptyResponseError(AbuError):
    """Raised when Unity closes the connection without a response."""


class HammerspoonError(AbuError):
    """Raised when Hammerspoon CLI interaction fails."""


class UnityNotFoundError(AbuError):
    """Raised when Unity Editor is not running."""


class RefreshTimeoutError(AbuError):
    """Raised when waiting for refresh completion exceeds the timeout."""


class CompilationError(AbuError):
    """Raised when Unity C# compilation produces errors."""


@dataclass(frozen=True)
class RefreshResult:
    """Result of waiting for a Unity asset refresh."""

    finished: bool
    success: bool
    errors: list[str] = field(default_factory=list)
    summary: str = ""


@dataclass(frozen=True)
class TestResult:
    """Result of running Unity Edit Mode tests."""

    finished: bool
    success: bool
    passed: int = 0
    failed: int = 0
    skipped: int = 0
    total: int = 0
    failures: list[str] = field(default_factory=list)
    summary: str = ""


def strip_ref(ref: str) -> str:
    """Strip a leading '@' from a ref argument."""
    if ref.startswith("@"):
        return ref[1:]
    return ref


def build_params(args: argparse.Namespace) -> dict[str, Any]:
    """Build the params dict for the wire command from parsed arguments."""
    command: str = args.command

    if command == "snapshot":
        params: dict[str, Any] = {}
        if args.compact:
            params["compact"] = True
        if args.interactive:
            params["interactive"] = True
        if args.max_depth is not None:
            params["maxDepth"] = args.max_depth
        return params

    if command == "click":
        return {"ref": strip_ref(args.ref)}

    if command == "hover":
        return {"ref": strip_ref(args.ref)}

    if command == "drag":
        params = {"source": strip_ref(args.source)}
        if args.target is not None:
            params["target"] = strip_ref(args.target)
        return params

    # screenshot
    return {}


def build_command(command: str, params: dict[str, Any]) -> str:
    """Build an NDJSON command line to send to Unity."""
    message = {
        "id": str(uuid.uuid4()),
        "command": command,
        "params": params,
    }
    return json.dumps(message) + "\n"


def handle_response(
    command: str, response: dict[str, Any], *, json_output: bool = False
) -> str:
    """Process a response from Unity and return the output string.

    By default, returns the formatted ARIA tree for commands that include a
    snapshot (snapshot, click, hover, drag). Pass json_output=True to get the
    raw JSON data instead. For screenshot, decodes base64 data and writes to a
    temp file, returning the file path. Raises AbuError on error responses.
    """
    if not response.get("success", False):
        raise AbuError(response.get("error", "Unknown error"))

    data = response.get("data", {})

    if command == "screenshot":
        b64_data: str = data.get("base64", "")
        png_bytes = base64.b64decode(b64_data)
        tmp_dir = tempfile.mkdtemp(prefix="abu-screenshot-")
        file_path = os.path.join(tmp_dir, "screenshot.png")
        with open(file_path, "wb") as f:
            f.write(png_bytes)
        return file_path

    if json_output:
        return json.dumps(data)

    parts: list[str] = []
    history = data.get("history")
    if history and isinstance(history, list) and len(history) > 0:
        parts.append("--- History ---")
        for entry in history:
            parts.append(entry)
        parts.append("---")

    snapshot = data.get("snapshot")
    if snapshot is not None:
        parts.append(snapshot)
        return "\n".join(parts)

    if parts:
        return "\n".join(parts)

    return json.dumps(data)


def send_command(command: str, params: dict[str, Any], port: int) -> dict[str, Any]:
    """Connect to Unity, send one command, and read one response.

    Raises ConnectionError if the connection is refused, TimeoutError if the
    socket read times out, or EmptyResponseError if Unity closes the
    connection without sending a response.
    """
    message = build_command(command, params)
    sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)

    try:
        sock.settimeout(30.0)
        sock.connect(("localhost", port))
    except (builtins_ConnectionRefusedError, OSError):
        sock.close()
        raise ConnectionError(
            f"Could not connect to Unity on localhost:{port}. Is the game running?"
        )

    try:
        sock.sendall(message.encode("utf-8"))
        buf = b""
        while True:
            try:
                chunk = sock.recv(4096)
            except builtins_TimeoutError:
                raise TimeoutError("Timed out waiting for response from Unity")
            if not chunk:
                break
            buf += chunk
            if b"\n" in buf:
                break

        if not buf.strip():
            raise EmptyResponseError(
                "Connection closed without response from Unity"
            )

        line = buf.split(b"\n", 1)[0]
        return json.loads(line.decode("utf-8"))
    finally:
        sock.close()


def send_command_with_wait(
    command: str, params: dict[str, Any], port: int, wait_timeout: int
) -> dict[str, Any]:
    """Retry send_command with exponential backoff until connected.

    Retries on ConnectionError and EmptyResponseError up to wait_timeout
    seconds. Raises the last error if the timeout is exceeded.
    """
    deadline = time.monotonic() + wait_timeout
    delay = 1.0
    max_delay = 5.0
    last_error: AbuError | None = None

    while time.monotonic() < deadline:
        try:
            return send_command(command, params, port)
        except (ConnectionError, EmptyResponseError) as e:
            last_error = e
            remaining = deadline - time.monotonic()
            if remaining <= 0:
                break
            sleep_time = min(delay, remaining, max_delay)
            print(
                f"Waiting for Unity (retrying in {sleep_time:.0f}s)...",
                file=sys.stderr,
            )
            time.sleep(sleep_time)
            delay = min(delay * 1.5, max_delay)

    raise last_error or ConnectionError(
        f"Could not connect to Unity on localhost:{port} within {wait_timeout}s"
    )


def run_hs(lua_code: str) -> str:
    """Execute Lua code via the Hammerspoon CLI and return stdout.

    Filters out extension loading lines. Raises HammerspoonError on failure.
    """
    try:
        result = subprocess.run(
            ["hs", "-c", lua_code],
            capture_output=True,
            text=True,
            timeout=10,
        )
    except FileNotFoundError:
        raise HammerspoonError(
            "'hs' CLI not found on PATH. "
            "Install via: Hammerspoon > Preferences > Install CLI tool"
        )
    except subprocess.TimeoutExpired:
        raise HammerspoonError("Hammerspoon CLI timed out")

    if result.returncode != 0:
        raise HammerspoonError(f"hs failed: {result.stderr.strip()}")

    lines = result.stdout.strip().splitlines()
    return "\n".join(
        line for line in lines if not line.startswith("-- Loading extension:")
    )


def ensure_hammerspoon() -> None:
    """Check that Hammerspoon is running, launch if needed."""
    result = subprocess.run(["pgrep", "-x", "Hammerspoon"], capture_output=True)
    if result.returncode != 0:
        print("Hammerspoon is not running. Launching...")
        subprocess.run(["open", "-a", "Hammerspoon"])
        time.sleep(3)
        print("Hammerspoon launched.")


def is_worktree() -> bool:
    """Check if the current directory is inside a git worktree."""
    try:
        result = subprocess.run(
            ["git", "rev-parse", "--git-common-dir"],
            capture_output=True, text=True, check=True,
        )
        return result.stdout.strip() != ".git"
    except subprocess.CalledProcessError:
        return False


def send_menu_item(path: list[str]) -> str:
    """Send a selectMenuItem command to Unity via Hammerspoon."""
    lua_path = ", ".join(f'"{item}"' for item in path)
    menu_label = " > ".join(path)
    lua = f"""
    local app = hs.application.find("{UNITY_BUNDLE_ID}")
    if not app then
        return "ERROR: Unity Editor not found"
    end
    local result = app:selectMenuItem({{{lua_path}}})
    if result then
        return "OK: Selected {menu_label} (pid " .. app:pid() .. ")"
    else
        return "ERROR: {menu_label} menu item not found"
    end
    """
    output = run_hs(lua)
    if output.startswith("ERROR"):
        if "not found" in output and "Unity" in output:
            raise UnityNotFoundError(output)
        raise HammerspoonError(output)
    return output


def send_refresh() -> str:
    """Trigger Assets > Refresh in Unity."""
    return send_menu_item(["Assets", "Refresh"])


def send_run_tests() -> str:
    """Trigger Tools > Run All Tests in Unity."""
    return send_menu_item(["Tools", "Run All Tests"])


def toggle_play_mode() -> str:
    """Toggle Edit > Play Mode > Play in Unity."""
    return send_menu_item(["Edit", "Play Mode", "Play"])


def get_log_size() -> int:
    """Return the current size of the Unity Editor log."""
    try:
        return EDITOR_LOG.stat().st_size
    except OSError:
        return 0


def read_new_log(offset: int) -> str:
    """Read new content from the Unity Editor log starting at offset."""
    try:
        with open(EDITOR_LOG, "r", errors="replace") as f:
            f.seek(offset)
            return f.read()
    except OSError:
        return ""


def _report_result(content: str) -> RefreshResult:
    """Parse log content to build a RefreshResult."""
    seen: set[str] = set()
    errors: list[str] = []
    for line in content.splitlines():
        stripped = line.strip()
        if "error CS" in stripped and stripped not in seen:
            seen.add(stripped)
            errors.append(stripped)

    if errors:
        return RefreshResult(
            finished=True,
            success=False,
            errors=errors,
            summary=f"{len(errors)} compilation error(s)",
        )

    summary = ""
    for line in reversed(content.splitlines()):
        if "Asset Pipeline Refresh" in line:
            summary = line.strip()
            break

    return RefreshResult(finished=True, success=True, errors=[], summary=summary)


def wait_for_refresh(log_offset: int) -> RefreshResult:
    """Poll the Editor log for refresh completion.

    Watches for script compilation requests, asset pipeline refresh markers,
    and build success/failure indicators. Returns a RefreshResult describing
    the outcome.
    """
    start = time.time()
    seen_initial_refresh = False
    needs_compilation = False

    while time.time() - start < TIMEOUT_SECONDS:
        content = read_new_log(log_offset)

        if "[ScriptCompilation] Requested" in content:
            needs_compilation = True

        if "RefreshV2(NoUpdateAssetOptions)" in content:
            seen_initial_refresh = True

        if needs_compilation:
            if "StopAssetImportingV2" in content:
                return _report_result(content)
            if "Tundra build failed" in content:
                return _report_result(content)
        elif seen_initial_refresh:
            time.sleep(1.0)
            content = read_new_log(log_offset)
            if "[ScriptCompilation] Requested" in content:
                needs_compilation = True
                continue
            return _report_result(content)

        time.sleep(POLL_INTERVAL)

    return RefreshResult(
        finished=False,
        success=False,
        errors=[],
        summary=f"Timed out after {TIMEOUT_SECONDS}s",
    )


def wait_for_tests(log_offset: int) -> TestResult:
    """Poll the Editor log for test run completion.

    Watches for [TestRunner] markers logged by RunAllTestsCommand.cs.
    Returns a TestResult describing the outcome.
    """
    start = time.time()

    while time.time() - start < TEST_TIMEOUT_SECONDS:
        content = read_new_log(log_offset)

        for line in content.splitlines():
            if "An unexpected error happened while running tests" in line:
                return TestResult(
                    finished=True,
                    success=False,
                    failures=["An unexpected error happened while running tests"],
                    summary="Test runner encountered an unexpected error",
                )

            if "[TestRunner] Run finished:" in line:
                failures = []
                for log_line in content.splitlines():
                    if "[TestRunner] FAIL:" in log_line:
                        idx = log_line.find("[TestRunner] FAIL:")
                        if idx >= 0:
                            failures.append(log_line[idx:].strip())

                match = re.search(
                    r"(\d+) passed, (\d+) failed, (\d+) skipped "
                    r"\(total: (\d+)\)",
                    line,
                )
                if match:
                    passed = int(match.group(1))
                    failed = int(match.group(2))
                    skipped = int(match.group(3))
                    total = int(match.group(4))
                    return TestResult(
                        finished=True,
                        success=failed == 0,
                        passed=passed,
                        failed=failed,
                        skipped=skipped,
                        total=total,
                        failures=failures,
                        summary=f"{passed} passed, {failed} failed, "
                        f"{skipped} skipped (total: {total})",
                    )

        time.sleep(POLL_INTERVAL)

    return TestResult(
        finished=False,
        success=False,
        summary=f"Timed out after {TEST_TIMEOUT_SECONDS}s",
    )


def is_play_mode_active() -> bool:
    """Check if Unity is in play mode by probing the Abu TCP port."""
    sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    try:
        sock.settimeout(1.0)
        sock.connect(("localhost", ABU_PORT))
        sock.close()
        return True
    except (builtins_ConnectionRefusedError, OSError):
        sock.close()
        return False


def do_refresh(play: bool = False) -> None:
    """Execute a refresh and optionally enter play mode."""
    log_offset = get_log_size()
    result_msg = send_refresh()
    print(result_msg)

    print("Waiting for asset refresh to complete...")
    result = wait_for_refresh(log_offset)

    if not result.finished:
        print(f"Error: {result.summary}", file=sys.stderr)
        sys.exit(1)

    if result.success:
        if result.summary:
            print(f"  {result.summary}")
        print("Asset refresh finished.")
    else:
        print("\nCompilation errors:")
        for err in result.errors:
            print(f"  {err}")
        print("Asset refresh finished with errors.")
        sys.exit(1)

    if play:
        play_result = toggle_play_mode()
        print(play_result)


def do_test() -> None:
    """Refresh, then run all Edit Mode tests and report results."""
    if is_play_mode_active():
        print("Play mode is active, exiting before running tests...")
        result_msg = toggle_play_mode()
        print(result_msg)
        time.sleep(2)

    do_refresh()

    print("\nTriggering test run...")
    log_offset = get_log_size()
    result_msg = send_run_tests()
    print(result_msg)

    print("Waiting for tests to complete...")
    result = wait_for_tests(log_offset)

    if not result.finished:
        print(f"Error: {result.summary}", file=sys.stderr)
        sys.exit(1)

    print(f"\nTest results: {result.summary}")

    if result.success:
        print("All tests passed.")
    else:
        print("\nFailures:")
        for failure in result.failures:
            print(f"  {failure}")
        sys.exit(1)


def do_cycle() -> None:
    """Exit play mode if active, refresh, then re-enter play mode."""
    if is_play_mode_active():
        print("Play mode is active, exiting...")
        result_msg = toggle_play_mode()
        print(result_msg)
        time.sleep(2)
    else:
        print("Play mode is not active.")

    do_refresh()

    print("\nEntering play mode...")
    play_result = toggle_play_mode()
    print(play_result)


def read_state_file() -> dict[str, Any] | None:
    """Read and parse the abu state file, returning None if unavailable."""
    try:
        return json.loads(ABU_STATE_FILE.read_text())
    except (OSError, json.JSONDecodeError):
        return None


def is_pid_alive(pid: int) -> bool:
    """Check whether a process with the given PID is running."""
    try:
        os.kill(pid, 0)
        return True
    except (OSError, ProcessLookupError):
        return False


def do_status() -> None:
    """Print a combined status report from state file, PID, and TCP probe."""
    state = read_state_file()
    tcp_up = is_play_mode_active()

    print("Unity Editor Status")
    print("=" * 40)

    if state is None:
        print("  State file:    not found")
        print("  Unity PID:     unknown")
        print("  Play mode:     unknown")
        print("  Game mode:     unknown")
    else:
        pid = state.get("unityPid", 0)
        alive = is_pid_alive(pid) if pid else False
        play_state = state.get("playModeState", "unknown")
        game_mode = state.get("gameMode", "unknown")
        timestamp = state.get("timestampUtc", "unknown")

        if alive:
            print("  State file:    ok")
            print(f"  Unity PID:     {pid} (running)")
        else:
            print("  State file:    stale")
            print(f"  Unity PID:     {pid} (not running)")

        in_play = play_state in ("EnteredPlayMode", "ExitingPlayMode")
        print(f"  Play mode:     {'active' if in_play else 'inactive'}")
        print(f"  Game mode:     {game_mode}")
        print(f"  Last updated:  {timestamp}")

    print(f"  TCP (:{ABU_PORT}):   {'reachable' if tcp_up else 'unreachable'}")


def build_parser() -> argparse.ArgumentParser:
    """Build the argument parser with all subcommands."""
    parser = argparse.ArgumentParser(
        prog="abu.py",
        description="Control Unity Editor and interact with a running game.",
    )
    parser.add_argument(
        "--json", action="store_true", help="Output raw JSON instead of formatted text"
    )
    parser.add_argument(
        "--wait",
        type=int,
        default=None,
        metavar="SECONDS",
        help="Retry connection for up to SECONDS before giving up",
    )
    subparsers = parser.add_subparsers(dest="command", required=True)

    # snapshot
    snapshot_parser = subparsers.add_parser("snapshot", help="Take a UI snapshot")
    snapshot_parser.add_argument(
        "--compact", action="store_true", help="Omit non-interactive unlabeled nodes"
    )
    snapshot_parser.add_argument(
        "--interactive", action="store_true", help="Show only interactive elements"
    )
    snapshot_parser.add_argument(
        "--max-depth", type=int, default=None, help="Maximum tree depth"
    )

    # click
    click_parser = subparsers.add_parser("click", help="Click a UI element")
    click_parser.add_argument("ref", help="Element ref (e.g. e1 or @e1)")

    # hover
    hover_parser = subparsers.add_parser("hover", help="Hover over a UI element")
    hover_parser.add_argument("ref", help="Element ref (e.g. e1 or @e1)")

    # drag
    drag_parser = subparsers.add_parser("drag", help="Drag from source to target")
    drag_parser.add_argument("source", help="Source element ref")
    drag_parser.add_argument("target", nargs="?", default=None, help="Target element ref")

    # screenshot
    subparsers.add_parser("screenshot", help="Capture a screenshot")

    # refresh
    refresh_parser = subparsers.add_parser(
        "refresh", help="Trigger asset refresh and wait for completion"
    )
    refresh_parser.add_argument(
        "--play", action="store_true",
        help="Enter play mode after successful refresh",
    )

    # play
    subparsers.add_parser("play", help="Toggle play mode")

    # test
    subparsers.add_parser(
        "test", help="Refresh then run all Edit Mode tests"
    )

    # cycle
    subparsers.add_parser(
        "cycle", help="Exit play mode (if active), refresh, re-enter play mode"
    )

    # status
    subparsers.add_parser(
        "status", help="Show Unity Editor state from abu state file and TCP probe"
    )

    return parser


def main() -> None:
    """Parse arguments and dispatch to the appropriate subcommand."""
    parser = build_parser()
    args = parser.parse_args()
    command: str = args.command

    if command == "status":
        do_status()
        return

    editor_commands = {"refresh", "play", "test", "cycle"}

    if command in editor_commands:
        if is_worktree():
            print(
                "Error: Unity commands cannot run from a git worktree.\n"
                "Hammerspoon can only interact with the Unity Editor open in the\n"
                "main working copy. Run from the main repository instead.",
                file=sys.stderr,
            )
            sys.exit(1)

        try:
            if not shutil.which("hs"):
                raise HammerspoonError(
                    "'hs' CLI not found on PATH. "
                    "Install via: Hammerspoon > Preferences > Install CLI tool"
                )

            ensure_hammerspoon()

            if command == "refresh":
                do_refresh(play=args.play)
            elif command == "play":
                result_msg = toggle_play_mode()
                print(result_msg)
            elif command == "test":
                do_test()
            elif command == "cycle":
                do_cycle()

        except AbuError as e:
            print(f"Error: {e}", file=sys.stderr)
            sys.exit(1)
    else:
        params = build_params(args)
        port = int(os.environ.get("ABU_PORT", "9999"))

        try:
            if args.wait is not None:
                response = send_command_with_wait(command, params, port, args.wait)
            else:
                response = send_command(command, params, port)
            output = handle_response(command, response, json_output=args.json)
        except AbuError as e:
            print(f"Error: {e}", file=sys.stderr)
            sys.exit(1)

        print(output)


if __name__ == "__main__":
    main()
