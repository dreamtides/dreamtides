#!/usr/bin/env python3
"""Tests for abu.py CLI functions."""

import argparse
import base64
import json
import os
import socket
import subprocess
import threading
import unittest
import uuid
from pathlib import Path
from unittest.mock import MagicMock, patch

from abu import (
    ABU_STATE_FILE,
    AbuError,
    CompilationError,
    ConnectionError,
    DEFAULT_ABU_PORT,
    EmptyResponseError,
    HammerspoonError,
    RefreshResult,
    RefreshTimeoutError,
    UnityNotFoundError,
    UnityProcessInfo,
    _report_result,
    build_command,
    build_params,
    build_parser,
    check_log_conflict,
    do_open,
    do_status,
    find_unity_executable,
    find_unity_process,
    handle_response,
    is_pid_alive,
    is_worktree,
    read_state_file,
    resolve_port,
    resolve_worktree_name,
    send_command,
    send_menu_item,
    strip_ref,
    wait_for_refresh,
)


class TestStripRef(unittest.TestCase):
    """Test the @ prefix stripping from ref arguments."""

    def test_strips_at_prefix(self) -> None:
        self.assertEqual(strip_ref("@e1"), "e1")

    def test_leaves_plain_ref(self) -> None:
        self.assertEqual(strip_ref("e1"), "e1")

    def test_strips_only_first_at(self) -> None:
        self.assertEqual(strip_ref("@@e1"), "@e1")

    def test_empty_after_at(self) -> None:
        self.assertEqual(strip_ref("@"), "")


class TestBuildParams(unittest.TestCase):
    """Test params construction for each command type."""

    def test_snapshot_no_flags(self) -> None:
        ns = argparse.Namespace(command="snapshot", compact=False, interactive=False, max_depth=None)
        self.assertEqual(build_params(ns), {})

    def test_snapshot_compact(self) -> None:
        ns = argparse.Namespace(command="snapshot", compact=True, interactive=False, max_depth=None)
        self.assertEqual(build_params(ns), {"compact": True})

    def test_snapshot_interactive(self) -> None:
        ns = argparse.Namespace(command="snapshot", compact=False, interactive=True, max_depth=None)
        self.assertEqual(build_params(ns), {"interactive": True})

    def test_snapshot_max_depth(self) -> None:
        ns = argparse.Namespace(command="snapshot", compact=False, interactive=False, max_depth=5)
        self.assertEqual(build_params(ns), {"maxDepth": 5})

    def test_snapshot_all_flags(self) -> None:
        ns = argparse.Namespace(command="snapshot", compact=True, interactive=True, max_depth=3)
        self.assertEqual(build_params(ns), {"compact": True, "interactive": True, "maxDepth": 3})

    def test_click_params(self) -> None:
        ns = argparse.Namespace(command="click", ref="@e1")
        self.assertEqual(build_params(ns), {"ref": "e1"})

    def test_hover_params(self) -> None:
        ns = argparse.Namespace(command="hover", ref="e2")
        self.assertEqual(build_params(ns), {"ref": "e2"})

    def test_drag_with_target(self) -> None:
        ns = argparse.Namespace(command="drag", source="@e1", target="@e2")
        self.assertEqual(build_params(ns), {"source": "e1", "target": "e2"})

    def test_drag_without_target(self) -> None:
        ns = argparse.Namespace(command="drag", source="e1", target=None)
        self.assertEqual(build_params(ns), {"source": "e1"})

    def test_screenshot_params(self) -> None:
        ns = argparse.Namespace(command="screenshot")
        self.assertEqual(build_params(ns), {})


class TestBuildCommand(unittest.TestCase):
    """Test NDJSON command construction."""

    def test_command_structure(self) -> None:
        cmd = build_command("snapshot", {"compact": True})
        parsed = json.loads(cmd)
        self.assertIn("id", parsed)
        self.assertEqual(parsed["command"], "snapshot")
        self.assertEqual(parsed["params"], {"compact": True})
        self.assertTrue(cmd.endswith("\n"))

    def test_command_has_uuid(self) -> None:
        cmd = build_command("click", {"ref": "e1"})
        parsed = json.loads(cmd)
        # Should be a valid UUID
        uuid.UUID(parsed["id"])


