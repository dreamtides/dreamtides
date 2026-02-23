#!/usr/bin/env python3
"""Tests for worktree.py module."""

import argparse
import json
import os
import tempfile
import unittest
from pathlib import Path
from unittest.mock import MagicMock, patch

from worktree import (
    DEFAULT_WORKTREE_BASE,
    EXCLUDE,
    FIRST_WORKTREE_PORT,
    POOL_SLOTS,
    _is_worktree_available,
    _worktree_branch,
    allocate_port,
    cmd_claim,
    cmd_reset,
    deallocate_port,
    dispatch,
    read_ports,
    register_subcommands,
    resolve_worktree_path,
    should_exclude,
    write_ports,
)


class TestShouldExclude(unittest.TestCase):
    """Test the exclusion filter for untracked items."""

    def test_excludes_ds_store(self) -> None:
        self.assertTrue(should_exclude(".DS_Store"))

    def test_excludes_pycache(self) -> None:
        self.assertTrue(should_exclude("__pycache__"))

    def test_excludes_nested_pycache(self) -> None:
        self.assertTrue(should_exclude("scripts/review/__pycache__"))

    def test_excludes_client_temp(self) -> None:
        self.assertTrue(should_exclude("client/Temp"))

    def test_excludes_client_temp_subdir(self) -> None:
        self.assertTrue(should_exclude("client/Temp/some_file"))

    def test_excludes_client_logs(self) -> None:
        self.assertTrue(should_exclude("client/Logs"))

    def test_excludes_glob_mm_profdata(self) -> None:
        self.assertTrue(should_exclude("something.mm_profdata"))

    def test_excludes_glob_csproj(self) -> None:
        self.assertTrue(should_exclude("MyProject.csproj"))

    def test_excludes_glob_sln(self) -> None:
        self.assertTrue(should_exclude("Solution.sln"))

    def test_excludes_abu_state(self) -> None:
        self.assertTrue(should_exclude(".abu-state.json"))

    def test_does_not_exclude_regular_dir(self) -> None:
        self.assertFalse(should_exclude("rules_engine/target"))

    def test_does_not_exclude_regular_file(self) -> None:
        self.assertFalse(should_exclude("client/Library/some_file"))

    def test_trailing_slash_stripped(self) -> None:
        self.assertTrue(should_exclude("client/Temp/"))

    def test_excludes_venv(self) -> None:
        self.assertTrue(should_exclude(".venv"))

    def test_excludes_tmp(self) -> None:
        self.assertTrue(should_exclude("tmp"))

    def test_excludes_nested_tmp(self) -> None:
        self.assertTrue(should_exclude("foo/tmp"))


