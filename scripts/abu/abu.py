#!/usr/bin/env python3
"""Communicate with Unity over TCP using the abu NDJSON protocol.

Provides subcommands for all abu operations: snapshot, click, hover, drag,
screenshot, launch, and close. Connects to Unity on localhost via a one-shot
TCP connection, sends one NDJSON command, reads one NDJSON response, and exits.
"""

import argparse
import base64
import json
import os
import socket
import sys
import tempfile
import uuid
from typing import Any

# Save builtins before shadowing with domain-specific subclasses.
builtins_ConnectionRefusedError = ConnectionRefusedError
builtins_TimeoutError = TimeoutError


class AbuError(Exception):
    """Raised when an abu command fails."""


class ConnectionError(AbuError):
    """Raised when the CLI cannot connect to Unity."""


class TimeoutError(AbuError):
    """Raised when waiting for a Unity response times out."""


class EmptyResponseError(AbuError):
    """Raised when Unity closes the connection without a response."""


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

    # screenshot, launch, close
    return {}


def build_command(command: str, params: dict[str, Any]) -> str:
    """Build an NDJSON command line to send to Unity."""
    message = {
        "id": str(uuid.uuid4()),
        "command": command,
        "params": params,
    }
    return json.dumps(message) + "\n"


def handle_response(command: str, response: dict[str, Any]) -> str:
    """Process a response from Unity and return the output string.

    For most commands, returns the JSON-serialized data object.
    For screenshot, decodes base64 data and writes to a temp file,
    returning the file path. Raises AbuError on error responses.
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


def build_parser() -> argparse.ArgumentParser:
    """Build the argument parser with all subcommands."""
    parser = argparse.ArgumentParser(
        prog="abu.py",
        description="Communicate with Unity over TCP using the abu NDJSON protocol.",
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

    # launch
    subparsers.add_parser("launch", help="Launch the game")

    # close
    subparsers.add_parser("close", help="Close the game")

    return parser


def main() -> None:
    parser = build_parser()
    args = parser.parse_args()
    command: str = args.command
    params = build_params(args)
    port = int(os.environ.get("ABU_PORT", "9999"))

    try:
        response = send_command(command, params, port)
        output = handle_response(command, response)
    except AbuError as e:
        print(f"Error: {e}", file=sys.stderr)
        sys.exit(1)

    print(output)


if __name__ == "__main__":
    main()
