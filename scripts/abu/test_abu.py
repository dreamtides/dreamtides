#!/usr/bin/env python3
"""Tests for abu.py CLI functions."""

import json
import socket
import threading
import unittest


class TestStripRef(unittest.TestCase):
    """Test the @ prefix stripping from ref arguments."""

    def test_strips_at_prefix(self) -> None:
        from abu import strip_ref
        self.assertEqual(strip_ref("@e1"), "e1")

    def test_leaves_plain_ref(self) -> None:
        from abu import strip_ref
        self.assertEqual(strip_ref("e1"), "e1")

    def test_strips_only_leading_at(self) -> None:
        from abu import strip_ref
        self.assertEqual(strip_ref("@e3"), "e3")

    def test_empty_after_at(self) -> None:
        from abu import strip_ref
        self.assertEqual(strip_ref("@"), "")


class TestBuildParams(unittest.TestCase):
    """Test params construction for each command type."""

    def test_snapshot_no_flags(self) -> None:
        from abu import build_params
        import argparse
        ns = argparse.Namespace(command="snapshot", compact=False, interactive=False, max_depth=None)
        self.assertEqual(build_params(ns), {})

    def test_snapshot_compact(self) -> None:
        from abu import build_params
        import argparse
        ns = argparse.Namespace(command="snapshot", compact=True, interactive=False, max_depth=None)
        self.assertEqual(build_params(ns), {"compact": True})

    def test_snapshot_interactive(self) -> None:
        from abu import build_params
        import argparse
        ns = argparse.Namespace(command="snapshot", compact=False, interactive=True, max_depth=None)
        self.assertEqual(build_params(ns), {"interactive": True})

    def test_snapshot_max_depth(self) -> None:
        from abu import build_params
        import argparse
        ns = argparse.Namespace(command="snapshot", compact=False, interactive=False, max_depth=5)
        self.assertEqual(build_params(ns), {"maxDepth": 5})

    def test_snapshot_all_flags(self) -> None:
        from abu import build_params
        import argparse
        ns = argparse.Namespace(command="snapshot", compact=True, interactive=True, max_depth=3)
        self.assertEqual(build_params(ns), {"compact": True, "interactive": True, "maxDepth": 3})

    def test_click_params(self) -> None:
        from abu import build_params
        import argparse
        ns = argparse.Namespace(command="click", ref="@e1")
        self.assertEqual(build_params(ns), {"ref": "e1"})

    def test_hover_params(self) -> None:
        from abu import build_params
        import argparse
        ns = argparse.Namespace(command="hover", ref="e2")
        self.assertEqual(build_params(ns), {"ref": "e2"})

    def test_drag_with_target(self) -> None:
        from abu import build_params
        import argparse
        ns = argparse.Namespace(command="drag", source="@e1", target="@e2")
        self.assertEqual(build_params(ns), {"source": "e1", "target": "e2"})

    def test_drag_without_target(self) -> None:
        from abu import build_params
        import argparse
        ns = argparse.Namespace(command="drag", source="e1", target=None)
        self.assertEqual(build_params(ns), {"source": "e1"})

    def test_screenshot_params(self) -> None:
        from abu import build_params
        import argparse
        ns = argparse.Namespace(command="screenshot")
        self.assertEqual(build_params(ns), {})

    def test_launch_params(self) -> None:
        from abu import build_params
        import argparse
        ns = argparse.Namespace(command="launch")
        self.assertEqual(build_params(ns), {})

    def test_close_params(self) -> None:
        from abu import build_params
        import argparse
        ns = argparse.Namespace(command="close")
        self.assertEqual(build_params(ns), {})


