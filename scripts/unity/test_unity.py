#!/usr/bin/env python3
"""Tests for unity.py CLI functions."""

import subprocess
import unittest
from unittest.mock import MagicMock, patch

from unity import (
    CompilationError,
    HammerspoonError,
    RefreshResult,
    RefreshTimeoutError,
    UnityError,
    UnityNotFoundError,
    _report_result,
    build_parser,
    is_worktree,
    wait_for_refresh,
)


class TestIsWorktree(unittest.TestCase):
    """Test git worktree detection."""

    @patch("unity.subprocess.run")
    def test_main_repo_not_worktree(self, mock_run: MagicMock) -> None:
        mock_run.return_value = subprocess.CompletedProcess(
            args=[], returncode=0, stdout=".git\n", stderr=""
        )
        self.assertFalse(is_worktree())

    @patch("unity.subprocess.run")
    def test_worktree_detected(self, mock_run: MagicMock) -> None:
        mock_run.return_value = subprocess.CompletedProcess(
            args=[], returncode=0, stdout="/path/to/main/.git\n", stderr=""
        )
        self.assertTrue(is_worktree())

    @patch("unity.subprocess.run")
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

    @patch("unity.TIMEOUT_SECONDS", 0.5)
    @patch("unity.POLL_INTERVAL", 0.1)
    @patch("unity.read_new_log")
    def test_timeout_returns_not_finished(self, mock_read: MagicMock) -> None:
        mock_read.return_value = ""
        result = wait_for_refresh(0)
        self.assertFalse(result.finished)
        self.assertFalse(result.success)

    @patch("unity.POLL_INTERVAL", 0.01)
    @patch("unity.read_new_log")
    def test_no_compilation_refresh_completes(self, mock_read: MagicMock) -> None:
        mock_read.return_value = (
            "RefreshV2(NoUpdateAssetOptions)\n"
            "Asset Pipeline Refresh (id=abc): Total: 0.5s\n"
        )
        result = wait_for_refresh(0)
        self.assertTrue(result.finished)
        self.assertTrue(result.success)

    @patch("unity.POLL_INTERVAL", 0.01)
    @patch("unity.read_new_log")
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

    @patch("unity.POLL_INTERVAL", 0.01)
    @patch("unity.read_new_log")
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

    @patch("unity.POLL_INTERVAL", 0.01)
    @patch("unity.read_new_log")
    def test_tundra_build_failed(self, mock_read: MagicMock) -> None:
        mock_read.return_value = (
            "[ScriptCompilation] Requested\n"
            "Tundra build failed\n"
            "Assets/Foo.cs(10,5): error CS1002: ; expected\n"
        )
        result = wait_for_refresh(0)
        self.assertTrue(result.finished)
        self.assertFalse(result.success)


class TestBuildParser(unittest.TestCase):
    """Test argparse configuration."""

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
    """Test that all exceptions derive from UnityError."""

    def test_hammerspoon_error(self) -> None:
        self.assertIsInstance(HammerspoonError("test"), UnityError)

    def test_unity_not_found_error(self) -> None:
        self.assertIsInstance(UnityNotFoundError("test"), UnityError)

    def test_refresh_timeout_error(self) -> None:
        self.assertIsInstance(RefreshTimeoutError("test"), UnityError)

    def test_compilation_error(self) -> None:
        self.assertIsInstance(CompilationError("test"), UnityError)


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