class TestHandleResponse(unittest.TestCase):
    """Test response handling for each command type."""

    def test_snapshot_response(self) -> None:
        response = {
            "id": "test-id",
            "success": True,
            "data": {
                "snapshot": "- application \"Dreamtides\"",
                "refs": {"e1": {"role": "button", "name": "End Turn"}},
            },
        }
        result = handle_response("snapshot", response)
        self.assertEqual(result, "- application \"Dreamtides\"")

    def test_snapshot_response_json_mode(self) -> None:
        response = {
            "id": "test-id",
            "success": True,
            "data": {
                "snapshot": "- application \"Dreamtides\"",
                "refs": {"e1": {"role": "button", "name": "End Turn"}},
            },
        }
        result = handle_response("snapshot", response, json_output=True)
        parsed = json.loads(result)
        self.assertEqual(parsed["snapshot"], "- application \"Dreamtides\"")
        self.assertIn("e1", parsed["refs"])

    def test_click_response(self) -> None:
        response = {
            "id": "test-id",
            "success": True,
            "data": {
                "clicked": True,
                "snapshot": "- app",
                "refs": {},
            },
        }
        result = handle_response("click", response)
        self.assertEqual(result, "- app")

    def test_click_response_json_mode(self) -> None:
        response = {
            "id": "test-id",
            "success": True,
            "data": {
                "clicked": True,
                "snapshot": "- app",
                "refs": {},
            },
        }
        result = handle_response("click", response, json_output=True)
        parsed = json.loads(result)
        self.assertTrue(parsed["clicked"])

    def test_screenshot_response(self) -> None:
        # Create a tiny valid PNG-like blob
        png_bytes = b"\x89PNG\r\n\x1a\n" + b"\x00" * 16
        b64 = base64.b64encode(png_bytes).decode()
        response = {
            "id": "test-id",
            "success": True,
            "data": {"base64": b64},
        }
        result = handle_response("screenshot", response)
        # Should be a file path
        self.assertTrue(result.endswith(".png"))
        self.assertTrue(os.path.exists(result))
        with open(result, "rb") as f:
            self.assertEqual(f.read(), png_bytes)

    def test_response_with_history(self) -> None:
        response = {
            "id": "test-id",
            "success": True,
            "data": {
                "clicked": True,
                "snapshot": "- app",
                "refs": {},
                "history": [
                    "Opponent's turn begins",
                    "Stormcaller moved from battlefield to void",
                    "Your turn begins",
                ],
            },
        }
        result = handle_response("click", response)
        lines = result.split("\n")
        self.assertEqual(lines[0], "--- History ---")
        self.assertEqual(lines[1], "Opponent's turn begins")
        self.assertEqual(lines[2], "Stormcaller moved from battlefield to void")
        self.assertEqual(lines[3], "Your turn begins")
        self.assertEqual(lines[4], "---")
        self.assertEqual(lines[5], "- app")

    def test_response_without_history(self) -> None:
        response = {
            "id": "test-id",
            "success": True,
            "data": {
                "snapshot": "- app",
                "refs": {},
            },
        }
        result = handle_response("snapshot", response)
        self.assertEqual(result, "- app")

    def test_response_with_empty_history(self) -> None:
        response = {
            "id": "test-id",
            "success": True,
            "data": {
                "clicked": True,
                "snapshot": "- app",
                "refs": {},
                "history": [],
            },
        }
        result = handle_response("click", response)
        self.assertEqual(result, "- app")

    def test_response_with_history_json_mode(self) -> None:
        response = {
            "id": "test-id",
            "success": True,
            "data": {
                "clicked": True,
                "snapshot": "- app",
                "refs": {},
                "history": ["Your turn begins"],
            },
        }
        result = handle_response("click", response, json_output=True)
        parsed = json.loads(result)
        self.assertEqual(parsed["history"], ["Your turn begins"])

    def test_error_response_raises(self) -> None:
        response = {
            "id": "test-id",
            "success": False,
            "error": "Something went wrong",
        }
        with self.assertRaises(AbuError) as ctx:
            handle_response("snapshot", response)
        self.assertIn("Something went wrong", str(ctx.exception))


class TestBuildParser(unittest.TestCase):
    """Test argparse configuration for TCP commands."""

    def test_snapshot_defaults(self) -> None:
        parser = build_parser()
        args = parser.parse_args(["snapshot"])
        self.assertEqual(args.command, "snapshot")
        self.assertFalse(args.compact)
        self.assertFalse(args.interactive)
        self.assertIsNone(args.max_depth)

    def test_snapshot_all_flags(self) -> None:
        parser = build_parser()
        args = parser.parse_args(["snapshot", "--compact", "--interactive", "--max-depth", "5"])
        self.assertTrue(args.compact)
        self.assertTrue(args.interactive)
        self.assertEqual(args.max_depth, 5)

    def test_click_ref(self) -> None:
        parser = build_parser()
        args = parser.parse_args(["click", "@e1"])
        self.assertEqual(args.command, "click")
        self.assertEqual(args.ref, "@e1")

    def test_hover_ref(self) -> None:
        parser = build_parser()
        args = parser.parse_args(["hover", "e2"])
        self.assertEqual(args.command, "hover")
        self.assertEqual(args.ref, "e2")

    def test_drag_with_target(self) -> None:
        parser = build_parser()
        args = parser.parse_args(["drag", "@e1", "@e2"])
        self.assertEqual(args.command, "drag")
        self.assertEqual(args.source, "@e1")
        self.assertEqual(args.target, "@e2")

    def test_drag_without_target(self) -> None:
        parser = build_parser()
        args = parser.parse_args(["drag", "e1"])
        self.assertEqual(args.command, "drag")
        self.assertEqual(args.source, "e1")
        self.assertIsNone(args.target)

    def test_screenshot(self) -> None:
        parser = build_parser()
        args = parser.parse_args(["screenshot"])
        self.assertEqual(args.command, "screenshot")


