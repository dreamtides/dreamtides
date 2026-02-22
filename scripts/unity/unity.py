#!/usr/bin/env python3
"""Control Unity Editor via Hammerspoon for asset refresh, play mode, and tests.

Provides subcommands for Unity Editor operations: refresh (trigger asset
compilation and wait for completion), play (toggle play mode), and test
(refresh then run all Edit Mode tests). Uses Hammerspoon's selectMenuItem
to drive Unity's menu bar and tails the Editor log to detect results.

Prerequisites:
  1. Hammerspoon.app installed and running
  2. ~/.hammerspoon/init.lua contains: require("hs.ipc")
  3. hs CLI installed (Hammerspoon > Preferences > Install CLI tool)
  4. Hammerspoon granted Accessibility in System Settings > Privacy & Security
"""

import argparse
import re
import shutil
import subprocess
import sys
import time
from dataclasses import dataclass, field
from pathlib import Path

UNITY_BUNDLE_ID = "com.unity3d.UnityEditor5.x"
EDITOR_LOG = Path.home() / "Library" / "Logs" / "Unity" / "Editor.log"
TIMEOUT_SECONDS = 120
TEST_TIMEOUT_SECONDS = 300
POLL_INTERVAL = 0.3
ABU_PORT = 9999


class UnityError(Exception):
    """Base exception for Unity script errors."""


class HammerspoonError(UnityError):
    """Raised when Hammerspoon CLI interaction fails."""


class UnityNotFoundError(UnityError):
    """Raised when Unity Editor is not running."""


class RefreshTimeoutError(UnityError):
    """Raised when waiting for refresh completion exceeds the timeout."""


class CompilationError(UnityError):
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
            if "[TestRunner] Run finished:" in line:
                failures = []
                for log_line in content.splitlines():
                    if "[TestRunner] FAIL:" in log_line:
                        # Extract the FAIL message after the Unity log prefix
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
    import socket as _socket

    sock = _socket.socket(_socket.AF_INET, _socket.SOCK_STREAM)
    try:
        sock.settimeout(1.0)
        sock.connect(("localhost", ABU_PORT))
        sock.close()
        return True
    except (ConnectionRefusedError, OSError):
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
    # Step 1: Refresh to trigger recompilation
    do_refresh()

    # Step 2: Run tests
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


def build_parser() -> argparse.ArgumentParser:
    """Build the argument parser with all subcommands."""
    parser = argparse.ArgumentParser(
        prog="unity.py",
        description="Control Unity Editor via Hammerspoon.",
    )
    subparsers = parser.add_subparsers(dest="command", required=True)

    refresh_parser = subparsers.add_parser(
        "refresh", help="Trigger asset refresh and wait for completion"
    )
    refresh_parser.add_argument(
        "--play", action="store_true",
        help="Enter play mode after successful refresh",
    )

    subparsers.add_parser("play", help="Toggle play mode")

    subparsers.add_parser(
        "test", help="Refresh then run all Edit Mode tests"
    )

    subparsers.add_parser(
        "cycle", help="Exit play mode (if active), refresh, re-enter play mode"
    )

    return parser


def main() -> None:
    """Parse arguments and dispatch to the appropriate subcommand."""
    parser = build_parser()
    args = parser.parse_args()

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

        if args.command == "refresh":
            do_refresh(play=args.play)

        elif args.command == "play":
            result_msg = toggle_play_mode()
            print(result_msg)

        elif args.command == "test":
            do_test()

        elif args.command == "cycle":
            do_cycle()

    except UnityError as e:
        print(f"Error: {e}", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()
