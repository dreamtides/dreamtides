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
import signal
import socket
import subprocess
import sys
import tempfile
import time
import uuid
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any

import worktree as worktree_mod

# Save builtins before shadowing with domain-specific subclasses.
builtins_ConnectionRefusedError = ConnectionRefusedError
builtins_TimeoutError = TimeoutError

UNITY_BUNDLE_ID = "com.unity3d.UnityEditor5.x"
EDITOR_LOG = Path.home() / "Library" / "Logs" / "Unity" / "Editor.log"
TIMEOUT_SECONDS = 120
TEST_TIMEOUT_SECONDS = 300
POLL_INTERVAL = 0.3
DEFAULT_ABU_PORT = 9999
ABU_STATE_FILE = Path(__file__).resolve().parent.parent.parent / ".abu-state.json"
WORKTREE_BASE = Path.home() / "dreamtides-worktrees"
PORTS_FILE = WORKTREE_BASE / ".ports.json"
MAIN_REPO_ROOT = Path(__file__).resolve().parent.parent.parent
SAVE_DIR = (
    Path.home()
    / "Library"
    / "Application Support"
    / "Dreamtides"
    / "Dreamtides"
)
RESTART_TIMEOUT_SECONDS = 180
UNITY_EXECUTABLE_PATTERN = "/Unity.app/Contents/MacOS/Unity"
CLIENT_DIR = Path(__file__).resolve().parent.parent.parent / "client"

# Mode name mapping: normalized input → (menu label, GameMode enum name for log)
MODE_MAP: dict[str, tuple[str, str]] = {
    "quest": ("Quest", "Quest"),
    "battle": ("Battle", "Battle"),
    "prototypequest": ("Prototype Quest", "PrototypeQuest"),
    "prototype quest": ("Prototype Quest", "PrototypeQuest"),
}

# Device name mapping: slug → (menu label, slug for log confirmation)
DEVICE_MAP: dict[str, tuple[str, str]] = {
    "landscape-16x9": ("Landscape 16:9 (1920x1080)", "landscape-16x9"),
    "landscape-16x10": ("Landscape 16:10 (2560x1600)", "landscape-16x10"),
    "landscape-21x9": ("Landscape 21:9 (3440x1440)", "landscape-21x9"),
    "landscape-3x2": ("Landscape 3:2 (1470x956)", "landscape-3x2"),
    "landscape-5x4": ("Landscape 5:4 (1280x1024)", "landscape-5x4"),
    "landscape-32x9": ("Landscape 32:9 (5120x1440)", "landscape-32x9"),
    "iphone-12": ("iPhone 12 (1170x2532)", "iphone-12"),
    "iphone-se": ("iPhone SE (750x1334)", "iphone-se"),
    "ipad-pro-12": ("iPad Pro 12 (2048x2732)", "ipad-pro-12"),
    "ipod-touch-6": ("iPod Touch 6 (640x1136)", "ipod-touch-6"),
    "samsung-note-20": ("Samsung Note 20 (1440x3088)", "samsung-note-20"),
    "samsung-z-fold-2": ("Samsung Z Fold 2 (960x2658)", "samsung-z-fold-2"),
    "pixel-5": ("Pixel 5 (1080x2340)", "pixel-5"),
}


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
        if getattr(args, "effect_logs", False):
            params["effectLogs"] = True
        return params

    if command == "click":
        params: dict[str, Any] = {"ref": strip_ref(args.ref)}
        if getattr(args, "effect_logs", False):
            params["effectLogs"] = True
        return params

    if command == "hover":
        params: dict[str, Any] = {"ref": strip_ref(args.ref)}
        if getattr(args, "effect_logs", False):
            params["effectLogs"] = True
        return params

    if command == "drag":
        params: dict[str, Any] = {"source": strip_ref(args.source)}
        if args.target is not None:
            params["target"] = strip_ref(args.target)
        if getattr(args, "effect_logs", False):
            params["effectLogs"] = True
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

    effect_logs = data.get("effectLogs")
    if effect_logs and isinstance(effect_logs, list) and len(effect_logs) > 0:
        parts.append("--- Effect Logs ---")
        for entry in effect_logs:
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
            stdin=subprocess.DEVNULL,
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
        raise HammerspoonError(
            "Hammerspoon CLI timed out. "
            "Try: killall Hammerspoon && open -a Hammerspoon"
        )

    if result.returncode != 0:
        stderr = result.stderr.strip()
        if "ipc" in stderr.lower():
            raise HammerspoonError(
                f"Hammerspoon IPC connection failed: {stderr}\n"
                "Try: killall Hammerspoon && open -a Hammerspoon"
            )
        raise HammerspoonError(f"hs failed: {stderr}")

    lines = result.stdout.strip().splitlines()
    return "\n".join(
        line for line in lines if not line.startswith("-- Loading extension:")
    )


