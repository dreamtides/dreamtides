#!/usr/bin/env python3

"""Unit tests for review performance logging scripts."""

from __future__ import annotations

import json
import tempfile
import threading
import unittest
from pathlib import Path

import analyze_review_perf
import profile_cargo_test
import review_perf_log


class ReviewPerfLogTests(unittest.TestCase):
    """Tests for low-level performance log helpers."""

    def test_schema_requires_known_event(self) -> None:
        with self.assertRaises(review_perf_log.ReviewPerfLogError):
            review_perf_log.ensure_event_schema({"event": "bogus", "run_id": "x"})

    def test_append_and_prune_runs(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            log_path = Path(temp_dir) / "review.jsonl"

            review_perf_log.append_event({"event": "run_start", "run_id": "r1", "run_seq": 1}, log_path)
            review_perf_log.append_event({"event": "run_end", "run_id": "r1", "duration_ms": 10, "status": "ok"}, log_path)
            review_perf_log.append_event({"event": "run_start", "run_id": "r2", "run_seq": 2}, log_path)
            review_perf_log.append_event({"event": "run_end", "run_id": "r2", "duration_ms": 11, "status": "ok"}, log_path)
            review_perf_log.append_event({"event": "run_start", "run_id": "r3", "run_seq": 3}, log_path)
            review_perf_log.append_event({"event": "run_end", "run_id": "r3", "duration_ms": 12, "status": "ok"}, log_path)

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

            threads = [threading.Thread(target=worker, args=(worker_id,)) for worker_id in range(4)]
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
            {"event": "run_start", "run_id": "r1", "run_seq": 1, "ts": "2026-01-01T00:00:00Z", "source": "default", "git_commit": "abc"},
            {"event": "step_end", "run_id": "r1", "step_name": "test", "duration_ms": 2000},
            {"event": "cargo_compile_end", "run_id": "r1", "duration_ms": 250},
            {"event": "test_binary_end", "run_id": "r1", "binary": "/tmp/bin1", "duration_ms": 1000, "step_name": "test"},
            {"event": "run_end", "run_id": "r1", "status": "ok", "duration_ms": 5000, "ts": "2026-01-01T00:01:00Z"},
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
        self.assertEqual(profile_cargo_test.parse_test_result_counts(sample), (12, 1, 2, 0, 3))

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


if __name__ == "__main__":
    unittest.main()