class TestSendCommand(unittest.TestCase):
    """Test TCP communication with a mock Unity server."""

    def _start_mock_server(
        self,
        response_data: dict | None = None,
        capture: list | None = None,
    ) -> tuple[int, threading.Event]:
        """Start a TCP server that returns a canned NDJSON response.

        If response_data is None, the server accepts the connection and reads
        the command but sends nothing back (simulating an empty response).

        If capture is provided (a list), the parsed command dict received from
        the client is appended to it.
        """
        server = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        server.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
        server.bind(("localhost", 0))
        server.listen(1)
        port = server.getsockname()[1]
        done = threading.Event()

        def serve() -> None:
            try:
                conn, _ = server.accept()
                # Read the command line
                data = b""
                while b"\n" not in data:
                    chunk = conn.recv(4096)
                    if not chunk:
                        break
                    data += chunk
                if capture is not None and data:
                    line = data.split(b"\n", 1)[0]
                    capture.append(json.loads(line.decode("utf-8")))
                # Send the response if one was provided
                if response_data is not None:
                    response_line = json.dumps(response_data).encode("utf-8") + b"\n"
                    conn.sendall(response_line)
                conn.close()
            finally:
                server.close()
                done.set()

        t = threading.Thread(target=serve, daemon=True)
        t.start()
        return port, done

    def test_send_command_success(self) -> None:
        response_data = {
            "id": "test-id",
            "success": True,
            "data": {"snapshot": "- app", "refs": {}},
        }
        port, done = self._start_mock_server(response_data)
        result = send_command("snapshot", {}, port)
        done.wait(timeout=5)
        self.assertTrue(result["success"])
        self.assertEqual(result["data"]["snapshot"], "- app")

    def test_send_command_error_response(self) -> None:
        response_data = {
            "id": "test-id",
            "success": False,
            "error": "Not found",
        }
        port, done = self._start_mock_server(response_data)
        result = send_command("click", {"ref": "e1"}, port)
        done.wait(timeout=5)
        self.assertFalse(result["success"])
        self.assertEqual(result["error"], "Not found")

    def test_send_command_connection_refused(self) -> None:
        # Use a port that nothing is listening on
        with self.assertRaises(ConnectionError):
            send_command("snapshot", {}, 1)

    def test_send_command_sends_valid_ndjson(self) -> None:
        """Verify the server receives valid NDJSON with the expected fields."""
        captured: list[dict] = []
        response_data = {"id": "x", "success": True, "data": {}}
        port, done = self._start_mock_server(response_data, capture=captured)
        send_command("click", {"ref": "e1"}, port)
        done.wait(timeout=5)

        self.assertEqual(len(captured), 1)
        received = captured[0]
        self.assertIn("id", received)
        self.assertEqual(received["command"], "click")
        self.assertEqual(received["params"], {"ref": "e1"})

    def test_send_command_empty_response(self) -> None:
        """Server accepts connection but sends nothing and closes."""
        port, done = self._start_mock_server(response_data=None)
        with self.assertRaises(EmptyResponseError):
            send_command("snapshot", {}, port)
        done.wait(timeout=5)


class TestIsWorktree(unittest.TestCase):
    """Test git worktree detection."""

    @patch("abu.subprocess.run")
    def test_main_repo_not_worktree(self, mock_run: MagicMock) -> None:
        mock_run.return_value = subprocess.CompletedProcess(
            args=[], returncode=0, stdout=".git\n.git\n", stderr=""
        )
        self.assertFalse(is_worktree())

    @patch("abu.subprocess.run")
    def test_subdirectory_not_worktree(self, mock_run: MagicMock) -> None:
        mock_run.return_value = subprocess.CompletedProcess(
            args=[], returncode=0,
            stdout="/Users/me/project/.git\n../../.git\n", stderr=""
        )
        with patch("abu.os.path.realpath", side_effect=[
            "/Users/me/project/.git",
            "/Users/me/project/.git",
        ]):
            self.assertFalse(is_worktree())

    @patch("abu.subprocess.run")
    def test_worktree_detected(self, mock_run: MagicMock) -> None:
        mock_run.return_value = subprocess.CompletedProcess(
            args=[], returncode=0,
            stdout="/Users/me/project/.git/worktrees/branch\n/Users/me/project/.git\n",
            stderr=""
        )
        self.assertTrue(is_worktree())

    @patch("abu.subprocess.run")
    def test_git_failure_returns_false(self, mock_run: MagicMock) -> None:
        mock_run.side_effect = subprocess.CalledProcessError(128, "git")
        self.assertFalse(is_worktree())