def _restart_hammerspoon() -> None:
    """Kill and relaunch Hammerspoon."""
    subprocess.run(
        ["pkill", "-x", "Hammerspoon"],
        capture_output=True,
    )
    time.sleep(2)
    subprocess.run(["open", "-a", "Hammerspoon"])
    time.sleep(5)


def ensure_hammerspoon() -> None:
    """Verify Hammerspoon is running with a working IPC connection.

    Launches Hammerspoon if it is not running. Performs a health check
    to verify IPC is responsive. If the check fails, restarts
    Hammerspoon and retries once before raising.
    """
    result = subprocess.run(["pgrep", "-x", "Hammerspoon"], capture_output=True)
    if result.returncode != 0:
        print("Hammerspoon is not running. Launching...")
        subprocess.run(["open", "-a", "Hammerspoon"])
        time.sleep(5)
        print("Hammerspoon launched.")

    try:
        run_hs('return "ok"')
    except HammerspoonError:
        print("Hammerspoon IPC is not responding. Restarting...")
        _restart_hammerspoon()
        try:
            run_hs('return "ok"')
        except HammerspoonError:
            raise HammerspoonError(
                "Hammerspoon is not responding after restart. "
                "Try manually relaunching from the menu bar."
            )
        print("Hammerspoon restarted successfully.")


def is_worktree() -> bool:
    """Check if the current directory is inside a git worktree."""
    try:
        result = subprocess.run(
            ["git", "rev-parse", "--git-dir", "--git-common-dir"],
            capture_output=True, text=True, check=True,
        )
        lines = result.stdout.strip().splitlines()
        if len(lines) != 2:
            return False
        return os.path.realpath(lines[0]) != os.path.realpath(lines[1])
    except subprocess.CalledProcessError:
        return False


def resolve_worktree_name() -> str | None:
    """Return the worktree name if running inside a worktree, else None."""
    try:
        resolved = MAIN_REPO_ROOT.resolve()
        base = WORKTREE_BASE.resolve()
        if resolved.is_relative_to(base):
            return resolved.relative_to(base).parts[0]
    except (ValueError, IndexError):
        pass
    return None


def resolve_port() -> int:
    """Resolve the ABU port: env var > worktree .ports.json > default 9999."""
    env_port = os.environ.get("ABU_PORT")
    if env_port:
        return int(env_port)
    name = resolve_worktree_name()
    if name is None:
        return DEFAULT_ABU_PORT
    try:
        ports: dict[str, int] = json.loads(PORTS_FILE.read_text())
        if name in ports:
            return ports[name]
    except (OSError, json.JSONDecodeError):
        pass
    print(
        f"Error: Worktree '{name}' has no port assigned in {PORTS_FILE}.\n"
        f"Run 'abu worktree create' to set up the worktree properly.",
        file=sys.stderr,
    )
    sys.exit(1)


def resolve_editor_log() -> Path:
    """Resolve the Editor log path from the state file or fall back to default."""
    state = read_state_file()
    if state:
        log_file = state.get("logFile")
        if log_file:
            return Path(log_file)
    return EDITOR_LOG