class TestPortAllocation(unittest.TestCase):
    """Test port allocation and deallocation."""

    def setUp(self) -> None:
        self.tmpdir = tempfile.mkdtemp()
        self.ports_file = Path(self.tmpdir) / ".ports.json"

    def tearDown(self) -> None:
        import shutil
        shutil.rmtree(self.tmpdir, ignore_errors=True)

    @patch("worktree.PORTS_FILE")
    def test_read_ports_empty(self, mock_file: MagicMock) -> None:
        mock_file.read_text.side_effect = FileNotFoundError
        self.assertEqual(read_ports(), {})

    @patch("worktree.PORTS_FILE")
    def test_read_ports_invalid_json(self, mock_file: MagicMock) -> None:
        mock_file.read_text.return_value = "not json"
        self.assertEqual(read_ports(), {})

    @patch("worktree.PORTS_FILE")
    def test_read_ports_valid(self, mock_file: MagicMock) -> None:
        mock_file.read_text.return_value = json.dumps({"alpha": 10000})
        self.assertEqual(read_ports(), {"alpha": 10000})

    @patch("worktree.write_ports")
    @patch("worktree.read_ports")
    def test_allocate_port_new(self, mock_read: MagicMock, mock_write: MagicMock) -> None:
        mock_read.return_value = {}
        port = allocate_port("alpha")
        self.assertEqual(port, FIRST_WORKTREE_PORT)
        mock_write.assert_called_once_with({"alpha": FIRST_WORKTREE_PORT})

    @patch("worktree.write_ports")
    @patch("worktree.read_ports")
    def test_allocate_port_existing(self, mock_read: MagicMock, mock_write: MagicMock) -> None:
        mock_read.return_value = {"alpha": 10005}
        port = allocate_port("alpha")
        self.assertEqual(port, 10005)
        mock_write.assert_not_called()

    @patch("worktree.write_ports")
    @patch("worktree.read_ports")
    def test_allocate_port_skips_used(self, mock_read: MagicMock, mock_write: MagicMock) -> None:
        mock_read.return_value = {"alpha": 10000, "beta": 10001}
        port = allocate_port("gamma")
        self.assertEqual(port, 10002)
        mock_write.assert_called_once()

    @patch("worktree.write_ports")
    @patch("worktree.read_ports")
    def test_deallocate_port_exists(self, mock_read: MagicMock, mock_write: MagicMock) -> None:
        mock_read.return_value = {"alpha": 10000, "beta": 10001}
        deallocate_port("alpha")
        mock_write.assert_called_once_with({"beta": 10001})

    @patch("worktree.write_ports")
    @patch("worktree.read_ports")
    def test_deallocate_port_not_found(self, mock_read: MagicMock, mock_write: MagicMock) -> None:
        mock_read.return_value = {"alpha": 10000}
        deallocate_port("gamma")
        mock_write.assert_not_called()