class TestBuildCommand(unittest.TestCase):
    """Test NDJSON command construction."""

    def test_command_structure(self) -> None:
        from abu import build_command
        cmd = build_command("snapshot", {"compact": True})
        parsed = json.loads(cmd)
        self.assertIn("id", parsed)
        self.assertEqual(parsed["command"], "snapshot")
        self.assertEqual(parsed["params"], {"compact": True})
        self.assertTrue(cmd.endswith("\n"))

    def test_command_has_uuid(self) -> None:
        from abu import build_command
        import uuid
        cmd = build_command("click", {"ref": "e1"})
        parsed = json.loads(cmd)
        # Should be a valid UUID
        uuid.UUID(parsed["id"])


class TestHandleResponse(unittest.TestCase):
    """Test response handling for each command type."""

    def test_snapshot_response(self) -> None:
        from abu import handle_response
        response = {
            "id": "test-id",
            "success": True,
            "data": {
                "snapshot": "- application \"Dreamtides\"",
                "refs": {"e1": {"role": "button", "name": "End Turn"}},
            },
        }
        result = handle_response("snapshot", response)
        self.assertIsInstance(result, str)
        parsed = json.loads(result)
        self.assertEqual(parsed["snapshot"], "- application \"Dreamtides\"")
        self.assertIn("e1", parsed["refs"])

    def test_click_response(self) -> None:
        from abu import handle_response
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
        parsed = json.loads(result)
        self.assertTrue(parsed["clicked"])

    def test_screenshot_response(self) -> None:
        import base64
        from abu import handle_response
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
        import os
        self.assertTrue(os.path.exists(result))
        with open(result, "rb") as f:
            self.assertEqual(f.read(), png_bytes)

    def test_launch_response(self) -> None:
        from abu import handle_response
        response = {
            "id": "test-id",
            "success": True,
            "data": {"launched": True},
        }
        result = handle_response("launch", response)
        parsed = json.loads(result)
        self.assertTrue(parsed["launched"])

    def test_close_response(self) -> None:
        from abu import handle_response
        response = {
            "id": "test-id",
            "success": True,
            "data": {"closed": True},
        }
        result = handle_response("close", response)
        parsed = json.loads(result)
        self.assertTrue(parsed["closed"])

    def test_error_response_raises(self) -> None:
        from abu import handle_response, AbuError
        response = {
            "id": "test-id",
            "success": False,
            "error": "Something went wrong",
        }
        with self.assertRaises(AbuError) as ctx:
            handle_response("snapshot", response)
        self.assertIn("Something went wrong", str(ctx.exception))


class TestBuildParser(unittest.TestCase):
    """Test argparse configuration."""

    def test_snapshot_defaults(self) -> None:
        from abu import build_parser
        parser = build_parser()
        args = parser.parse_args(["snapshot"])
        self.assertEqual(args.command, "snapshot")
        self.assertFalse(args.compact)
        self.assertFalse(args.interactive)
        self.assertIsNone(args.max_depth)

    def test_snapshot_all_flags(self) -> None:
        from abu import build_parser
        parser = build_parser()
        args = parser.parse_args(["snapshot", "--compact", "--interactive", "--max-depth", "5"])
        self.assertTrue(args.compact)
        self.assertTrue(args.interactive)
        self.assertEqual(args.max_depth, 5)

    def test_click_ref(self) -> None:
        from abu import build_parser
        parser = build_parser()
        args = parser.parse_args(["click", "@e1"])
        self.assertEqual(args.command, "click")
        self.assertEqual(args.ref, "@e1")

    def test_hover_ref(self) -> None:
        from abu import build_parser
        parser = build_parser()
        args = parser.parse_args(["hover", "e2"])
        self.assertEqual(args.command, "hover")
        self.assertEqual(args.ref, "e2")

    def test_drag_with_target(self) -> None:
        from abu import build_parser
        parser = build_parser()
        args = parser.parse_args(["drag", "@e1", "@e2"])
        self.assertEqual(args.command, "drag")
        self.assertEqual(args.source, "@e1")
        self.assertEqual(args.target, "@e2")

    def test_drag_without_target(self) -> None:
        from abu import build_parser
        parser = build_parser()
        args = parser.parse_args(["drag", "e1"])
        self.assertEqual(args.command, "drag")
        self.assertEqual(args.source, "e1")
        self.assertIsNone(args.target)

    def test_screenshot(self) -> None:
        from abu import build_parser
        parser = build_parser()
        args = parser.parse_args(["screenshot"])
        self.assertEqual(args.command, "screenshot")

    def test_launch(self) -> None:
        from abu import build_parser
        parser = build_parser()
        args = parser.parse_args(["launch"])
        self.assertEqual(args.command, "launch")

    def test_close(self) -> None:
        from abu import build_parser
        parser = build_parser()
        args = parser.parse_args(["close"])
        self.assertEqual(args.command, "close")


