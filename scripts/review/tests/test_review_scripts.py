#!/usr/bin/env python3

"""Unit tests for review performance logging scripts."""

from __future__ import annotations

import json
import os
import subprocess
import sys
import tempfile
import threading
import unittest
from dataclasses import replace
from pathlib import Path

SCRIPT_DIR = Path(__file__).resolve().parent
REVIEW_DIR = SCRIPT_DIR.parent
if str(REVIEW_DIR) not in sys.path:
    sys.path.insert(0, str(REVIEW_DIR))

import analyze_review_perf
import profile_cargo_test
import review_perf_log
import review_scope

REVIEW_IN_WORKTREE_SCRIPT = REVIEW_DIR / "review_in_worktree.sh"


class ReviewPerfLogTests(unittest.TestCase):
    """Tests for low-level performance log helpers."""

    def test_schema_requires_known_event(self) -> None:
        with self.assertRaises(review_perf_log.ReviewPerfLogError):
            review_perf_log.ensure_event_schema({"event": "bogus", "run_id": "x"})

    def test_schema_accepts_scope_plan(self) -> None:
        event = review_perf_log.ensure_event_schema(
            {"event": "scope_plan", "run_id": "run-1"}
        )
        self.assertEqual(event["event"], "scope_plan")

    def test_append_and_prune_runs(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            log_path = Path(temp_dir) / "review.jsonl"

            review_perf_log.append_event(
                {"event": "run_start", "run_id": "r1", "run_seq": 1}, log_path
            )
            review_perf_log.append_event(
                {"event": "run_end", "run_id": "r1", "duration_ms": 10, "status": "ok"},
                log_path,
            )
            review_perf_log.append_event(
                {"event": "run_start", "run_id": "r2", "run_seq": 2}, log_path
            )
            review_perf_log.append_event(
                {"event": "run_end", "run_id": "r2", "duration_ms": 11, "status": "ok"},
                log_path,
            )
            review_perf_log.append_event(
                {"event": "run_start", "run_id": "r3", "run_seq": 3}, log_path
            )
            review_perf_log.append_event(
                {"event": "run_end", "run_id": "r3", "duration_ms": 12, "status": "ok"},
                log_path,
            )

            removed = review_perf_log.prune_to_max_runs(2, log_path)
            self.assertEqual(removed, 1)

            with open(log_path, "r", encoding="utf-8") as handle:
                runs = {json.loads(line)["run_id"] for line in handle if line.strip()}

            self.assertEqual(runs, {"r2", "r3"})

    def test_concurrent_append_uses_lock_file(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            log_path = Path(temp_dir) / "review.jsonl"

            def worker(worker_id: int) -> None:
                for event_index in range(25):
                    review_perf_log.append_event(
                        {
                            "event": "step_end",
                            "run_id": f"run-{worker_id}",
                            "step_name": "test",
                            "duration_ms": event_index,
                            "status": "ok",
                        },
                        log_path,
                    )

            threads = [
                threading.Thread(target=worker, args=(worker_id,))
                for worker_id in range(4)
            ]
            for thread in threads:
                thread.start()
            for thread in threads:
                thread.join()

            lock_path = review_perf_log.lock_path_for(log_path)
            self.assertTrue(lock_path.exists())

            with open(log_path, "r", encoding="utf-8") as handle:
                rows = [json.loads(line) for line in handle if line.strip()]
            self.assertEqual(len(rows), 100)


class AnalyzeReviewPerfTests(unittest.TestCase):
    """Tests for run aggregation and regression input handling."""

    def test_aggregate_runs(self) -> None:
        events = [
            {
                "event": "run_start",
                "run_id": "r1",
                "run_seq": 1,
                "ts": "2026-01-01T00:00:00Z",
                "source": "default",
                "git_commit": "abc",
            },
            {
                "event": "step_end",
                "run_id": "r1",
                "step_name": "test",
                "duration_ms": 2000,
            },
            {"event": "cargo_compile_end", "run_id": "r1", "duration_ms": 250},
            {
                "event": "test_binary_end",
                "run_id": "r1",
                "binary": "/tmp/bin1",
                "duration_ms": 1000,
                "step_name": "test",
            },
            {
                "event": "run_end",
                "run_id": "r1",
                "status": "ok",
                "duration_ms": 5000,
                "ts": "2026-01-01T00:01:00Z",
            },
        ]

        runs = analyze_review_perf.aggregate_runs(events, include_backfill=False)
        self.assertEqual(len(runs), 1)
        run = runs[0]
        self.assertEqual(run.total_ms, 5000)
        self.assertEqual(run.step_ms["test"], 2000)
        self.assertEqual(run.compile_ms, 250)
        self.assertEqual(len(run.test_binary_ms), 1)


class ProfileCargoTestHelpersTests(unittest.TestCase):
    """Tests for parser/fallback logic in test profiler."""

    def test_parse_test_result_counts(self) -> None:
        sample = "test result: ok. 12 passed; 1 failed; 2 ignored; 0 measured; 3 filtered out"
        self.assertEqual(
            profile_cargo_test.parse_test_result_counts(sample), (12, 1, 2, 0, 3)
        )

    def test_nightly_fallback_detection(self) -> None:
        result = profile_cargo_test.BinaryExecutionResult(
            exit_code=101,
            output="",
            error_output='error: The "report-time" flag is only accepted on the nightly compiler with -Z unstable-options',
            passed=0,
            failed=0,
            ignored=0,
            measured=0,
            filtered=0,
        )
        self.assertTrue(profile_cargo_test.should_fallback_from_nightly(result))


class ReviewScopePlannerTests(unittest.TestCase):
    """Tests for scoped review planning and validation helpers."""

    def setUp(self) -> None:
        self.step_names = [
            "check-snapshots",
            "check-format",
            "check-docs-format",
            "check-token-limits",
            "review-scope-validate",
            "build",
            "clippy",
            "style-validator",
            "rlf-lint",
            "test-core",
            "python-test",
            "local-unity-test",
            "parser-test",
            "tv-check",
            "tv-clippy",
            "tv-test",
        ]
        self.config = review_scope.ScopeConfig(
            required_global_full_triggers=(
                ".github/",
                "justfile",
                "rules_engine/Cargo.toml",
                "rules_engine/Cargo.lock",
                "scripts/review/review_scope.py",
                "scripts/review/review_scope_config.json",
            ),
            global_full_triggers=(
                ".github/",
                "justfile",
                "rules_engine/Cargo.toml",
                "rules_engine/Cargo.lock",
                "scripts/review/review_scope.py",
                "scripts/review/review_scope_config.json",
                "rules_engine/tabula/",
            ),
            parser_crate_seeds=("parser", "parser_tests", "parser_benchmarks"),
            parser_path_prefixes=(
                "rules_engine/src/parser/",
                "rules_engine/tests/parser_tests/",
            ),
            tv_crate_seeds=("tv", "tv_tests"),
            tv_path_prefixes=("rules_engine/src/tv/", "rules_engine/tests/tv_tests/"),
            csharp_crate_seeds=(),
            csharp_path_prefixes=("client/Assets/",),
            cqs_crate_seeds=(),
            cqs_path_prefixes=("scripts/constructed_quest_prototype/",),
            always_run_steps=(
                "check-snapshots",
                "check-format",
                "check-docs-format",
                "check-token-limits",
                "review-scope-validate",
                "build",
                "clippy",
                "style-validator",
                "rlf-lint",
                "test-core",
            ),
            markdown_only_skip_steps=(
                "build",
                "clippy",
                "rlf-lint",
                "test-core",
                "python-test",
                "parser-test",
                "tv-check",
                "tv-clippy",
                "tv-test",
                "local-unity-test",
            ),
            python_docs_only_skip_steps=(
                "build",
                "clippy",
                "test-core",
                "local-unity-test",
            ),
            parser_steps=("parser-test",),
            tv_steps=("tv-check", "tv-clippy", "tv-test"),
            python_steps=("python-test",),
            csharp_steps=("local-unity-test",),
            cqs_steps=("cqs-check",),
            core_path_prefixes=(
                "client/Assets/StreamingAssets/",
                "scripts/quest_prototype/",
            ),
        )
        self.metadata = review_scope.WorkspaceMetadata(
            crate_roots={
                "core_data": "rules_engine/src/core_data",
                "parser": "rules_engine/src/parser",
                "parser_tests": "rules_engine/tests/parser_tests",
                "rules_engine": "rules_engine/src/rules_engine",
                "tv_tests": "rules_engine/tests/tv_tests",
            },
            reverse_dependencies={
                "core_data": {"parser", "rules_engine"},
                "parser": {"parser_tests"},
                "parser_tests": set(),
                "rules_engine": set(),
                "tv_tests": set(),
            },
        )

    def test_changed_files_env_override_precedence(self) -> None:
        env = {
            "REVIEW_SCOPE_MODE": "enforce",
            "REVIEW_SCOPE_CHANGED_FILES": "rules_engine/src/core_data/src/lib.rs\n",
            "REVIEW_SCOPE_BASE_REF": "abc",
            "REVIEW_SCOPE_HEAD_REF": "def",
        }

        def fail_if_called(_: list[str], __: Path) -> tuple[int, str, str]:
            raise AssertionError(
                "git commands should not run when REVIEW_SCOPE_CHANGED_FILES is set"
            )

        decision = review_scope.plan_review_scope(
            self.step_names,
            env=env,
            repo_root=Path.cwd(),
            config=self.config,
            metadata=self.metadata,
            command_runner=fail_if_called,
        )

        self.assertEqual(
            decision.changed_files_source, "env:REVIEW_SCOPE_CHANGED_FILES"
        )
        self.assertEqual(
            decision.changed_files, ["rules_engine/src/core_data/src/lib.rs"]
        )

    def test_default_local_strategy_uses_only_head_changes(self) -> None:
        env = {"REVIEW_SCOPE_MODE": "enforce"}
        calls: list[list[str]] = []

        def command_runner(args: list[str], _: Path) -> tuple[int, str, str]:
            calls.append(args)
            if args == ["git", "diff", "--name-only", "--cached", "HEAD"]:
                return (0, "rules_engine/src/core_data/src/lib.rs\n", "")
            if args == ["git", "diff", "--name-only"]:
                return (0, "rules_engine/src/rules_engine/src/lib.rs\n", "")
            if args == ["git", "ls-files", "--others", "--exclude-standard"]:
                return (0, "scratch/new_file.txt\n", "")
            raise AssertionError(f"unexpected command: {args}")

        changed = review_scope.resolve_changed_files(
            env, Path.cwd(), command_runner=command_runner
        )
        self.assertEqual(changed.source, "local-head-dirty")
        self.assertEqual(
            changed.changed_files,
            [
                "rules_engine/src/core_data/src/lib.rs",
                "rules_engine/src/rules_engine/src/lib.rs",
                "scratch/new_file.txt",
            ],
        )
        self.assertEqual(
            calls,
            [
                ["git", "diff", "--name-only", "--cached", "HEAD"],
                ["git", "diff", "--name-only"],
                ["git", "ls-files", "--others", "--exclude-standard"],
            ],
        )

    def test_default_local_strategy_clean_forces_full(self) -> None:
        env = {"REVIEW_SCOPE_MODE": "enforce"}

        def command_runner(args: list[str], _: Path) -> tuple[int, str, str]:
            if tuple(args) in {
                ("git", "diff", "--name-only", "--cached", "HEAD"),
                ("git", "diff", "--name-only"),
                ("git", "ls-files", "--others", "--exclude-standard"),
            }:
                return (0, "", "")
            raise AssertionError(f"unexpected command: {args}")

        decision = review_scope.plan_review_scope(
            self.step_names,
            env=env,
            repo_root=Path.cwd(),
            config=self.config,
            metadata=self.metadata,
            command_runner=command_runner,
        )
        self.assertEqual(decision.changed_files_source, "local-clean")
        self.assertTrue(decision.forced_full)
        self.assertEqual(decision.forced_full_reason, "no changed files detected")

    def test_merge_base_union_strategy_includes_branch_diff(self) -> None:
        env = {
            "REVIEW_SCOPE_MODE": "enforce",
            "REVIEW_SCOPE_LOCAL_STRATEGY": "merge-base-union",
        }
        calls: list[list[str]] = []

        def command_runner(args: list[str], _: Path) -> tuple[int, str, str]:
            calls.append(args)
            if args == ["git", "diff", "--name-only", "--cached", "HEAD"]:
                return (0, "rules_engine/src/core_data/src/lib.rs\n", "")
            if args == ["git", "diff", "--name-only"]:
                return (0, "", "")
            if args == ["git", "ls-files", "--others", "--exclude-standard"]:
                return (0, "", "")
            if args == ["git", "merge-base", "origin/master", "HEAD"]:
                return (0, "123abc\n", "")
            if args == ["git", "diff", "--name-only", "123abc...HEAD"]:
                return (0, "rules_engine/src/parser/src/lib.rs\n", "")
            raise AssertionError(f"unexpected command: {args}")

        changed = review_scope.resolve_changed_files(
            env, Path.cwd(), command_runner=command_runner
        )
        self.assertEqual(changed.source, "local-merge-base-union")
        self.assertEqual(
            changed.changed_files,
            [
                "rules_engine/src/parser/src/lib.rs",
                "rules_engine/src/core_data/src/lib.rs",
            ],
        )
        self.assertIn(["git", "merge-base", "origin/master", "HEAD"], calls)

    def test_base_head_override_precedence(self) -> None:
        env = {
            "REVIEW_SCOPE_MODE": "enforce",
            "REVIEW_SCOPE_BASE_REF": "abc123",
            "REVIEW_SCOPE_HEAD_REF": "def456",
            "CI": "1",
        }
        calls: list[list[str]] = []

        def command_runner(args: list[str], _: Path) -> tuple[int, str, str]:
            calls.append(args)
            if args == ["git", "diff", "--name-only", "abc123...def456"]:
                return (0, "rules_engine/src/core_data/src/lib.rs\n", "")
            raise AssertionError(f"unexpected command: {args}")

        decision = review_scope.plan_review_scope(
            self.step_names,
            env=env,
            repo_root=Path.cwd(),
            config=self.config,
            metadata=self.metadata,
            command_runner=command_runner,
        )

        self.assertEqual(decision.changed_files_source, "git:abc123...def456")
        self.assertEqual(calls, [["git", "diff", "--name-only", "abc123...def456"]])

    def test_unknown_path_forces_full(self) -> None:
        env = {
            "REVIEW_SCOPE_MODE": "enforce",
            "REVIEW_SCOPE_CHANGED_FILES": "notes/unmapped.txt",
        }
        decision = review_scope.plan_review_scope(
            self.step_names,
            env=env,
            repo_root=Path.cwd(),
            config=self.config,
            metadata=self.metadata,
        )
        self.assertTrue(decision.forced_full)
        self.assertIn("unmapped changed path", decision.forced_full_reason)
        self.assertIn("notes/unmapped.txt", decision.unmapped_paths)

    def test_core_path_prefix_skips_csharp_domain(self) -> None:
        env = {
            "REVIEW_SCOPE_MODE": "enforce",
            "REVIEW_SCOPE_CHANGED_FILES": "client/Assets/StreamingAssets/Tabula/rendered-cards.toml",
        }
        decision = review_scope.plan_review_scope(
            self.step_names,
            env=env,
            repo_root=Path.cwd(),
            config=self.config,
            metadata=self.metadata,
        )
        self.assertFalse(decision.forced_full)
        self.assertNotIn("csharp", decision.domains)
        self.assertIn("local-unity-test", decision.skipped_steps)
        self.assertNotIn(
            "client/Assets/StreamingAssets/Tabula/rendered-cards.toml",
            decision.unmapped_paths,
        )

    def test_tabula_path_forces_full(self) -> None:
        env = {
            "REVIEW_SCOPE_MODE": "enforce",
            "REVIEW_SCOPE_CHANGED_FILES": "rules_engine/tabula/cards.toml",
        }
        decision = review_scope.plan_review_scope(
            self.step_names,
            env=env,
            repo_root=Path.cwd(),
            config=self.config,
            metadata=self.metadata,
        )
        self.assertTrue(decision.forced_full)
        self.assertIn("matched global full trigger", decision.forced_full_reason)

    def test_global_trigger_forces_full(self) -> None:
        env = {
            "REVIEW_SCOPE_MODE": "enforce",
            "REVIEW_SCOPE_CHANGED_FILES": "justfile",
        }
        decision = review_scope.plan_review_scope(
            self.step_names,
            env=env,
            repo_root=Path.cwd(),
            config=self.config,
            metadata=self.metadata,
        )
        self.assertTrue(decision.forced_full)
        self.assertIn("justfile", decision.forced_full_reason)

    def test_reverse_dependency_marks_parser_domain(self) -> None:
        env = {
            "REVIEW_SCOPE_MODE": "enforce",
            "REVIEW_SCOPE_CHANGED_FILES": "rules_engine/src/core_data/src/lib.rs",
        }
        decision = review_scope.plan_review_scope(
            self.step_names,
            env=env,
            repo_root=Path.cwd(),
            config=self.config,
            metadata=self.metadata,
        )
        self.assertFalse(decision.forced_full)
        self.assertIn("parser", decision.domains)
        self.assertIn("parser-test", decision.selected_steps)

    def test_tv_only_change_skips_parser_when_enforced(self) -> None:
        env = {
            "REVIEW_SCOPE_MODE": "enforce",
            "REVIEW_SCOPE_CHANGED_FILES": "rules_engine/src/tv/src-tauri/src/main.rs",
        }
        decision = review_scope.plan_review_scope(
            self.step_names,
            env=env,
            repo_root=Path.cwd(),
            config=self.config,
            metadata=self.metadata,
        )
        self.assertFalse(decision.forced_full)
        self.assertIn("tv", decision.domains)
        self.assertNotIn("parser-test", decision.selected_steps)
        self.assertIn("parser-test", decision.skipped_steps)
        self.assertIn("tv-check", decision.selected_steps)

    def test_dry_run_reports_skips_without_enforcement(self) -> None:
        env = {
            "REVIEW_SCOPE_MODE": "dry-run",
            "REVIEW_SCOPE_CHANGED_FILES": "rules_engine/src/tv/src-tauri/src/main.rs",
        }
        decision = review_scope.plan_review_scope(
            self.step_names,
            env=env,
            repo_root=Path.cwd(),
            config=self.config,
            metadata=self.metadata,
        )
        self.assertFalse(decision.enforce)
        self.assertIn("parser-test", decision.skipped_steps)
        self.assertIn("tv-check", decision.selected_steps)

    def test_enforce_mode_core_only_skips_parser_and_tv_steps(self) -> None:
        env = {
            "REVIEW_SCOPE_MODE": "enforce",
            "REVIEW_SCOPE_CHANGED_FILES": "rules_engine/src/rules_engine/src/lib.rs",
        }
        decision = review_scope.plan_review_scope(
            self.step_names,
            env=env,
            repo_root=Path.cwd(),
            config=self.config,
            metadata=self.metadata,
        )
        self.assertTrue(decision.enforce)
        self.assertNotIn("python-test", decision.selected_steps)
        self.assertNotIn("parser-test", decision.selected_steps)
        self.assertNotIn("tv-check", decision.selected_steps)
        self.assertIn("python-test", decision.skipped_steps)
        self.assertIn("parser-test", decision.skipped_steps)
        self.assertIn("tv-check", decision.skipped_steps)

    def test_markdown_only_change_skips_build_lints_and_all_tests(self) -> None:
        env = {
            "REVIEW_SCOPE_MODE": "enforce",
            "REVIEW_SCOPE_CHANGED_FILES": "docs/notes/plan.MD",
        }
        decision = review_scope.plan_review_scope(
            self.step_names,
            env=env,
            repo_root=Path.cwd(),
            config=self.config,
            metadata=self.metadata,
        )

        self.assertFalse(decision.forced_full)
        self.assertTrue(decision.markdown_only)
        self.assertEqual(decision.domains, ["docs"])
        self.assertEqual(decision.unmapped_paths, [])
        self.assertIn("check-snapshots", decision.selected_steps)
        self.assertIn("check-format", decision.selected_steps)
        self.assertIn("check-docs-format", decision.selected_steps)
        self.assertIn("check-token-limits", decision.selected_steps)
        self.assertIn("review-scope-validate", decision.selected_steps)
        self.assertIn("style-validator", decision.selected_steps)
        self.assertNotIn("build", decision.selected_steps)
        self.assertNotIn("clippy", decision.selected_steps)
        self.assertNotIn("test-core", decision.selected_steps)
        self.assertNotIn("python-test", decision.selected_steps)
        self.assertNotIn("parser-test", decision.selected_steps)
        self.assertNotIn("tv-check", decision.selected_steps)
        self.assertNotIn("tv-clippy", decision.selected_steps)
        self.assertNotIn("tv-test", decision.selected_steps)
        self.assertEqual(decision.skipped_steps["build"], "markdown-only changes")
        self.assertEqual(decision.skipped_steps["test-core"], "markdown-only changes")
        self.assertEqual(decision.skipped_steps["python-test"], "markdown-only changes")
        self.assertEqual(decision.skipped_steps["tv-test"], "markdown-only changes")

    def test_python_change_selects_python_test_step(self) -> None:
        env = {
            "REVIEW_SCOPE_MODE": "enforce",
            "REVIEW_SCOPE_CHANGED_FILES": "scripts/utility/grid_generator.py",
        }
        decision = review_scope.plan_review_scope(
            self.step_names,
            env=env,
            repo_root=Path.cwd(),
            config=self.config,
            metadata=self.metadata,
        )
        self.assertFalse(decision.forced_full)
        self.assertIn("python", decision.domains)
        self.assertNotIn("docs", decision.domains)
        self.assertNotIn("build", decision.selected_steps)
        self.assertNotIn("clippy", decision.selected_steps)
        self.assertNotIn("test-core", decision.selected_steps)
        self.assertIn("python-test", decision.selected_steps)
        self.assertEqual(decision.skipped_steps["build"], "python/docs-only changes")
        self.assertEqual(decision.skipped_steps["clippy"], "python/docs-only changes")
        self.assertEqual(
            decision.skipped_steps["test-core"], "python/docs-only changes"
        )
        self.assertNotIn("python-test", decision.skipped_steps)

    def test_shell_script_change_is_core_not_unmapped(self) -> None:
        env = {
            "REVIEW_SCOPE_MODE": "enforce",
            "REVIEW_SCOPE_CHANGED_FILES": "scripts/worktrees/cleanup_integrated_task.sh",
        }
        decision = review_scope.plan_review_scope(
            self.step_names,
            env=env,
            repo_root=Path.cwd(),
            config=self.config,
            metadata=self.metadata,
        )
        self.assertFalse(decision.forced_full)
        self.assertEqual(decision.domains, ["core"])
        self.assertEqual(decision.unmapped_paths, [])
        self.assertIn("build", decision.selected_steps)
        self.assertIn("clippy", decision.selected_steps)
        self.assertIn("test-core", decision.selected_steps)
        self.assertNotIn("python-test", decision.selected_steps)
        self.assertIn("python-test", decision.skipped_steps)

    def test_quest_prototype_change_is_core_not_unmapped(self) -> None:
        env = {
            "REVIEW_SCOPE_MODE": "enforce",
            "REVIEW_SCOPE_CHANGED_FILES": "scripts/quest_prototype/src/types/cards.ts",
        }
        decision = review_scope.plan_review_scope(
            self.step_names,
            env=env,
            repo_root=Path.cwd(),
            config=self.config,
            metadata=self.metadata,
        )
        self.assertFalse(decision.forced_full)
        self.assertEqual(decision.domains, ["core"])
        self.assertEqual(decision.unmapped_paths, [])
        self.assertNotIn("local-unity-test", decision.selected_steps)
        self.assertIn("local-unity-test", decision.skipped_steps)

    def test_mixed_markdown_and_code_change_is_not_markdown_only(self) -> None:
        env = {
            "REVIEW_SCOPE_MODE": "enforce",
            "REVIEW_SCOPE_CHANGED_FILES": "docs/notes/plan.md\nrules_engine/src/core_data/src/lib.rs",
        }
        decision = review_scope.plan_review_scope(
            self.step_names,
            env=env,
            repo_root=Path.cwd(),
            config=self.config,
            metadata=self.metadata,
        )
        self.assertFalse(decision.markdown_only)
        self.assertFalse(decision.forced_full)
        self.assertIn("docs", decision.domains)
        self.assertIn("parser", decision.domains)
        self.assertIn("build", decision.selected_steps)
        self.assertIn("test-core", decision.selected_steps)

    def test_python_and_markdown_only_skips_rust_steps(self) -> None:
        env = {
            "REVIEW_SCOPE_MODE": "enforce",
            "REVIEW_SCOPE_CHANGED_FILES": "docs/notes/plan.md\nscripts/utility/grid_generator.py",
        }
        decision = review_scope.plan_review_scope(
            self.step_names,
            env=env,
            repo_root=Path.cwd(),
            config=self.config,
            metadata=self.metadata,
        )
        self.assertFalse(decision.forced_full)
        self.assertIn("docs", decision.domains)
        self.assertIn("python", decision.domains)
        self.assertNotIn("build", decision.selected_steps)
        self.assertNotIn("clippy", decision.selected_steps)
        self.assertNotIn("test-core", decision.selected_steps)
        self.assertIn("python-test", decision.selected_steps)
        self.assertEqual(decision.skipped_steps["build"], "python/docs-only changes")
        self.assertEqual(decision.skipped_steps["clippy"], "python/docs-only changes")
        self.assertEqual(
            decision.skipped_steps["test-core"], "python/docs-only changes"
        )

    def test_scope_validator_passes_repo_config(self) -> None:
        repo_root = Path(__file__).resolve().parents[3]
        config = review_scope.load_scope_config()
        metadata = review_scope.load_workspace_metadata(repo_root)
        errors = review_scope.validate_scope_configuration(config, metadata)
        self.assertEqual(errors, [])

    def test_scope_validator_fails_missing_required_trigger(self) -> None:
        broken = replace(
            self.config,
            global_full_triggers=tuple(
                trigger
                for trigger in self.config.global_full_triggers
                if trigger != ".github/"
            ),
        )
        errors = review_scope.validate_scope_configuration(broken, self.metadata)
        self.assertTrue(
            any("missing required global full triggers" in error for error in errors)
        )


class ReviewInWorktreeShellTests(unittest.TestCase):
    """Regression tests for review worktree slot cleanup helpers."""

    def test_normalize_slot_reclaims_locked_slot_named_branch(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            temp_path = Path(temp_dir)
            repo_root = temp_path / "repo"
            slot_path = temp_path / "worktrees" / "alpha"

            self._init_repo(repo_root)
            base = self._git_stdout(repo_root, "rev-parse", "HEAD")
            self._git(repo_root, "worktree", "add", "-b", "alpha", str(slot_path), base)

            (slot_path / "tracked.txt").write_text("slot change\n", encoding="utf-8")
            self._git(slot_path, "commit", "-am", "slot branch change")
            (slot_path / "scratch.txt").write_text("temp\n", encoding="utf-8")
            (slot_path / ".review-lock").write_text("", encoding="utf-8")
            self._git(slot_path, "add", "-f", ".review-lock")

            self._run_bash(
                f'source "{REVIEW_IN_WORKTREE_SCRIPT}"; '
                f'normalize_slot_for_review alpha "{slot_path}" "{base}" true',
                repo_root,
                temp_dir,
            )

            self.assertEqual(self._git_stdout(slot_path, "rev-parse", "HEAD"), base)
            self.assertEqual(
                self._git_stdout(slot_path, "branch", "--show-current"), ""
            )
            self.assertEqual(self._git_stdout(slot_path, "status", "--short"), "")
            self.assertEqual(
                self._git_stdout(repo_root, "branch", "--list", "alpha"), ""
            )

    def test_normalize_slot_skips_non_pool_feature_branch(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            temp_path = Path(temp_dir)
            repo_root = temp_path / "repo"
            slot_path = temp_path / "worktrees" / "alpha"

            self._init_repo(repo_root)
            base = self._git_stdout(repo_root, "rev-parse", "HEAD")
            self._git(
                repo_root,
                "worktree",
                "add",
                "-b",
                "feature-review",
                str(slot_path),
                base,
            )

            self._run_bash(
                f'source "{REVIEW_IN_WORKTREE_SCRIPT}"; '
                f'normalize_slot_for_review alpha "{slot_path}" "{base}"',
                repo_root,
                temp_dir,
            )

            self.assertEqual(
                self._git_stdout(slot_path, "branch", "--show-current"),
                "feature-review",
            )
            self.assertIn(
                "feature-review",
                self._git_stdout(repo_root, "branch", "--list", "feature-review"),
            )

    def _git(self, cwd: Path, *args: str) -> subprocess.CompletedProcess[str]:
        return subprocess.run(
            ["git", *args],
            check=True,
            cwd=cwd,
            text=True,
            capture_output=True,
        )

    def _git_stdout(self, cwd: Path, *args: str) -> str:
        return self._git(cwd, *args).stdout.strip()

    def _init_repo(self, repo_root: Path) -> None:
        repo_root.mkdir(parents=True)
        self._git(repo_root, "init", "-b", "master")
        self._git(repo_root, "config", "user.email", "review-tests@example.com")
        self._git(repo_root, "config", "user.name", "Review Tests")
        (repo_root / "tracked.txt").write_text("base\n", encoding="utf-8")
        self._git(repo_root, "add", "tracked.txt")
        self._git(repo_root, "commit", "-m", "initial")

    def _run_bash(self, command: str, repo_root: Path, home: str) -> None:
        env = os.environ.copy()
        env["DREAMTIDES_REPO_ROOT"] = str(repo_root)
        env["HOME"] = home
        subprocess.run(
            ["bash", "-lc", command],
            check=True,
            cwd=repo_root,
            env=env,
            text=True,
            capture_output=True,
        )


if __name__ == "__main__":
    unittest.main()
