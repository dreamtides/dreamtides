#!/usr/bin/env bash
# Helper script to run cargo tests with filter validation.
# Fails if a filter is provided but no tests match.
#
# Usage: run_cargo_test.sh <package> [test_filter_args...]
# Example: run_cargo_test.sh parser_v2_tests my_test_name
#
# Environment variables:
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

# Build the cargo test command
CARGO_ARGS="--manifest-path rules_engine/Cargo.toml -p $PACKAGE"

if [ -n "${CARGO_TEST_QUIET:-}" ]; then
    CARGO_ARGS="$CARGO_ARGS -q"
fi

# Build test args (after --)
TEST_ARGS=""
if [ -n "${CARGO_TEST_THREADS:-}" ]; then
    TEST_ARGS="--test-threads=$CARGO_TEST_THREADS"
fi

# Add filter args if provided
FILTER_ARGS=""
if [ $# -gt 0 ]; then
    FILTER_ARGS="$*"
fi

# Combine test args
if [ -n "$TEST_ARGS" ] || [ -n "$FILTER_ARGS" ]; then
    CARGO_ARGS="$CARGO_ARGS -- $TEST_ARGS $FILTER_ARGS"
fi

output=$(cargo test $CARGO_ARGS 2>&1) || {
    echo "$output"
    exit 1
}

# If a filter was provided, check that at least one test ran
if [ -n "$FILTER_ARGS" ]; then
    total_passed=$(echo "$output" | grep -E "^test result:" | sed 's/.*\. \([0-9]*\) passed.*/\1/' | awk '{sum+=$1} END {print sum+0}')
    if [ "$total_passed" -eq 0 ]; then
        echo "Error: No tests matched filter '$FILTER_ARGS'"
        echo "This usually means the test doesn't exist or isn't being compiled."
        exit 1
    fi
fi

echo "Success"