class TestReportResult(unittest.TestCase):
    """Test log parsing and RefreshResult construction."""

    def test_no_errors(self) -> None:
        content = (
            "Some log line\n"
            "Asset Pipeline Refresh (id=abc): Total: 1.234s\n"
        )
        result = _report_result(content)
        self.assertTrue(result.finished)
        self.assertTrue(result.success)
        self.assertEqual(result.errors, [])
        self.assertIn("Asset Pipeline Refresh", result.summary)

    def test_compilation_errors(self) -> None:
        content = (
            "Assets/Foo.cs(10,5): error CS1002: ; expected\n"
            "Assets/Bar.cs(20,3): error CS0246: type not found\n"
        )
        result = _report_result(content)
        self.assertTrue(result.finished)
        self.assertFalse(result.success)
        self.assertEqual(len(result.errors), 2)

    def test_duplicate_errors_deduplicated(self) -> None:
        content = (
            "Assets/Foo.cs(10,5): error CS1002: ; expected\n"
            "Assets/Foo.cs(10,5): error CS1002: ; expected\n"
        )
        result = _report_result(content)
        self.assertEqual(len(result.errors), 1)

    def test_empty_content(self) -> None:
        result = _report_result("")
        self.assertTrue(result.finished)
        self.assertTrue(result.success)
        self.assertEqual(result.errors, [])
        self.assertEqual(result.summary, "")

    def test_error_summary_message(self) -> None:
        content = (
            "Assets/Foo.cs(10,5): error CS1002: ; expected\n"
            "Assets/Bar.cs(20,3): error CS0246: type not found\n"
            "Assets/Baz.cs(30,1): error CS0103: name does not exist\n"
        )
        result = _report_result(content)
        self.assertEqual(result.summary, "3 compilation error(s)")


class TestWaitForRefresh(unittest.TestCase):
    """Test refresh polling logic."""

    @patch("abu.TIMEOUT_SECONDS", 0.5)
    @patch("abu.POLL_INTERVAL", 0.1)
    @patch("abu.read_new_log")
    def test_timeout_returns_not_finished(self, mock_read: MagicMock) -> None:
        mock_read.return_value = ""
        result = wait_for_refresh(0)
        self.assertFalse(result.finished)
        self.assertFalse(result.success)

    @patch("abu.POLL_INTERVAL", 0.01)
    @patch("abu.read_new_log")
    def test_no_compilation_refresh_completes(self, mock_read: MagicMock) -> None:
        mock_read.return_value = (
            "RefreshV2(NoUpdateAssetOptions)\n"
            "Asset Pipeline Refresh (id=abc): Total: 0.5s\n"
        )
        result = wait_for_refresh(0)
        self.assertTrue(result.finished)
        self.assertTrue(result.success)

    @patch("abu.POLL_INTERVAL", 0.01)
    @patch("abu.read_new_log")
    def test_compilation_with_errors(self, mock_read: MagicMock) -> None:
        mock_read.return_value = (
            "[ScriptCompilation] Requested\n"
            "RefreshV2(NoUpdateAssetOptions)\n"
            "Assets/Foo.cs(10,5): error CS1002: ; expected\n"
            "StopAssetImportingV2\n"
        )
        result = wait_for_refresh(0)
        self.assertTrue(result.finished)
        self.assertFalse(result.success)
        self.assertEqual(len(result.errors), 1)

    @patch("abu.POLL_INTERVAL", 0.01)
    @patch("abu.read_new_log")
    def test_successful_compilation(self, mock_read: MagicMock) -> None:
        mock_read.return_value = (
            "[ScriptCompilation] Requested\n"
            "RefreshV2(NoUpdateAssetOptions)\n"
            "*** Tundra build success\n"
            "Reloading assemblies after finishing script compilation.\n"
            "StopAssetImportingV2\n"
            "Asset Pipeline Refresh (id=abc): Total: 2.1s\n"
        )
        result = wait_for_refresh(0)
        self.assertTrue(result.finished)
        self.assertTrue(result.success)

    @patch("abu.POLL_INTERVAL", 0.01)
    @patch("abu.read_new_log")
    def test_tundra_build_failed(self, mock_read: MagicMock) -> None:
        mock_read.return_value = (
            "[ScriptCompilation] Requested\n"
            "Tundra build failed\n"
            "Assets/Foo.cs(10,5): error CS1002: ; expected\n"
        )
        result = wait_for_refresh(0)
        self.assertTrue(result.finished)
        self.assertFalse(result.success)


class TestBuildParserUnity(unittest.TestCase):
    """Test argparse configuration for editor commands."""

    def test_refresh_defaults(self) -> None:
        parser = build_parser()
        args = parser.parse_args(["refresh"])
        self.assertEqual(args.command, "refresh")
        self.assertFalse(args.play)

    def test_refresh_with_play(self) -> None:
        parser = build_parser()
        args = parser.parse_args(["refresh", "--play"])
        self.assertEqual(args.command, "refresh")
        self.assertTrue(args.play)

    def test_play_command(self) -> None:
        parser = build_parser()
        args = parser.parse_args(["play"])
        self.assertEqual(args.command, "play")

    def test_no_command_fails(self) -> None:
        parser = build_parser()
        with self.assertRaises(SystemExit):
            parser.parse_args([])


