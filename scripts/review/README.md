# Review Scripts

This directory contains the scripts that power scoped `just review` runs and review performance analysis.

## Files

- `review_runner.py`: orchestrates `just review` steps, applies scope decisions, and emits structured telemetry events.
- `review_scope.py`: computes changed-file scope and selects which review steps should run or be skipped.
- `review_scope_config.json`: static scope policy for domains, force-full triggers, and step sets.
- `review_perf_log.py`: append/prune/validate helpers for `.logs/review.jsonl` events.
- `analyze_review_perf.py`: summarizes historical review perf events and supports backfill analysis.
- `profile_cargo_test.py`: profiles cargo test execution and emits per-binary/per-case performance events.
- `profile_benchmark_binary.py`: builds/locates benchmark binaries and supports optional `samply` profiling.
- `tests/test_review_scripts.py`: Python unit tests for review script modules and planner behavior.

## Entry Points

The root `justfile` invokes these scripts directly:

- `just review` -> `python3 scripts/review/review_runner.py`
- `just review-scope-plan` -> `python3 scripts/review/review_scope.py plan`
- `just review-scope-validate` -> `python3 scripts/review/review_scope.py validate`
- `just review-analyze` -> `python3 scripts/review/analyze_review_perf.py`

`scripts/review/profile_cargo_test.py` emits test-level performance events via `review_perf_log.py`.
`just review` runs `python-test`, which executes `python3 -m unittest discover -s scripts/review/tests -p "test_*.py"` when Python domain scope is impacted (or in forced full mode).

## Conventions

- Keep review-only automation in this directory.
- Update `review_scope_config.json` path triggers if files are added, renamed, or moved.
- Prefer `just review-scope-plan` and `just review-scope-validate` when changing planner logic.
