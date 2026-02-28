#!/usr/bin/env python3

import argparse
import json
import os
import subprocess
import sys
import tempfile


# ANSI color codes
class Colors:
    GREEN = "\033[92m"
    RED = "\033[91m"
    RESET = "\033[0m"


def print_colored(text, color):
    """Print text in the specified color."""
    print(f"{color}{text}{Colors.RESET}")


def get_project_root():
    """Get the project root directory based on the script's location."""
    script_dir = os.path.dirname(os.path.abspath(__file__))
    return os.path.dirname(script_dir)  # Go up one level from scripts/


def run_benchmark(benchmark_name, max_time_ms):
    """Runs the specified benchmark using cargo criterion and checks the results."""
    max_allowed_time = max_time_ms * 1_000_000  # Convert ms to ns

    # Change to project root directory
    project_root = get_project_root()
    os.chdir(project_root)

    # Create temporary file for JSON output
    with tempfile.NamedTemporaryFile(delete=False) as temp_file:
        temp_file_path = temp_file.name

    try:
        cmd = ["cargo", "criterion", benchmark_name, "--message-format=json"]
        process = subprocess.Popen(
            cmd,
            stdout=open(temp_file_path, "w"),
            stderr=subprocess.PIPE,
            text=True,
            bufsize=1,  # Line buffered
            universal_newlines=True,
        )

        # Stream stderr output in real-time
        while True:
            line = process.stderr.readline()
            if not line and process.poll() is not None:
                break
            if line:
                print(line, file=sys.stderr, end="")

        # Get the final return code
        return_code = process.wait()

        # Check if the cargo criterion command failed
        if return_code != 0:
            print_colored(
                f"Error: 'cargo criterion {benchmark_name}' failed with exit code {return_code}.",
                Colors.RED,
            )
            return 1  # Indicate failure

        # Parse the JSON output to get the benchmark results
        benchmark_time = parse_benchmark_results(temp_file_path, benchmark_name)

        # Check if the benchmark time exceeds the maximum allowed time
        if benchmark_time is None:
            print_colored(
                f"Error: Could not find benchmark results for '{benchmark_name}'",
                Colors.RED,
            )
            return 1

        if benchmark_time > max_allowed_time:
            print_colored(
                f"ERROR: Benchmark '{benchmark_name}' exceeded maximum allowed time!",
                Colors.RED,
            )
            print(
                f"  Actual time:   {benchmark_time:,} ns ({benchmark_time/1_000_000:.2f} ms)"
            )
            print(
                f"  Maximum time:  {max_allowed_time:,} ns ({max_allowed_time/1_000_000:.2f} ms)"
            )
            print(
                f"  Difference:    {benchmark_time - max_allowed_time:,} ns ({(benchmark_time - max_allowed_time)/1_000_000:.2f} ms)"
            )
            return 1
        else:
            print_colored(
                f"SUCCESS: Benchmark '{benchmark_name}' completed within allowed time",
                Colors.GREEN,
            )
            print(
                f"  Actual time:   {benchmark_time:,} ns ({benchmark_time/1_000_000:.2f} ms)"
            )
            print(
                f"  Maximum time:  {max_allowed_time:,} ns ({max_allowed_time/1_000_000:.2f} ms)"
            )
            return 0

    finally:
        # Clean up temporary file
        if os.path.exists(temp_file_path):
            os.unlink(temp_file_path)


def parse_benchmark_results(json_file_path, benchmark_name):
    """Parses the JSON output from cargo criterion to extract the benchmark time."""
    with open(json_file_path, "r") as f:
        content = f.read()

    # The file contains multiple JSON objects, so we need to parse them separately
    benchmark_time = None

    # Split the content at each opening brace and parse each object
    json_objects = []
    brace_count = 0
    start_index = 0

    for i, char in enumerate(content):
        if char == "{":
            if brace_count == 0:
                start_index = i
            brace_count += 1
        elif char == "}":
            brace_count -= 1
            if brace_count == 0:
                json_objects.append(content[start_index : i + 1])

    # Parse each JSON object and look for the benchmark result
    for json_str in json_objects:
        try:
            data = json.loads(json_str)

            # Check if this is a benchmark completion object for our benchmark
            if (
                data.get("reason") == "benchmark-complete"
                and data.get("id") == benchmark_name
            ):

                # Extract the typical estimate value in nanoseconds
                if "typical" in data and "estimate" in data["typical"]:
                    benchmark_time = data["typical"]["estimate"]
                    break

        except json.JSONDecodeError:
            continue

    return benchmark_time


def main():
    parser = argparse.ArgumentParser(description="Run and check Rust benchmarks.")
    parser.add_argument("benchmark", help="Name of the benchmark to run")
    parser.add_argument(
        "--maximum-time-ms",
        type=int,
        required=True,
        help="Maximum allowed time for the benchmark in milliseconds",
    )
    args = parser.parse_args()

    return run_benchmark(args.benchmark, args.maximum_time_ms)


if __name__ == "__main__":
    sys.exit(main())