class TestResolveWorktreePath(unittest.TestCase):
    """Test worktree path resolution."""

    def test_none_returns_cwd(self) -> None:
        result = resolve_worktree_path(None)
        self.assertEqual(result, Path.cwd().resolve())

    def test_existing_path_returned(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            result = resolve_worktree_path(tmpdir)
            self.assertEqual(result, Path(tmpdir).resolve())

    @patch("worktree.DEFAULT_WORKTREE_BASE")
    def test_branch_name_resolved(self, mock_base: MagicMock) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            wt_dir = Path(tmpdir) / "alpha"
            wt_dir.mkdir()
            mock_base.__truediv__ = lambda self, key: Path(tmpdir) / key
            result = resolve_worktree_path("alpha")
            self.assertEqual(result, wt_dir)

    @patch("worktree.DEFAULT_WORKTREE_BASE", Path("/nonexistent/base"))
    def test_not_found_exits(self) -> None:
        with self.assertRaises(SystemExit):
            resolve_worktree_path("nosuch")


class TestWorktreeBranch(unittest.TestCase):
    """Test _worktree_branch helper."""

    @patch("worktree.run_cmd")
    def test_finds_branch(self, mock_run: MagicMock) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            wt_path = Path(tmpdir) / "alpha"
            wt_path.mkdir()
            mock_run.return_value = MagicMock(
                stdout=f"worktree {wt_path}\nbranch refs/heads/my-feature\n\n",
            )
            result = _worktree_branch(wt_path)
            self.assertEqual(result, "my-feature")

    @patch("worktree.run_cmd")
    def test_returns_none_for_detached(self, mock_run: MagicMock) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            wt_path = Path(tmpdir) / "alpha"
            wt_path.mkdir()
            mock_run.return_value = MagicMock(
                stdout=f"worktree {wt_path}\nHEAD abc123\ndetached\n\n",
            )
            result = _worktree_branch(wt_path)
            self.assertIsNone(result)

    @patch("worktree.run_cmd")
    def test_returns_none_for_unknown(self, mock_run: MagicMock) -> None:
        mock_run.return_value = MagicMock(
            stdout="worktree /some/other/path\nbranch refs/heads/main\n\n",
        )
        with tempfile.TemporaryDirectory() as tmpdir:
            result = _worktree_branch(Path(tmpdir) / "nonexistent")
            self.assertIsNone(result)


class TestIsWorktreeAvailable(unittest.TestCase):
    """Test _is_worktree_available helper."""

    @patch("worktree.run_cmd")
    def test_clean_and_merged(self, mock_run: MagicMock) -> None:
        mock_run.side_effect = [
            MagicMock(returncode=0),  # merge-base --is-ancestor succeeds
            MagicMock(stdout=""),  # git status --porcelain is clean
        ]
        self.assertTrue(_is_worktree_available(Path("/tmp/wt"), "master"))

    @patch("worktree.run_cmd")
    def test_dirty_tracked_files(self, mock_run: MagicMock) -> None:
        mock_run.side_effect = [
            MagicMock(returncode=0),  # merged
            MagicMock(stdout=" M some_file.py\n"),  # dirty
        ]
        self.assertFalse(_is_worktree_available(Path("/tmp/wt"), "master"))

    @patch("worktree.run_cmd")
    def test_not_merged(self, mock_run: MagicMock) -> None:
        mock_run.side_effect = [
            MagicMock(returncode=1),  # not ancestor
        ]
        self.assertFalse(_is_worktree_available(Path("/tmp/wt"), "master"))

    @patch("worktree.run_cmd")
    def test_untracked_only_is_available(self, mock_run: MagicMock) -> None:
        mock_run.side_effect = [
            MagicMock(returncode=0),  # merged
            MagicMock(stdout="?? untracked_file.txt\n?? another.log\n"),  # untracked only
        ]
        self.assertTrue(_is_worktree_available(Path("/tmp/wt"), "master"))


class TestCmdClaim(unittest.TestCase):
    """Test cmd_claim command."""

    def _make_args(self, branch: str = "feat", base: str = "master") -> argparse.Namespace:
        return argparse.Namespace(branch=branch, base=base)

    @patch("worktree._claim_reuse")
    @patch("worktree._is_worktree_available")
    @patch("worktree._worktree_branch")
    @patch("worktree.get_free_gb", return_value=50.0)
    @patch("worktree.verify_apfs", return_value=True)
    @patch("worktree.DEFAULT_WORKTREE_BASE")
    def test_reuses_available_slot(
        self,
        mock_base: MagicMock,
        mock_apfs: MagicMock,
        mock_free: MagicMock,
        mock_branch: MagicMock,
        mock_available: MagicMock,
        mock_reuse: MagicMock,
    ) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            base_path = Path(tmpdir)
            alpha_path = base_path / "alpha"
            alpha_path.mkdir()
            mock_base.__truediv__ = lambda self, key: base_path / key
            mock_branch.return_value = "old-branch"
            mock_available.return_value = True

            with patch("builtins.print") as mock_print:
                cmd_claim(self._make_args())
                mock_reuse.assert_called_once_with(alpha_path, "alpha", "feat", "master")
                mock_print.assert_called_once_with(alpha_path)

    @patch("worktree._claim_create")
    @patch("worktree._worktree_branch")
    @patch("worktree.get_free_gb", return_value=50.0)
    @patch("worktree.verify_apfs", return_value=True)
    @patch("worktree.DEFAULT_WORKTREE_BASE")
    def test_creates_new_slot(
        self,
        mock_base: MagicMock,
        mock_apfs: MagicMock,
        mock_free: MagicMock,
        mock_branch: MagicMock,
        mock_create: MagicMock,
    ) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            base_path = Path(tmpdir)
            mock_base.__truediv__ = lambda self, key: base_path / key
            # No slots exist => all empty

            with patch("builtins.print") as mock_print:
                cmd_claim(self._make_args())
                expected_path = base_path / "alpha"
                mock_create.assert_called_once_with(expected_path, "alpha", "feat", "master")
                mock_print.assert_called_once_with(expected_path)

    @patch("worktree._is_worktree_available")
    @patch("worktree._worktree_branch")
    @patch("worktree.get_free_gb", return_value=50.0)
    @patch("worktree.verify_apfs", return_value=True)
    @patch("worktree.DEFAULT_WORKTREE_BASE")
    def test_all_occupied_error(
        self,
        mock_base: MagicMock,
        mock_apfs: MagicMock,
        mock_free: MagicMock,
        mock_branch: MagicMock,
        mock_available: MagicMock,
    ) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            base_path = Path(tmpdir)
            for slot in POOL_SLOTS:
                (base_path / slot).mkdir()
            mock_base.__truediv__ = lambda self, key: base_path / key
            mock_branch.return_value = "some-branch"
            mock_available.return_value = False

            with self.assertRaises(SystemExit):
                cmd_claim(self._make_args())

    @patch("worktree._worktree_branch")
    @patch("worktree.get_free_gb", return_value=50.0)
    @patch("worktree.verify_apfs", return_value=True)
    @patch("worktree.DEFAULT_WORKTREE_BASE")
    def test_branch_conflict_error(
        self,
        mock_base: MagicMock,
        mock_apfs: MagicMock,
        mock_free: MagicMock,
        mock_branch: MagicMock,
    ) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            base_path = Path(tmpdir)
            (base_path / "alpha").mkdir()
            mock_base.__truediv__ = lambda self, key: base_path / key
            mock_branch.return_value = "feat"  # Same as requested branch

            with self.assertRaises(SystemExit):
                cmd_claim(self._make_args(branch="feat"))


class TestCmdReset(unittest.TestCase):
    """Test cmd_reset command."""

    def _make_args(self) -> argparse.Namespace:
        return argparse.Namespace()

    @patch("worktree.write_ports")
    @patch("worktree.run_cmd")
    @patch("worktree.list_worktree_paths")
    def test_no_worktrees(
        self,
        mock_list: MagicMock,
        mock_run: MagicMock,
        mock_write_ports: MagicMock,
    ) -> None:
        mock_list.return_value = []
        mock_run.return_value = MagicMock(returncode=0)
        cmd_reset(self._make_args())
        mock_write_ports.assert_called_once_with({})

    @patch("worktree.write_ports")
    @patch("worktree.run_cmd")
    @patch("worktree._worktree_branch")
    @patch("worktree.list_worktree_paths")
    def test_removes_worktrees_and_branches(
        self,
        mock_list: MagicMock,
        mock_branch: MagicMock,
        mock_run: MagicMock,
        mock_write_ports: MagicMock,
    ) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            alpha_path = Path(tmpdir) / "alpha"
            alpha_path.mkdir()
            beta_path = Path(tmpdir) / "beta"
            beta_path.mkdir()
            mock_list.return_value = [alpha_path, beta_path]
            mock_branch.side_effect = ["feature-1", "feature-2"]
            mock_run.return_value = MagicMock(returncode=0)

            cmd_reset(self._make_args())

            # Should have called git worktree remove for each, git worktree prune, and branch -D for each
            remove_calls = [
                c for c in mock_run.call_args_list
                if "worktree" in c[0][0] and "remove" in c[0][0]
            ]
            self.assertEqual(len(remove_calls), 2)

            branch_calls = [
                c for c in mock_run.call_args_list
                if "branch" in c[0][0] and "-D" in c[0][0]
            ]
            self.assertEqual(len(branch_calls), 2)

            mock_write_ports.assert_called_once_with({})

    @patch("worktree.write_ports")
    @patch("worktree.run_cmd")
    @patch("worktree._worktree_branch")
    @patch("worktree.list_worktree_paths")
    def test_skips_master_branch_deletion(
        self,
        mock_list: MagicMock,
        mock_branch: MagicMock,
        mock_run: MagicMock,
        mock_write_ports: MagicMock,
    ) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            alpha_path = Path(tmpdir) / "alpha"
            alpha_path.mkdir()
            mock_list.return_value = [alpha_path]
            mock_branch.return_value = "master"
            mock_run.return_value = MagicMock(returncode=0)

            cmd_reset(self._make_args())

            branch_calls = [
                c for c in mock_run.call_args_list
                if "branch" in c[0][0] and "-D" in c[0][0]
            ]
            self.assertEqual(len(branch_calls), 0)

    @patch("worktree.write_ports")
    @patch("worktree.run_cmd")
    @patch("worktree._worktree_branch")
    @patch("worktree.list_worktree_paths")
    def test_handles_none_branch(
        self,
        mock_list: MagicMock,
        mock_branch: MagicMock,
        mock_run: MagicMock,
        mock_write_ports: MagicMock,
    ) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            alpha_path = Path(tmpdir) / "alpha"
            alpha_path.mkdir()
            mock_list.return_value = [alpha_path]
            mock_branch.return_value = None
            mock_run.return_value = MagicMock(returncode=0)

            cmd_reset(self._make_args())

            branch_calls = [
                c for c in mock_run.call_args_list
                if "branch" in c[0][0] and "-D" in c[0][0]
            ]
            self.assertEqual(len(branch_calls), 0)
            mock_write_ports.assert_called_once_with({})


class TestRegisterSubcommands(unittest.TestCase):
    """Test that subcommands are correctly registered."""

    def _build_parser(self) -> argparse.ArgumentParser:
        parser = argparse.ArgumentParser()
        subparsers = parser.add_subparsers(dest="command", required=True)
        register_subcommands(subparsers)
        return parser

    def test_worktree_create(self) -> None:
        parser = self._build_parser()
        args = parser.parse_args(["worktree", "create", "my-branch"])
        self.assertEqual(args.command, "worktree")
        self.assertEqual(args.worktree_command, "create")
        self.assertEqual(args.branch, "my-branch")
        self.assertFalse(args.existing)
        self.assertEqual(args.base, "master")
        self.assertIsNone(args.dest)
        self.assertFalse(args.dry_run)

    def test_worktree_create_with_flags(self) -> None:
        parser = self._build_parser()
        args = parser.parse_args([
            "worktree", "create", "feat", "--existing", "--base", "develop",
            "--dest", "/tmp/wt", "--dry-run",
        ])
        self.assertTrue(args.existing)
        self.assertEqual(args.base, "develop")
        self.assertEqual(args.dest, "/tmp/wt")
        self.assertTrue(args.dry_run)

    def test_worktree_remove(self) -> None:
        parser = self._build_parser()
        args = parser.parse_args(["worktree", "remove", "alpha"])
        self.assertEqual(args.worktree_command, "remove")
        self.assertEqual(args.target, "alpha")
        self.assertFalse(args.delete_branch)

    def test_worktree_remove_delete_branch(self) -> None:
        parser = self._build_parser()
        args = parser.parse_args(["worktree", "remove", "alpha", "--delete-branch"])
        self.assertTrue(args.delete_branch)

    def test_worktree_refresh_default(self) -> None:
        parser = self._build_parser()
        args = parser.parse_args(["worktree", "refresh"])
        self.assertEqual(args.worktree_command, "refresh")
        self.assertIsNone(args.target)
        self.assertFalse(args.all)
        self.assertFalse(args.build)
        self.assertFalse(args.dry_run)

    def test_worktree_refresh_with_target(self) -> None:
        parser = self._build_parser()
        args = parser.parse_args(["worktree", "refresh", "alpha", "--build", "--dry-run"])
        self.assertEqual(args.target, "alpha")
        self.assertTrue(args.build)
        self.assertTrue(args.dry_run)

    def test_worktree_refresh_all(self) -> None:
        parser = self._build_parser()
        args = parser.parse_args(["worktree", "refresh", "--all"])
        self.assertTrue(args.all)

    def test_worktree_activate(self) -> None:
        parser = self._build_parser()
        args = parser.parse_args(["worktree", "activate", "alpha"])
        self.assertEqual(args.worktree_command, "activate")
        self.assertEqual(args.branch, "alpha")
        self.assertEqual(args.base, "master")
        self.assertFalse(args.dry_run)

    def test_worktree_activate_with_flags(self) -> None:
        parser = self._build_parser()
        args = parser.parse_args(["worktree", "activate", "alpha", "--base", "develop", "--dry-run"])
        self.assertEqual(args.base, "develop")
        self.assertTrue(args.dry_run)

    def test_worktree_list(self) -> None:
        parser = self._build_parser()
        args = parser.parse_args(["worktree", "list"])
        self.assertEqual(args.worktree_command, "list")

    def test_worktree_claim(self) -> None:
        parser = self._build_parser()
        args = parser.parse_args(["worktree", "claim", "my-branch"])
        self.assertEqual(args.worktree_command, "claim")
        self.assertEqual(args.branch, "my-branch")
        self.assertEqual(args.base, "master")

    def test_worktree_claim_with_base(self) -> None:
        parser = self._build_parser()
        args = parser.parse_args(["worktree", "claim", "my-branch", "--base", "develop"])
        self.assertEqual(args.branch, "my-branch")
        self.assertEqual(args.base, "develop")

    def test_worktree_reset(self) -> None:
        parser = self._build_parser()
        args = parser.parse_args(["worktree", "reset"])
        self.assertEqual(args.worktree_command, "reset")

    def test_worktree_no_subcommand_fails(self) -> None:
        parser = self._build_parser()
        with self.assertRaises(SystemExit):
            parser.parse_args(["worktree"])


class TestDispatch(unittest.TestCase):
    """Test that dispatch routes to correct handlers."""

    @patch("worktree.cmd_list")
    def test_dispatch_list(self, mock_list: MagicMock) -> None:
        args = argparse.Namespace(worktree_command="list")
        dispatch(args)
        mock_list.assert_called_once_with(args)

    @patch("worktree.cmd_create")
    def test_dispatch_create(self, mock_create: MagicMock) -> None:
        args = argparse.Namespace(worktree_command="create")
        dispatch(args)
        mock_create.assert_called_once_with(args)

    @patch("worktree.cmd_remove")
    def test_dispatch_remove(self, mock_remove: MagicMock) -> None:
        args = argparse.Namespace(worktree_command="remove")
        dispatch(args)
        mock_remove.assert_called_once_with(args)

    @patch("worktree.cmd_refresh")
    def test_dispatch_refresh(self, mock_refresh: MagicMock) -> None:
        args = argparse.Namespace(worktree_command="refresh")
        dispatch(args)
        mock_refresh.assert_called_once_with(args)

    @patch("worktree.cmd_activate")
    def test_dispatch_activate(self, mock_activate: MagicMock) -> None:
        args = argparse.Namespace(worktree_command="activate")
        dispatch(args)
        mock_activate.assert_called_once_with(args)

    @patch("worktree.cmd_claim")
    def test_dispatch_claim(self, mock_claim: MagicMock) -> None:
        args = argparse.Namespace(worktree_command="claim")
        dispatch(args)
        mock_claim.assert_called_once_with(args)

    @patch("worktree.cmd_reset")
    def test_dispatch_reset(self, mock_reset: MagicMock) -> None:
        args = argparse.Namespace(worktree_command="reset")
        dispatch(args)
        mock_reset.assert_called_once_with(args)


class TestExcludeSet(unittest.TestCase):
    """Test the EXCLUDE set contains expected patterns."""

    def test_contains_ds_store(self) -> None:
        self.assertIn(".DS_Store", EXCLUDE)

    def test_contains_pycache(self) -> None:
        self.assertIn("__pycache__", EXCLUDE)

    def test_contains_client_temp(self) -> None:
        self.assertIn("client/Temp", EXCLUDE)

    def test_contains_abu_state(self) -> None:
        self.assertIn(".abu-state.json", EXCLUDE)


if __name__ == "__main__":
    unittest.main()