class TestExceptionHierarchy(unittest.TestCase):
    """Test that all exceptions derive from AbuError."""

    def test_hammerspoon_error(self) -> None:
        self.assertIsInstance(HammerspoonError("test"), AbuError)

    def test_unity_not_found_error(self) -> None:
        self.assertIsInstance(UnityNotFoundError("test"), AbuError)

    def test_refresh_timeout_error(self) -> None:
        self.assertIsInstance(RefreshTimeoutError("test"), AbuError)

    def test_compilation_error(self) -> None:
        self.assertIsInstance(CompilationError("test"), AbuError)


class TestRefreshResult(unittest.TestCase):
    """Test RefreshResult dataclass."""

    def test_default_values(self) -> None:
        result = RefreshResult(finished=True, success=True)
        self.assertEqual(result.errors, [])
        self.assertEqual(result.summary, "")

    def test_with_errors(self) -> None:
        result = RefreshResult(
            finished=True,
            success=False,
            errors=["error CS1002: ; expected"],
            summary="1 compilation error(s)",
        )
        self.assertFalse(result.success)
        self.assertEqual(len(result.errors), 1)


class TestBuildParserStatus(unittest.TestCase):
    """Test argparse recognizes the status subcommand."""

    def test_status_command(self) -> None:
        parser = build_parser()
        args = parser.parse_args(["status"])
        self.assertEqual(args.command, "status")


class TestReadStateFile(unittest.TestCase):
    """Test state file reading."""

    @patch("abu.ABU_STATE_FILE")
    def test_returns_none_when_missing(self, mock_path: MagicMock) -> None:
        mock_path.read_text.side_effect = FileNotFoundError
        self.assertIsNone(read_state_file())

    @patch("abu.ABU_STATE_FILE")
    def test_returns_none_on_invalid_json(self, mock_path: MagicMock) -> None:
        mock_path.read_text.return_value = "not json"
        self.assertIsNone(read_state_file())

    @patch("abu.ABU_STATE_FILE")
    def test_returns_parsed_state(self, mock_path: MagicMock) -> None:
        state = {
            "version": 1,
            "playModeState": "EnteredPlayMode",
            "gameMode": "Battle",
            "unityPid": 12345,
            "timestampUtc": "2026-02-22T12:34:56Z",
        }
        mock_path.read_text.return_value = json.dumps(state)
        result = read_state_file()
        self.assertEqual(result, state)


class TestIsPidAlive(unittest.TestCase):
    """Test PID liveness checking."""

    def test_current_process_is_alive(self) -> None:
        self.assertTrue(is_pid_alive(os.getpid()))

    def test_nonexistent_pid(self) -> None:
        # PID 2^30 is almost certainly not running
        self.assertFalse(is_pid_alive(2**30))


class TestDoStatus(unittest.TestCase):
    """Test the do_status output."""

    @patch("abu.is_play_mode_active", return_value=False)
    @patch("abu.read_state_file", return_value=None)
    def test_no_state_file(self, _mock_read: MagicMock, _mock_tcp: MagicMock) -> None:
        import io
        from contextlib import redirect_stdout
        buf = io.StringIO()
        with redirect_stdout(buf):
            do_status()
        output = buf.getvalue()
        self.assertIn("State file:    not found", output)
        self.assertIn("Unity PID:     unknown", output)
        self.assertIn("TCP", output)

    @patch("abu.is_play_mode_active", return_value=True)
    @patch("abu.is_pid_alive", return_value=True)
    @patch("abu.read_state_file", return_value={
        "version": 1,
        "playModeState": "EnteredPlayMode",
        "gameMode": "Battle",
        "unityPid": 12345,
        "timestampUtc": "2026-02-22T12:34:56Z",
    })
    def test_active_play_mode(
        self, _mock_read: MagicMock, _mock_pid: MagicMock, _mock_tcp: MagicMock,
    ) -> None:
        import io
        from contextlib import redirect_stdout
        buf = io.StringIO()
        with redirect_stdout(buf):
            do_status()
        output = buf.getvalue()
        self.assertIn("State file:    ok", output)
        self.assertIn("12345 (running)", output)
        self.assertIn("Play mode:     active", output)
        self.assertIn("Game mode:     Battle", output)
        self.assertIn("reachable", output)

    @patch("abu.is_play_mode_active", return_value=False)
    @patch("abu.is_pid_alive", return_value=False)
    @patch("abu.read_state_file", return_value={
        "version": 1,
        "playModeState": "EnteredPlayMode",
        "gameMode": "Quest",
        "unityPid": 99999,
        "timestampUtc": "2026-02-22T12:00:00Z",
    })
    def test_stale_pid(
        self, _mock_read: MagicMock, _mock_pid: MagicMock, _mock_tcp: MagicMock,
    ) -> None:
        import io
        from contextlib import redirect_stdout
        buf = io.StringIO()
        with redirect_stdout(buf):
            do_status()
        output = buf.getvalue()
        self.assertIn("State file:    stale", output)
        self.assertIn("99999 (not running)", output)
        self.assertIn("unreachable", output)


