#!/usr/bin/env bash
# Helper script to run cargo tests with filter validation.
# Fails if a filter is provided but no tests match.
#
# Usage: run_cargo_test.sh <package> [test_filter_args...]
# Example: run_cargo_test.sh parser_v2_tests my_test_name
#
# Environment variables:
#   REVIEW_PERF - If set to 1, route through review/profile_cargo_test.py
#   RUST_MIN_STACK - Set minimum stack size (e.g., for deep parser hierarchies)
#   CARGO_TEST_QUIET - If set, pass -q to cargo test
#   CARGO_TEST_THREADS - If set, pass --test-threads=N to cargo test

set -euo pipefail

if [ $# -lt 1 ]; then
    echo "Usage: $0 <package> [test_filter_args...]"
    exit 1
fi

PACKAGE="$1"
shift

FILTER_ARGS=()
for arg in "$@"; do
    if [ -n "$arg" ]; then
        FILTER_ARGS+=("$arg")
    fi
done

if [ "${REVIEW_PERF:-0}" = "1" ]; then
    profiler_args=(
        --manifest-path rules_engine/Cargo.toml
        --package "$PACKAGE"
        --require-match
    )

    if [ -n "${CARGO_TEST_QUIET:-}" ]; then
        profiler_args+=(--quiet)
    fi

    if [ -n "${CARGO_TEST_THREADS:-}" ]; then
        profiler_args+=(--test-threads "$CARGO_TEST_THREADS")
    fi

    if [ "${#FILTER_ARGS[@]}" -gt 0 ]; then
        profiler_args+=("${FILTER_ARGS[@]}")
    fi

    if ! python3 scripts/review/profile_cargo_test.py "${profiler_args[@]}"; then
        exit 1
    fi

    exit 0
fi

# Legacy path
CARGO_ARGS=(--manifest-path rules_engine/Cargo.toml -p "$PACKAGE")

if [ -n "${CARGO_TEST_QUIET:-}" ]; then
    CARGO_ARGS+=(-q)
fi

TEST_ARGS=()
if [ -n "${CARGO_TEST_THREADS:-}" ]; then
    TEST_ARGS+=("--test-threads=$CARGO_TEST_THREADS")
fi

if [ "${#TEST_ARGS[@]}" -gt 0 ] || [ "${#FILTER_ARGS[@]}" -gt 0 ]; then
    CARGO_ARGS+=(--)
    if [ "${#TEST_ARGS[@]}" -gt 0 ]; then
        CARGO_ARGS+=("${TEST_ARGS[@]}")
    fi
    if [ "${#FILTER_ARGS[@]}" -gt 0 ]; then
        CARGO_ARGS+=("${FILTER_ARGS[@]}")
    fi
fi

output=$(cargo test "${CARGO_ARGS[@]}" 2>&1) || {
    echo "$output"
    exit 1
}

if [ "${#FILTER_ARGS[@]}" -gt 0 ]; then
    total_passed=$(echo "$output" | grep -E "^test result:" | sed 's/.*\. \([0-9]*\) passed.*/\1/' | awk '{sum+=$1} END {print sum+0}')
    if [ "$total_passed" -eq 0 ]; then
        echo "Error: No tests matched filter '$*'"
        echo "This usually means the test doesn't exist or isn't being compiled."
        exit 1
    fi
fi

echo "Success"