class TestSendCommand(unittest.TestCase):
    """Test TCP communication with a mock Unity server."""

    def _start_mock_server(self, response_data: dict) -> tuple[int, threading.Event]:
        """Start a TCP server that returns a canned NDJSON response."""
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
                # Send the response
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
        from abu import send_command
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
        from abu import send_command
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
        from abu import send_command
        # Use a port that nothing is listening on
        with self.assertRaises(SystemExit) as ctx:
            send_command("snapshot", {}, 1)
        self.assertEqual(ctx.exception.code, 1)

    def test_send_command_sends_valid_ndjson(self) -> None:
        """Verify the server receives valid NDJSON with the expected fields."""
        received_data: dict = {}
        server = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        server.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
        server.bind(("localhost", 0))
        server.listen(1)
        port = server.getsockname()[1]

        def serve() -> None:
            conn, _ = server.accept()
            data = b""
            while b"\n" not in data:
                chunk = conn.recv(4096)
                if not chunk:
                    break
                data += chunk
            line = data.split(b"\n", 1)[0]
            received_data.update(json.loads(line.decode("utf-8")))
            response = json.dumps({"id": "x", "success": True, "data": {}}).encode() + b"\n"
            conn.sendall(response)
            conn.close()
            server.close()

        t = threading.Thread(target=serve, daemon=True)
        t.start()

        from abu import send_command
        send_command("click", {"ref": "e1"}, port)
        t.join(timeout=5)

        self.assertIn("id", received_data)
        self.assertEqual(received_data["command"], "click")
        self.assertEqual(received_data["params"], {"ref": "e1"})


    def test_send_command_empty_response(self) -> None:
        """Server accepts connection but sends nothing and closes."""
        server = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        server.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
        server.bind(("localhost", 0))
        server.listen(1)
        port = server.getsockname()[1]

        def serve() -> None:
            conn, _ = server.accept()
            # Read the command but send nothing back
            conn.recv(4096)
            conn.close()
            server.close()

        t = threading.Thread(target=serve, daemon=True)
        t.start()

        from abu import send_command
        with self.assertRaises(SystemExit) as ctx:
            send_command("snapshot", {}, port)
        self.assertEqual(ctx.exception.code, 1)
        t.join(timeout=5)


class TestHoverDragResponses(unittest.TestCase):
    """Test handle_response for hover and drag commands."""

    def test_hover_response(self) -> None:
        from abu import handle_response
        response = {
            "id": "test-id",
            "success": True,
            "data": {"hovered": True, "snapshot": "- ui", "refs": {}},
        }
        result = handle_response("hover", response)
        parsed = json.loads(result)
        self.assertTrue(parsed["hovered"])
        self.assertEqual(parsed["snapshot"], "- ui")

    def test_drag_response(self) -> None:
        from abu import handle_response
        response = {
            "id": "test-id",
            "success": True,
            "data": {"dragged": True, "snapshot": "- ui", "refs": {}},
        }
        result = handle_response("drag", response)
        parsed = json.loads(result)
        self.assertTrue(parsed["dragged"])


if __name__ == "__main__":
    unittest.main()