class TestBuildParserRestart(unittest.TestCase):
    """Test argparse recognizes the restart subcommand."""

    def test_restart_command(self) -> None:
        parser = build_parser()
        args = parser.parse_args(["restart"])
        self.assertEqual(args.command, "restart")


class TestFindUnityProcess(unittest.TestCase):
    """Test Unity process discovery."""

    @patch("abu.subprocess.run")
    def test_finds_unity_process(self, mock_run: MagicMock) -> None:
        ps_list = subprocess.CompletedProcess(
            args=[], returncode=0,
            stdout=(
                "  PID COMM\n"
                "  123 /Applications/Unity/Hub/Editor/6000.1.3f1"
                "/Unity.app/Contents/MacOS/Unity\n"
            ),
            stderr="",
        )
        ps_args = subprocess.CompletedProcess(
            args=[], returncode=0,
            stdout=(
                "/Applications/Unity/Hub/Editor/6000.1.3f1"
                "/Unity.app/Contents/MacOS/Unity"
                " -projectPath /Users/me/project/client\n"
            ),
            stderr="",
        )
        mock_run.side_effect = [ps_list, ps_args]
        info = find_unity_process()
        self.assertEqual(info.pid, 123)
        self.assertIn("Unity.app/Contents/MacOS/Unity", info.executable)
        self.assertEqual(info.project_path, "/Users/me/project/client")

    @patch("abu.subprocess.run")
    def test_finds_unity_with_lowercase_projectpath(self, mock_run: MagicMock) -> None:
        ps_list = subprocess.CompletedProcess(
            args=[], returncode=0,
            stdout=(
                "  PID COMM\n"
                "  123 /Applications/Unity/Hub/Editor/6000.2.2f1"
                "/Unity.app/Contents/MacOS/Unity\n"
            ),
            stderr="",
        )
        ps_args = subprocess.CompletedProcess(
            args=[], returncode=0,
            stdout=(
                "/Applications/Unity/Hub/Editor/6000.2.2f1"
                "/Unity.app/Contents/MacOS/Unity"
                " -projectpath /Users/me/project/client\n"
            ),
            stderr="",
        )
        mock_run.side_effect = [ps_list, ps_args]
        info = find_unity_process()
        self.assertEqual(info.project_path, "/Users/me/project/client")

    @patch("abu.subprocess.run")
    def test_skips_batch_mode_workers(self, mock_run: MagicMock) -> None:
        ps_list = subprocess.CompletedProcess(
            args=[], returncode=0,
            stdout=(
                "  PID COMM\n"
                "  100 /Applications/Unity/Hub/Editor/6000.2.2f1"
                "/Unity.app/Contents/MacOS/Unity\n"
                "  200 /Applications/Unity/Hub/Editor/6000.2.2f1"
                "/Unity.app/Contents/MacOS/Unity\n"
            ),
            stderr="",
        )
        # First candidate is a batchMode worker
        ps_args_worker = subprocess.CompletedProcess(
            args=[], returncode=0,
            stdout=(
                "/Applications/Unity/Hub/Editor/6000.2.2f1"
                "/Unity.app/Contents/MacOS/Unity"
                " -batchMode -name AssetImportWorker0"
                " -projectPath /Users/me/project/client\n"
            ),
            stderr="",
        )
        # Second candidate is the main editor
        ps_args_main = subprocess.CompletedProcess(
            args=[], returncode=0,
            stdout=(
                "/Applications/Unity/Hub/Editor/6000.2.2f1"
                "/Unity.app/Contents/MacOS/Unity"
                " -projectpath /Users/me/project/client\n"
            ),
            stderr="",
        )
        mock_run.side_effect = [ps_list, ps_args_worker, ps_args_main]
        info = find_unity_process()
        self.assertEqual(info.pid, 200)

    @patch("abu.subprocess.run")
    def test_raises_when_no_unity(self, mock_run: MagicMock) -> None:
        ps_list = subprocess.CompletedProcess(
            args=[], returncode=0,
            stdout="  PID COMM\n  456 /usr/bin/python3\n",
            stderr="",
        )
        mock_run.return_value = ps_list
        with self.assertRaises(UnityNotFoundError):
            find_unity_process()

    @patch("abu.CLIENT_DIR", "/fallback/client")
    @patch("abu.subprocess.run")
    def test_falls_back_to_client_dir(self, mock_run: MagicMock) -> None:
        ps_list = subprocess.CompletedProcess(
            args=[], returncode=0,
            stdout=(
                "  PID COMM\n"
                "  789 /Applications/Unity/Hub/Editor/6000.1.3f1"
                "/Unity.app/Contents/MacOS/Unity\n"
            ),
            stderr="",
        )
        ps_args = subprocess.CompletedProcess(
            args=[], returncode=0,
            stdout=(
                "/Applications/Unity/Hub/Editor/6000.1.3f1"
                "/Unity.app/Contents/MacOS/Unity\n"
            ),
            stderr="",
        )
        mock_run.side_effect = [ps_list, ps_args]
        info = find_unity_process()
        self.assertEqual(info.pid, 789)
        self.assertEqual(info.project_path, "/fallback/client")