def all_state_files() -> list[Path]:
    """Return paths to all abu state files (main repo + worktrees)."""
    seen: set[Path] = set()
    files: list[Path] = []
    for candidate in [ABU_STATE_FILE]:
        resolved = candidate.resolve()
        if resolved not in seen and resolved.exists():
            seen.add(resolved)
            files.append(candidate)
    if WORKTREE_BASE.is_dir():
        for child in WORKTREE_BASE.iterdir():
            if child.is_dir():
                candidate = child / ".abu-state.json"
                resolved = candidate.resolve()
                if candidate.exists() and resolved not in seen:
                    seen.add(resolved)
                    files.append(candidate)
    return files


def check_log_conflict() -> None:
    """Check that no two live editors share the same log file path."""
    log_to_editors: dict[str, list[str]] = {}
    for state_file in all_state_files():
        try:
            state = json.loads(state_file.read_text())
        except (OSError, json.JSONDecodeError):
            continue
        pid = state.get("unityPid", 0)
        if not pid or not is_pid_alive(pid):
            continue
        log_file = state.get("logFile", str(EDITOR_LOG))
        log_to_editors.setdefault(log_file, []).append(str(state_file.parent))

    for log_path, editors in log_to_editors.items():
        if len(editors) > 1:
            print(
                f"Error: Multiple live editors share log file {log_path}:\n"
                + "\n".join(f"  - {e}" for e in editors)
                + "\nRestart the conflicting editor with 'abu restart' to assign "
                "a per-worktree log file.",
                file=sys.stderr,
            )
            sys.exit(1)


