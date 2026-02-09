#!/usr/bin/env python3
"""Smoke tests for .codex/scripts/task.py."""

from __future__ import annotations

import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path


SCRIPT_PATH = Path(__file__).resolve().with_name("task.py")


class TaskCliSmokeTest(unittest.TestCase):
    """Validate expected task CLI behaviors and edge cases."""

    def run_cmd(
        self,
        root: Path,
        *args: str,
        stdin_text: str | None = None,
        expect_code: int = 0,
    ) -> subprocess.CompletedProcess[str]:
        command = [sys.executable, str(SCRIPT_PATH), "--root", str(root), *args]
        result = subprocess.run(
            command,
            text=True,
            input=stdin_text,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=False,
        )
        self.assertEqual(
            result.returncode,
            expect_code,
            msg=f"Unexpected exit code.\nstdout:\n{result.stdout}\nstderr:\n{result.stderr}",
        )
        return result

    def test_smoke_scenarios(self) -> None:
        """Run scenario coverage from init through validation failures."""
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)

            self.run_cmd(root, "init")
            index_path = root / ".codex" / "tasks" / "index.json"
            self.assertTrue(index_path.exists())

            markdown_one = root / "task_1.md"
            markdown_one.write_text("## Context\nFirst task.\n", encoding="utf-8")
            self.run_cmd(
                root,
                "add",
                "--title",
                "First task",
                "--markdown-file",
                str(markdown_one),
            )
            markdown_two = root / "task_2.md"
            markdown_two.write_text("## Context\nSecond task.\n", encoding="utf-8")
            self.run_cmd(
                root,
                "add",
                "--title",
                "Second task",
                "--markdown-file",
                str(markdown_two),
                "--blocked-by",
                "T0001",
            )

            index_payload = json.loads(index_path.read_text(encoding="utf-8"))
            self.assertEqual(index_payload["next_id"], 3)
            self.assertEqual(index_payload["tasks"][1]["blocked_by"], ["T0001"])

            ready_before = self.run_cmd(root, "ready", "--json")
            ready_before_payload = json.loads(ready_before.stdout)
            self.assertEqual([task["id"] for task in ready_before_payload], ["T0001"])

            started = self.run_cmd(root, "start", "--json", "--body")
            started_payload = json.loads(started.stdout)
            self.assertEqual(started_payload["id"], "T0001")
            self.assertEqual(started_payload["status"], "in_progress")
            self.assertIn("## Context", started_payload["markdown"])

            none_ready = self.run_cmd(root, "start", "--json")
            self.assertIsNone(json.loads(none_ready.stdout))

            self.run_cmd(root, "release", "T0001")
            restarted = self.run_cmd(root, "start", "--id-only")
            self.assertEqual(restarted.stdout.strip(), "T0001")

            self.run_cmd(root, "finish", "T0001")
            ready_after = self.run_cmd(root, "ready", "--json")
            ready_after_payload = json.loads(ready_after.stdout)
            self.assertEqual([task["id"] for task in ready_after_payload], ["T0002"])

            self.run_cmd(root, "update", "T0002", "--add-blocker", "T0001")
            self.run_cmd(root, "update", "T0002", "--remove-blocker", "T0001")
            task_two = self.run_cmd(root, "get", "T0002", "--json")
            task_two_payload = json.loads(task_two.stdout)
            self.assertEqual(task_two_payload["blocked_by"], [])

            body_output = self.run_cmd(root, "get", "T0002", "--body")
            self.assertIn("## Context", body_output.stdout)

            self.run_cmd(
                root,
                "update",
                "T0002",
                "--add-blocker",
                "T0001",
            )
            self.run_cmd(
                root,
                "update",
                "T0001",
                "--add-blocker",
                "T0002",
                expect_code=1,
            )
            self.run_cmd(root, "update", "T0002", "--remove-blocker", "T0001")

            cycle_task = root / "task_3.md"
            cycle_task.write_text("## Context\nThird task.\n", encoding="utf-8")
            self.run_cmd(
                root,
                "add",
                "--title",
                "Third task",
                "--markdown-file",
                str(cycle_task),
            )

            self.run_cmd(
                root,
                "add",
                "--title",
                "Bad blocker",
                "--markdown-file",
                str(cycle_task),
                "--blocked-by",
                "T9999",
                expect_code=1,
            )
            self.run_cmd(
                root,
                "update",
                "T0002",
                "--status",
                "invalid",
                expect_code=2,
            )
            self.run_cmd(root, "get", "T9999", expect_code=1)

            mutated_index = json.loads(index_path.read_text(encoding="utf-8"))
            mutated_index["tasks"][0]["blocked_by"] = ["T0002"]
            mutated_index["tasks"][1]["blocked_by"] = ["T0001"]
            index_path.write_text(
                json.dumps(mutated_index, indent=2) + "\n",
                encoding="utf-8",
            )
            self.run_cmd(root, "validate", expect_code=1)

    def test_add_is_exclusive_under_parallel_writers(self) -> None:
        """Ensure concurrent add operations do not reuse IDs or drop tasks."""
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            self.run_cmd(root, "init")
            markdown_one = root / "task_1.md"
            markdown_one.write_text("First\n", encoding="utf-8")
            markdown_two = root / "task_2.md"
            markdown_two.write_text("Second\n", encoding="utf-8")

            first_command = [
                sys.executable,
                str(SCRIPT_PATH),
                "--root",
                str(root),
                "add",
                "--title",
                "First",
                "--markdown-file",
                str(markdown_one),
            ]
            second_command = [
                sys.executable,
                str(SCRIPT_PATH),
                "--root",
                str(root),
                "add",
                "--title",
                "Second",
                "--markdown-file",
                str(markdown_two),
            ]
            first_process = subprocess.Popen(
                first_command,
                text=True,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
            )
            second_process = subprocess.Popen(
                second_command,
                text=True,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
            )
            first_stdout, first_stderr = first_process.communicate()
            second_stdout, second_stderr = second_process.communicate()
            self.assertEqual(first_process.returncode, 0, first_stderr)
            self.assertEqual(second_process.returncode, 0, second_stderr)
            self.assertIn("Created T000", first_stdout)
            self.assertIn("Created T000", second_stdout)

            listed = self.run_cmd(root, "list", "--all", "--json")
            listed_payload = json.loads(listed.stdout)
            self.assertEqual([task["id"] for task in listed_payload], ["T0001", "T0002"])

    def test_start_is_exclusive_under_parallel_claimers(self) -> None:
        """Ensure only one process can claim a ready task at a time."""
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            self.run_cmd(root, "init")
            markdown = root / "task_1.md"
            markdown.write_text("First\n", encoding="utf-8")
            self.run_cmd(
                root,
                "add",
                "--title",
                "First",
                "--markdown-file",
                str(markdown),
            )

            processes: list[subprocess.Popen[str]] = []
            for _ in range(8):
                command = [
                    sys.executable,
                    str(SCRIPT_PATH),
                    "--root",
                    str(root),
                    "start",
                    "--id-only",
                ]
                processes.append(
                    subprocess.Popen(
                        command,
                        text=True,
                        stdout=subprocess.PIPE,
                        stderr=subprocess.PIPE,
                    )
                )

            claims: list[str] = []
            for process in processes:
                stdout, stderr = process.communicate()
                self.assertEqual(process.returncode, 0, stderr)
                if stdout.strip():
                    claims.append(stdout.strip())
            self.assertEqual(claims, ["T0001"])

            task_payload = json.loads(self.run_cmd(root, "get", "T0001", "--json").stdout)
            self.assertEqual(task_payload["status"], "in_progress")
            self.assertIsInstance(task_payload["claimed_by"], str)
            self.assertIsInstance(task_payload["claimed_at"], str)
            self.assertIsInstance(task_payload["lease_expires_at"], str)

    def test_start_reclaims_expired_lease(self) -> None:
        """Ensure start reclaims stale in-progress leases before selecting work."""
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            self.run_cmd(root, "init")
            markdown = root / "task_1.md"
            markdown.write_text("First\n", encoding="utf-8")
            self.run_cmd(
                root,
                "add",
                "--title",
                "First",
                "--markdown-file",
                str(markdown),
            )
            self.run_cmd(
                root,
                "start",
                "--id-only",
                "--claimant",
                "worker-a",
                "--lease-seconds",
                "600",
            )

            index_path = root / ".codex" / "tasks" / "index.json"
            index_payload = json.loads(index_path.read_text(encoding="utf-8"))
            index_payload["tasks"][0]["lease_expires_at"] = "2000-01-01T00:00:00Z"
            index_path.write_text(json.dumps(index_payload, indent=2) + "\n", encoding="utf-8")

            restarted = json.loads(self.run_cmd(root, "start", "--json").stdout)
            self.assertEqual(restarted["id"], "T0001")
            self.assertEqual(restarted["status"], "in_progress")
            self.assertNotEqual(restarted["lease_expires_at"], "2000-01-01T00:00:00Z")


if __name__ == "__main__":
    unittest.main()