class TestResolveWorktreeName(unittest.TestCase):
    """Test worktree name resolution from repo root."""

    @patch("abu.MAIN_REPO_ROOT", Path("/Users/me/project"))
    @patch("abu.WORKTREE_BASE", Path("/Users/me/dreamtides-worktrees"))
    def test_main_repo_returns_none(self) -> None:
        self.assertIsNone(resolve_worktree_name())

    @patch("abu.MAIN_REPO_ROOT", Path("/Users/me/dreamtides-worktrees/alpha"))
    @patch("abu.WORKTREE_BASE", Path("/Users/me/dreamtides-worktrees"))
    def test_worktree_returns_name(self) -> None:
        self.assertEqual(resolve_worktree_name(), "alpha")

    @patch("abu.MAIN_REPO_ROOT", Path("/Users/me/dreamtides-worktrees/beta/nested"))
    @patch("abu.WORKTREE_BASE", Path("/Users/me/dreamtides-worktrees"))
    def test_nested_worktree_returns_top_level_name(self) -> None:
        self.assertEqual(resolve_worktree_name(), "beta")


class TestResolvePort(unittest.TestCase):
    """Test port resolution: env var > worktree .ports.json > default."""

    @patch("abu.resolve_worktree_name", return_value=None)
    def test_main_repo_returns_default(self, _mock: MagicMock) -> None:
        with patch.dict(os.environ, {}, clear=True):
            self.assertEqual(resolve_port(), DEFAULT_ABU_PORT)

    @patch("abu.resolve_worktree_name", return_value=None)
    def test_env_var_overrides(self, _mock: MagicMock) -> None:
        with patch.dict(os.environ, {"ABU_PORT": "8888"}):
            self.assertEqual(resolve_port(), 8888)

    @patch("abu.PORTS_FILE")
    @patch("abu.resolve_worktree_name", return_value="alpha")
    def test_worktree_reads_ports_file(self, _mock_name: MagicMock, mock_ports_file: MagicMock) -> None:
        mock_ports_file.read_text.return_value = json.dumps({"alpha": 10000})
        with patch.dict(os.environ, {}, clear=True):
            self.assertEqual(resolve_port(), 10000)


class TestSendMenuItemPidTargeted(unittest.TestCase):
    """Test PID-targeted vs bundle-ID Hammerspoon dispatch."""

    @patch("abu.run_hs", return_value="OK: Selected Assets > Refresh (pid 123)")
    @patch("abu.is_pid_alive", return_value=True)
    @patch("abu.read_state_file", return_value={"unityPid": 123})
    def test_valid_pid_uses_application_for_pid(
        self, _mock_state: MagicMock, _mock_alive: MagicMock, mock_hs: MagicMock,
    ) -> None:
        result = send_menu_item(["Assets", "Refresh"])
        self.assertIn("OK", result)
        lua_code = mock_hs.call_args[0][0]
        self.assertIn("applicationForPID(123)", lua_code)

    @patch("abu.run_hs", return_value="OK: Selected Assets > Refresh (pid 456)")
    @patch("abu.read_state_file", return_value=None)
    def test_missing_state_falls_back_to_bundle_id(
        self, _mock_state: MagicMock, mock_hs: MagicMock,
    ) -> None:
        result = send_menu_item(["Assets", "Refresh"])
        self.assertIn("OK", result)
        lua_code = mock_hs.call_args[0][0]
        self.assertIn("hs.application.find", lua_code)

    @patch("abu.run_hs", return_value="OK: Selected Assets > Refresh (pid 456)")
    @patch("abu.is_pid_alive", return_value=False)
    @patch("abu.read_state_file", return_value={"unityPid": 999})
    def test_stale_pid_falls_back_to_bundle_id(
        self, _mock_state: MagicMock, _mock_alive: MagicMock, mock_hs: MagicMock,
    ) -> None:
        result = send_menu_item(["Assets", "Refresh"])
        self.assertIn("OK", result)
        lua_code = mock_hs.call_args[0][0]
        self.assertIn("hs.application.find", lua_code)