def send_menu_item(path: list[str]) -> str:
    """Send a selectMenuItem command to Unity via Hammerspoon.

    Uses PID-targeted lookup when the state file has a live unityPid,
    falling back to bundle ID search otherwise.
    """
    lua_path = ", ".join(f'"{item}"' for item in path)
    menu_label = " > ".join(path)

    # Try to get a live PID from the state file for targeted lookup
    target_pid: int | None = None
    state = read_state_file()
    if state:
        pid = state.get("unityPid", 0)
        if pid and is_pid_alive(pid):
            target_pid = pid

    if target_pid is not None:
        find_app = f"hs.application.applicationForPID({target_pid})"
    else:
        find_app = f'hs.application.find("{UNITY_BUNDLE_ID}")'

    lua = f"""
    local app = {find_app}
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


def get_log_size(log_path: Path | None = None) -> int:
    """Return the current size of the Unity Editor log."""
    if log_path is None:
        log_path = resolve_editor_log()
    try:
        return log_path.stat().st_size
    except OSError:
        return 0


def read_new_log(offset: int, log_path: Path | None = None) -> str:
    """Read new content from the Unity Editor log starting at offset."""
    if log_path is None:
        log_path = resolve_editor_log()
    try:
        with open(log_path, "r", errors="replace") as f:
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


def is_play_mode_active(port: int | None = None) -> bool:
    """Check if Unity is in play mode by probing the Abu TCP port."""
    if port is None:
        port = resolve_port()
    sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    try:
        sock.settimeout(1.0)
        sock.connect(("localhost", port))
        sock.close()
        return True
    except (builtins_ConnectionRefusedError, OSError):
        sock.close()
        return False


def enter_play_mode() -> None:
    """Toggle play mode on and verify it actually started."""
    play_result = toggle_play_mode()
    print(play_result)
    time.sleep(3)
    if not is_play_mode_active():
        print(
            "Error: Play mode did not start. This usually means Unity has "
            "unresolved compilation errors.",
            file=sys.stderr,
        )
        sys.exit(1)


def ensure_unity_running() -> None:
    """Ensure Unity is running with an accessible log file.

    For worktrees, automatically launches Unity and waits for readiness
    if it is not already running or if the log file has become inaccessible
    (e.g. the Logs/ directory was deleted by git clean).
    """
    state = read_state_file()
    unity_alive = False
    if state:
        pid = state.get("unityPid", 0)
        if pid and is_pid_alive(pid):
            unity_alive = True

    log_path = resolve_editor_log()
    log_ok = log_path.exists()

    if unity_alive and log_ok:
        return

    wt_name = resolve_worktree_name()
    if wt_name is None:
        if not log_ok and log_path != EDITOR_LOG:
            print(
                f"Error: Editor log file not found at {log_path}\n"
                "Restart Unity with 'abu restart' to fix.",
                file=sys.stderr,
            )
            sys.exit(1)
        return

    # Kill orphaned Unity if log is inaccessible
    if unity_alive and not log_ok and state is not None:
        old_pid = state["unityPid"]
        print(f"Log file inaccessible, killing Unity (PID {old_pid})...")
        os.kill(old_pid, signal.SIGKILL)
        deadline = time.monotonic() + 10
        while time.monotonic() < deadline and is_pid_alive(old_pid):
            time.sleep(0.2)

    print(f"Launching Unity for worktree '{wt_name}'...")
    do_open(wt_name)
    _wait_for_unity_startup(wt_name)


def _wait_for_unity_startup(wt_name: str) -> None:
    """Poll until the worktree Unity has started and the log file exists."""
    worktree_root = WORKTREE_BASE / wt_name
    state_path = worktree_root / ".abu-state.json"
    print("Waiting for Unity Editor to be ready...")

    start = time.monotonic()
    while time.monotonic() - start < RESTART_TIMEOUT_SECONDS:
        try:
            state = json.loads(state_path.read_text())
        except (OSError, json.JSONDecodeError):
            time.sleep(POLL_INTERVAL)
            continue

        pid = state.get("unityPid", 0)
        if not pid or not is_pid_alive(pid):
            time.sleep(POLL_INTERVAL)
            continue

        log_file = state.get("logFile")
        if log_file and Path(log_file).exists():
            print("Unity Editor is ready.")
            return

        time.sleep(POLL_INTERVAL)

    print(
        f"Warning: Timed out after {RESTART_TIMEOUT_SECONDS}s waiting for "
        "Unity to start. It may still be loading.",
        file=sys.stderr,
    )


def do_refresh(play: bool = False) -> None:
    """Execute a refresh and optionally enter play mode."""
    check_log_conflict()
    ensure_unity_running()
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
        enter_play_mode()


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
    enter_play_mode()


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
    port = resolve_port()
    state = read_state_file()
    tcp_up = is_play_mode_active(port)
    wt_name = resolve_worktree_name()

    print("Unity Editor Status")
    print("=" * 40)

    if wt_name:
        print(f"  Worktree:      {wt_name}")
    else:
        print("  Worktree:      (main repo)")

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

    print(f"  TCP (:{port}):   {'reachable' if tcp_up else 'unreachable'}")


@dataclass(frozen=True)
class UnityProcessInfo:
    """Information about a running Unity Editor process."""

    pid: int
    executable: str
    project_path: str


def find_unity_process() -> UnityProcessInfo:
    """Find the main Unity Editor process via ps.

    Searches for processes whose command matches the Unity executable
    pattern, then filters out batch-mode workers (AssetImportWorker
    subprocesses). Extracts the executable path and project path from
    the full command-line arguments. Raises UnityNotFoundError if no
    Unity editor process is found.
    """
    try:
        result = subprocess.run(
            ["ps", "-eo", "pid,comm"],
            capture_output=True, text=True, check=True,
        )
    except subprocess.CalledProcessError as e:
        raise AbuError(f"Failed to list processes: {e}")

    candidates: list[tuple[int, str]] = []
    for line in result.stdout.splitlines():
        line = line.strip()
        parts = line.split(None, 1)
        if len(parts) == 2 and UNITY_EXECUTABLE_PATTERN in parts[1]:
            try:
                candidates.append((int(parts[0]), parts[1].strip()))
            except ValueError:
                continue

    if not candidates:
        raise UnityNotFoundError(
            "No running Unity Editor process found. "
            "Is Unity open?"
        )

    # Find the main editor process (not a -batchMode worker).
    # Prefer the process whose -projectPath matches CLIENT_DIR.
    target_client = str(Path(str(CLIENT_DIR)).resolve())
    matched: UnityProcessInfo | None = None
    first_non_batch: UnityProcessInfo | None = None

    for pid, executable in candidates:
        try:
            args_result = subprocess.run(
                ["ps", "-p", str(pid), "-o", "args="],
                capture_output=True, text=True, check=True,
            )
        except subprocess.CalledProcessError:
            continue

        args_line = args_result.stdout.strip()
        if "-batchMode" in args_line:
            continue

        project_path = str(CLIENT_DIR)
        match = re.search(r"-projectPath\s+(\S+)", args_line, re.IGNORECASE)
        if match:
            project_path = match.group(1)

        info = UnityProcessInfo(
            pid=pid, executable=executable, project_path=project_path,
        )

        if first_non_batch is None:
            first_non_batch = info

        try:
            if Path(project_path).resolve() == Path(target_client):
                matched = info
        except (OSError, ValueError):
            pass

    if matched is not None:
        return matched
    if first_non_batch is not None:
        return first_non_batch

    # All candidates were batch-mode workers; use the first one as fallback
    pid, executable = candidates[0]
    return UnityProcessInfo(
        pid=pid, executable=executable, project_path=str(CLIENT_DIR),
    )


def find_unity_executable(client_path: Path) -> tuple[str, Path]:
    """Find the Unity executable for a given client project directory.

    Reads ProjectVersion.txt to determine the Unity version, then locates
    the Unity.app bundle in the standard Hub install location. Returns a
    tuple of (version_string, app_path). Raises AbuError if the version
    file is missing or the Unity installation is not found.
    """
    version_file = client_path / "ProjectSettings" / "ProjectVersion.txt"
    if not version_file.exists():
        raise AbuError(f"ProjectVersion.txt not found at {version_file}")

    version: str | None = None
    for line in version_file.read_text().splitlines():
        match = re.match(r"m_EditorVersion:\s*(.+)", line)
        if match:
            version = match.group(1).strip()
            break

    if not version:
        raise AbuError(f"Could not parse m_EditorVersion from {version_file}")

    app_path = Path(f"/Applications/Unity/Hub/Editor/{version}/Unity.app")
    if not app_path.exists():
        raise AbuError(
            f"Unity {version} not found at {app_path}. "
            "Is this version installed via Unity Hub?"
        )

    return version, app_path


def do_open(name: str) -> None:
    """Open a worktree project in Unity with a per-worktree log file."""
    worktree_root = WORKTREE_BASE / name
    if not worktree_root.is_dir():
        raise AbuError(
            f"Worktree '{name}' not found at {worktree_root}. "
            "Run 'abu worktree create' first."
        )

    client_path = worktree_root / "client"
    if not client_path.is_dir():
        raise AbuError(f"Client directory not found at {client_path}")

    version, app_path = find_unity_executable(client_path)

    # Use .abu-logs instead of Logs to avoid .gitignore 'Logs/' pattern
    log_dir = worktree_root / ".abu-logs"
    log_dir.mkdir(parents=True, exist_ok=True)
    log_path = log_dir / "Editor.log"

    # Read the assigned port for this worktree, if any
    port: int | None = None
    try:
        ports: dict[str, int] = json.loads(PORTS_FILE.read_text())
        port = ports.get(name)
    except (OSError, json.JSONDecodeError):
        pass

    launch_args: list[str] = [
        "open", "-n", "-a", str(app_path), "--args",
        "-projectPath", str(client_path),
        "-logFile", str(log_path),
    ]

    subprocess.Popen(
        launch_args,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    )

    print(f"Launching Unity {version} for worktree '{name}'")
    print(f"  Project: {client_path}")
    print(f"  Log file: {log_path}")
    if port is not None:
        print(f"  ABU port: {port}")


def do_restart() -> None:
    """Kill the running Unity Editor and relaunch it with the same project."""
    check_log_conflict()
    print("Finding Unity Editor process...")
    info = find_unity_process()
    print(f"  Found Unity (PID {info.pid}): {info.executable}")
    print(f"  Project path: {info.project_path}")

    # Read current scene from state file
    state = read_state_file()
    active_scene = state.get("activeScene", "") if state else ""

    # Write restart marker file
    marker_path = Path(info.project_path) / ".abu-restart.json"
    if active_scene:
        marker = json.dumps({"scene": active_scene})
        marker_path.write_text(marker)
        print(f"  Scene to restore: {active_scene}")

    # Kill Unity with SIGKILL (works even when frozen)
    print(f"Killing Unity (PID {info.pid})...")
    os.kill(info.pid, signal.SIGKILL)

    # Wait for process death
    deadline = time.monotonic() + 10
    while time.monotonic() < deadline and is_pid_alive(info.pid):
        time.sleep(0.2)

    if is_pid_alive(info.pid):
        print("Warning: Unity process did not exit within 10 seconds", file=sys.stderr)

    print("Unity process terminated.")

    # Clean up crash artifacts to prevent the "recovering scene backups"
    # dialog from blocking startup.
    temp_dir = Path(info.project_path) / "Temp"
    for name in ("__Backupscenes", "BackupScenes"):
        backup_dir = temp_dir / name
        if backup_dir.is_dir():
            shutil.rmtree(backup_dir, ignore_errors=True)
            print(f"  Cleaned up {name}")

    # Relaunch Unity via 'open' for proper macOS app activation.
    # Extract the .app bundle path from the executable path.
    app_idx = info.executable.find(".app/")
    app_path = info.executable[:app_idx + 4] if app_idx >= 0 else info.executable
    print(f"Relaunching Unity ({app_path})...")

    launch_args: list[str] = [
        "open", "-n", "-a", app_path, "--args",
        "-projectPath", info.project_path,
    ]

    # For worktree editors, assign a per-worktree log file
    wt_name = resolve_worktree_name()
    if wt_name:
        log_dir = WORKTREE_BASE / wt_name / ".abu-logs"
        log_dir.mkdir(parents=True, exist_ok=True)
        restart_log = log_dir / "Editor.log"
        launch_args.extend(["-logFile", str(restart_log)])
        print(f"  Log file: {restart_log}")
    else:
        restart_log = EDITOR_LOG

    subprocess.Popen(
        launch_args,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    )

    # Poll Editor.log for [AbuRestart] Ready (domain reload + initial
    # compilation done), then wait for log stability to ensure asset
    # import workers and background tasks finish. Unity truncates
    # Editor.log on startup, so scan from offset 0.
    print("Waiting for Unity Editor to be ready...")
    start = time.monotonic()
    saw_marker = False
    last_log_size = 0
    stable_since: float | None = None
    log_stable_seconds = 10.0

    while time.monotonic() - start < RESTART_TIMEOUT_SECONDS:
        if not saw_marker:
            content = read_new_log(0, restart_log)
            if "[AbuRestart] Ready" in content:
                saw_marker = True
                last_log_size = get_log_size(restart_log)
                stable_since = time.monotonic()
                print("  Domain reload complete, waiting for editor to settle...")
        else:
            current_size = get_log_size(restart_log)
            if current_size != last_log_size:
                last_log_size = current_size
                stable_since = time.monotonic()
            elif stable_since and time.monotonic() - stable_since >= log_stable_seconds:
                print("Unity Editor is ready.")
                return

        time.sleep(POLL_INTERVAL)

    print(
        f"Warning: Timed out after {RESTART_TIMEOUT_SECONDS}s waiting for "
        "Unity to signal readiness. Unity may still be starting up.",
        file=sys.stderr,
    )


def do_clear_save() -> None:
    """Delete all save files from the Dreamtides save directory."""
    matches = sorted(SAVE_DIR.glob("save-*.json"))
    if not matches:
        print("No save files found.")
        return
    for path in matches:
        path.unlink()
        print(f"  Deleted {path.name}")
    print(f"Cleared {len(matches)} save file(s).")


def do_set_mode(mode_name: str) -> None:
    """Set the Unity play mode game mode via the Tools > Play Mode menu.

    Drives the menu item via Hammerspoon and polls Editor.log until the
    confirmation Debug.Log fires.
    """
    normalized = mode_name.lower().strip()
    if normalized not in MODE_MAP:
        valid = ", ".join(sorted({v[0] for v in MODE_MAP.values()}))
        print(
            f"Error: Unknown mode '{mode_name}'. Valid modes: {valid}",
            file=sys.stderr,
        )
        sys.exit(1)

    menu_label, log_enum = MODE_MAP[normalized]
    expected_log = f"Set play mode to {log_enum}"

    log_offset = get_log_size()
    result_msg = send_menu_item(["Tools", "Play Mode", menu_label])
    print(result_msg)

    print(f"Waiting for mode change to {menu_label}...")
    start = time.time()
    while time.time() - start < 30:
        content = read_new_log(log_offset)
        if expected_log in content:
            print(f"Mode set to {menu_label}.")
            return
        time.sleep(POLL_INTERVAL)

    print(
        f"Warning: Did not see '{expected_log}' in Editor.log within 30s",
        file=sys.stderr,
    )
    sys.exit(1)


def do_set_device(device_name: str) -> None:
    """Set the Unity device/resolution via the Tools > Device menu.

    Drives the menu item via Hammerspoon and polls Editor.log until the
    confirmation Debug.Log fires.
    """
    normalized = device_name.lower().strip()
    if normalized not in DEVICE_MAP:
        valid = ", ".join(sorted(DEVICE_MAP.keys()))
        print(
            f"Error: Unknown device '{device_name}'. Valid devices: {valid}",
            file=sys.stderr,
        )
        sys.exit(1)

    menu_label, slug = DEVICE_MAP[normalized]
    expected_log = f"Set device to {slug}"

    log_offset = get_log_size()
    result_msg = send_menu_item(["Tools", "Device", menu_label])
    print(result_msg)

    print(f"Waiting for device change to {menu_label}...")
    start = time.time()
    while time.time() - start < 30:
        content = read_new_log(log_offset)
        if expected_log in content:
            print(f"Device set to {menu_label}.")
            return
        time.sleep(POLL_INTERVAL)

    print(
        f"Warning: Did not see '{expected_log}' in Editor.log within 30s",
        file=sys.stderr,
    )
    sys.exit(1)


def do_create_save(args: argparse.Namespace) -> None:
    """Generate a test save file by invoking the test_save_generator binary."""
    project_root = Path(__file__).resolve().parent.parent.parent
    binary = "test_save_generator"

    # Build the binary first
    print("Building test_save_generator...")
    build_result = subprocess.run(
        ["cargo", "build", "--release", "-p", "test_save_generator"],
        cwd=project_root / "rules_engine",
        capture_output=True,
        text=True,
    )
    if build_result.returncode != 0:
        print(f"Build failed:\n{build_result.stderr}", file=sys.stderr)
        sys.exit(1)

    binary_path = (
        project_root / "rules_engine" / "target" / "release" / binary
    )

    cmd: list[str] = [str(binary_path)]
    cmd.extend(["--save-dir", str(SAVE_DIR)])

    if args.list_cards:
        cmd.append("--list-cards")
    else:
        # Clear existing saves first so the game loads the new one
        do_clear_save()

        if args.energy is not None:
            cmd.extend(["--energy", str(args.energy)])
        if args.cards:
            for card_name in args.cards:
                cmd.extend(["--card", card_name])

    result = subprocess.run(cmd, capture_output=True, text=True)
    if result.stdout:
        print(result.stdout, end="")
    if result.stderr:
        print(result.stderr, end="", file=sys.stderr)
    if result.returncode != 0:
        sys.exit(result.returncode)


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
    snapshot_parser.add_argument(
        "--effect-logs", action="store_true", help="Include visual effect logs in output"
    )

    # click
    click_parser = subparsers.add_parser("click", help="Click a UI element")
    click_parser.add_argument("ref", help="Element ref (e.g. e1 or @e1)")
    click_parser.add_argument(
        "--effect-logs", action="store_true", help="Include visual effect logs in output"
    )

    # hover
    hover_parser = subparsers.add_parser("hover", help="Hover over a UI element")
    hover_parser.add_argument("ref", help="Element ref (e.g. e1 or @e1)")
    hover_parser.add_argument(
        "--effect-logs", action="store_true", help="Include visual effect logs in output"
    )

    # drag
    drag_parser = subparsers.add_parser("drag", help="Drag from source to target")
    drag_parser.add_argument("source", help="Source element ref")
    drag_parser.add_argument("target", nargs="?", default=None, help="Target element ref")
    drag_parser.add_argument(
        "--effect-logs", action="store_true", help="Include visual effect logs in output"
    )

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

    # open
    open_parser = subparsers.add_parser(
        "open", help="Open a worktree project in Unity with a per-worktree log file"
    )
    open_parser.add_argument("name", help="Worktree name (e.g. alpha)")

    # restart
    subparsers.add_parser(
        "restart", help="Kill and relaunch Unity Editor, restoring the active scene"
    )

    # clear-save
    subparsers.add_parser(
        "clear-save", help="Delete all Dreamtides save files"
    )

    # set-mode
    set_mode_parser = subparsers.add_parser(
        "set-mode", help="Set the game mode for play mode (Quest, Battle, PrototypeQuest)"
    )
    set_mode_parser.add_argument(
        "mode", help="Mode name: Quest, Battle, or PrototypeQuest"
    )

    # set-device
    set_device_parser = subparsers.add_parser(
        "set-device",
        help="Set the device/resolution for Play Mode (e.g. iphone-se, landscape-16x9)",
    )
    set_device_parser.add_argument(
        "device",
        help="Device slug: " + ", ".join(sorted(DEVICE_MAP.keys())),
    )

    # create-save
    create_save_parser = subparsers.add_parser(
        "create-save",
        help="Generate a test save file with custom battle parameters",
    )
    create_save_parser.add_argument(
        "--energy", type=int, default=None,
        help="Set player energy to this value",
    )
    create_save_parser.add_argument(
        "--card", action="append", default=None, dest="cards",
        help="Add a card to player's hand by name (can be repeated)",
    )
    create_save_parser.add_argument(
        "--list-cards", action="store_true",
        help="List all available card names and exit",
    )

    # worktree
    worktree_mod.register_subcommands(subparsers)

    return parser


def main() -> None:
    """Parse arguments and dispatch to the appropriate subcommand."""
    parser = build_parser()
    args = parser.parse_args()
    command: str = args.command

    if command == "status":
        do_status()
        return

    if command == "clear-save":
        do_clear_save()
        return

    if command == "create-save":
        do_create_save(args)
        return

    if command == "worktree":
        worktree_mod.dispatch(args)
        return

    if command == "open":
        try:
            do_open(args.name)
        except AbuError as e:
            print(f"Error: {e}", file=sys.stderr)
            sys.exit(1)
        return

    editor_commands = {"refresh", "play", "test", "cycle", "restart", "set-mode", "set-device"}

    if command in editor_commands:
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
            elif command == "restart":
                do_restart()
            elif command == "set-mode":
                do_set_mode(args.mode)
            elif command == "set-device":
                do_set_device(args.device)

        except AbuError as e:
            print(f"Error: {e}", file=sys.stderr)
            sys.exit(1)
    else:
        params = build_params(args)
        port = resolve_port()

        try:
            if args.wait is not None:
                response = send_command_with_wait(command, params, port, args.wait)
            else:
                response = send_command(command, params, port)
            output = handle_response(command, response, json_output=args.json)
            print(output)
        except AbuError as e:
            print(f"Error: {e}", file=sys.stderr)
            sys.exit(1)


if __name__ == "__main__":
    main()
