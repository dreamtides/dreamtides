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

            self.run_cmd(root, "done", "T0001")
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


if __name__ == "__main__":
    unittest.main()
