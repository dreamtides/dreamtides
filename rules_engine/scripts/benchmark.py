#!/usr/bin/env python3

import argparse
import json
import os
import subprocess
import sys
import tempfile


# Mapping of benchmark names to their maximum allowed runtimes in nanoseconds
MAX_ALLOWED_TIMES = {
    "random_playout/random_playout": 50_000_000,  # 50ms
    "uct1_first_action/uct1_first_action": 30_000_000,  # 30ms
    "uct_1k_action/uct_1k_action": 20_000_000,  # 20ms
    "uct_50k_action/uct_50k_action": 500_000_000,  # 500ms
    "uct_single_threaded/uct_single_threaded": 1_000_000,  # 800ms
    "parse_abilities/parse_abilities": 10_000_000,  # 10ms
}


def run_benchmark(benchmark_name):
    """Runs the specified benchmark using cargo criterion and checks the results."""
    if benchmark_name not in MAX_ALLOWED_TIMES:
        print(f"Error: Unknown benchmark '{benchmark_name}'")
        print(f"Known benchmarks: {', '.join(MAX_ALLOWED_TIMES.keys())}")
        return 1

    max_allowed_time = MAX_ALLOWED_TIMES[benchmark_name]
    
    # Create temporary file for JSON output
    with tempfile.NamedTemporaryFile(delete=False) as temp_file:
        temp_file_path = temp_file.name

    try:
        # Run the benchmark, redirecting stdout to the temp file
        cmd = ["cargo", "criterion", benchmark_name, "--message-format=json"]
        process = subprocess.run(
            cmd, 
            stdout=open(temp_file_path, "w"),
            stderr=subprocess.PIPE,
            text=True,
        )
        
        # Print stderr output to console, regardless of success or failure of the command
        if process.stderr:
            print(process.stderr, file=sys.stderr, end="")

        # Check if the cargo criterion command failed
        if process.returncode != 0:
            print(f"Error: 'cargo criterion {benchmark_name}' failed with exit code {process.returncode}.", file=sys.stderr)
            return 1 # Indicate failure
        
        # Parse the JSON output to get the benchmark results
        benchmark_time = parse_benchmark_results(temp_file_path, benchmark_name)
        
        # Check if the benchmark time exceeds the maximum allowed time
        if benchmark_time is None:
            print(f"Error: Could not find benchmark results for '{benchmark_name}'")
            return 1
            
        if benchmark_time > max_allowed_time:
            print(f"ERROR: Benchmark '{benchmark_name}' exceeded maximum allowed time!")
            print(f"  Actual time:   {benchmark_time:,} ns ({benchmark_time/1_000_000:.2f} ms)")
            print(f"  Maximum time:  {max_allowed_time:,} ns ({max_allowed_time/1_000_000:.2f} ms)")
            print(f"  Difference:    {benchmark_time - max_allowed_time:,} ns ({(benchmark_time - max_allowed_time)/1_000_000:.2f} ms)")
            return 1
        else:
            print(f"SUCCESS: Benchmark '{benchmark_name}' completed within allowed time")
            print(f"  Actual time:   {benchmark_time:,} ns ({benchmark_time/1_000_000:.2f} ms)")
            print(f"  Maximum time:  {max_allowed_time:,} ns ({max_allowed_time/1_000_000:.2f} ms)")
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
        if char == '{':
            if brace_count == 0:
                start_index = i
            brace_count += 1
        elif char == '}':
            brace_count -= 1
            if brace_count == 0:
                json_objects.append(content[start_index:i+1])
    
    # Parse each JSON object and look for the benchmark result
    for json_str in json_objects:
        try:
            data = json.loads(json_str)
            
            # Check if this is a benchmark completion object for our benchmark
            if (data.get("reason") == "benchmark-complete" and 
                data.get("id") == benchmark_name):
                
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
    args = parser.parse_args()
    
    return run_benchmark(args.benchmark)


if __name__ == "__main__":
    sys.exit(main())
