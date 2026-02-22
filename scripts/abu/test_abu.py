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
from unittest.mock import MagicMock, patch

from abu import (
    AbuError,
    CompilationError,
    ConnectionError,
    EmptyResponseError,
    HammerspoonError,
    RefreshResult,
    RefreshTimeoutError,
    UnityNotFoundError,
    _report_result,
    build_command,
    build_params,
    build_parser,
    handle_response,
    is_worktree,
    send_command,
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
            args=[], returncode=0, stdout=".git\n", stderr=""
        )
        self.assertFalse(is_worktree())

    @patch("abu.subprocess.run")
    def test_worktree_detected(self, mock_run: MagicMock) -> None:
        mock_run.return_value = subprocess.CompletedProcess(
            args=[], returncode=0, stdout="/path/to/main/.git\n", stderr=""
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


if __name__ == "__main__":
    unittest.main()