class TestCheckLogConflict(unittest.TestCase):
    """Test log conflict detection between multiple editors."""

    @patch("abu.is_pid_alive", return_value=True)
    @patch("abu.all_state_files")
    def test_detects_shared_log_file(
        self, mock_files: MagicMock, _mock_alive: MagicMock,
    ) -> None:
        import tempfile
        with tempfile.TemporaryDirectory() as tmpdir:
            state1 = Path(tmpdir) / "main" / ".abu-state.json"
            state2 = Path(tmpdir) / "wt" / ".abu-state.json"
            state1.parent.mkdir()
            state2.parent.mkdir()
            state1.write_text(json.dumps({
                "unityPid": 100,
                "logFile": "/shared/Editor.log",
            }))
            state2.write_text(json.dumps({
                "unityPid": 200,
                "logFile": "/shared/Editor.log",
            }))
            mock_files.return_value = [state1, state2]
            with self.assertRaises(SystemExit):
                check_log_conflict()

    @patch("abu.is_pid_alive", return_value=True)
    @patch("abu.all_state_files")
    def test_passes_with_distinct_logs(
        self, mock_files: MagicMock, _mock_alive: MagicMock,
    ) -> None:
        import tempfile
        with tempfile.TemporaryDirectory() as tmpdir:
            state1 = Path(tmpdir) / "main" / ".abu-state.json"
            state2 = Path(tmpdir) / "wt" / ".abu-state.json"
            state1.parent.mkdir()
            state2.parent.mkdir()
            state1.write_text(json.dumps({
                "unityPid": 100,
                "logFile": "/main/Editor.log",
            }))
            state2.write_text(json.dumps({
                "unityPid": 200,
                "logFile": "/wt/Editor.log",
            }))
            mock_files.return_value = [state1, state2]
            check_log_conflict()  # Should not raise


class TestBuildParserOpen(unittest.TestCase):
    """Test argparse recognizes the open subcommand."""

    def test_open_command(self) -> None:
        parser = build_parser()
        args = parser.parse_args(["open", "alpha"])
        self.assertEqual(args.command, "open")
        self.assertEqual(args.name, "alpha")

    def test_open_missing_name_fails(self) -> None:
        parser = build_parser()
        with self.assertRaises(SystemExit):
            parser.parse_args(["open"])


class TestFindUnityExecutable(unittest.TestCase):
    """Test Unity executable discovery from ProjectVersion.txt."""

    def test_finds_unity_app(self) -> None:
        import tempfile
        with tempfile.TemporaryDirectory() as tmpdir:
            client = Path(tmpdir)
            settings = client / "ProjectSettings"
            settings.mkdir()
            (settings / "ProjectVersion.txt").write_text(
                "m_EditorVersion: 6000.2.2f1\n"
                "m_EditorVersionWithRevision: 6000.2.2f1 (ea398eefe1c2)\n"
            )
            app_path = Path(f"/Applications/Unity/Hub/Editor/6000.2.2f1/Unity.app")
            with patch.object(Path, "exists", return_value=True):
                version, result_path = find_unity_executable(client)
            self.assertEqual(version, "6000.2.2f1")
            self.assertEqual(result_path, app_path)

    def test_raises_when_version_file_missing(self) -> None:
        import tempfile
        with tempfile.TemporaryDirectory() as tmpdir:
            client = Path(tmpdir)
            with self.assertRaises(AbuError) as ctx:
                find_unity_executable(client)
            self.assertIn("ProjectVersion.txt not found", str(ctx.exception))

    def test_raises_when_unity_not_installed(self) -> None:
        import tempfile
        with tempfile.TemporaryDirectory() as tmpdir:
            client = Path(tmpdir)
            settings = client / "ProjectSettings"
            settings.mkdir()
            (settings / "ProjectVersion.txt").write_text(
                "m_EditorVersion: 9999.0.0f1\n"
            )
            with self.assertRaises(AbuError) as ctx:
                find_unity_executable(client)
            self.assertIn("not found at", str(ctx.exception))


class TestDoOpen(unittest.TestCase):
    """Test do_open worktree validation."""

    @patch("abu.WORKTREE_BASE", Path("/nonexistent/path"))
    def test_raises_when_worktree_missing(self) -> None:
        with self.assertRaises(AbuError) as ctx:
            do_open("nosuch")
        self.assertIn("not found", str(ctx.exception))

    @patch("abu.subprocess.Popen")
    @patch("abu.find_unity_executable")
    @patch("abu.PORTS_FILE")
    @patch("abu.WORKTREE_BASE")
    def test_launches_unity(
        self,
        mock_base: MagicMock,
        mock_ports: MagicMock,
        mock_find: MagicMock,
        mock_popen: MagicMock,
    ) -> None:
        import tempfile
        with tempfile.TemporaryDirectory() as tmpdir:
            wt_root = Path(tmpdir) / "alpha"
            client = wt_root / "client"
            client.mkdir(parents=True)
            mock_base.__truediv__ = lambda self, key: Path(tmpdir) / key
            mock_base.return_value = Path(tmpdir)
            mock_find.return_value = ("6000.2.2f1", Path("/Applications/Unity/Hub/Editor/6000.2.2f1/Unity.app"))
            mock_ports.read_text.return_value = json.dumps({"alpha": 10001})
            do_open("alpha")
            mock_popen.assert_called_once()
            call_args = mock_popen.call_args[0][0]
            self.assertIn("-logFile", call_args)
            self.assertIn("-projectPath", call_args)


if __name__ == "__main__":
    unittest.main()
