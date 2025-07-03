#!/usr/bin/env python3

import argparse
import json
import os
import subprocess
import sys
import tempfile
import statistics
from collections import defaultdict
from datetime import datetime


# ANSI color codes
class Colors:
    GREEN = '\033[92m'
    RED = '\033[91m'
    YELLOW = '\033[93m'
    BLUE = '\033[94m'
    MAGENTA = '\033[95m'
    CYAN = '\033[96m'
    BOLD = '\033[1m'
    RESET = '\033[0m'


def print_colored(text, color):
    """Print text in the specified color."""
    print(f"{color}{text}{Colors.RESET}")


def format_time(ns):
    """Format nanoseconds into human-readable time."""
    if ns >= 1_000_000_000:
        return f"{ns/1_000_000_000:.3f} s"
    elif ns >= 1_000_000:
        return f"{ns/1_000_000:.3f} ms"
    elif ns >= 1_000:
        return f"{ns/1_000:.3f} µs"
    else:
        return f"{ns:.0f} ns"


def calculate_confidence_interval(data, confidence=0.95):
    """Calculate confidence interval using simple method."""
    n = len(data)
    if n < 2:
        return None, None
    
    mean = statistics.mean(data)
    std_dev = statistics.stdev(data)
    std_error = std_dev / (n ** 0.5)
    
    # Using z-score for 95% confidence (1.96)
    z_score = 1.96 if confidence == 0.95 else 2.576  # 99% confidence
    margin = z_score * std_error
    
    return mean - margin, mean + margin


def get_project_root():
    """Get the project root directory based on the script's location."""
    script_dir = os.path.dirname(os.path.abspath(__file__))
    return os.path.dirname(script_dir)  # Go up one level from scripts/


def parse_benchmark_output(json_file_path):
    """Parse the JSON output from cargo criterion to extract benchmark results."""
    results = {}
    
    with open(json_file_path, "r") as f:
        content = f.read()
    
    # Split into individual JSON objects
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
    
    # Parse each JSON object
    for json_str in json_objects:
        try:
            data = json.loads(json_str)
            
            if data.get("reason") == "benchmark-complete":
                benchmark_id = data.get("id")
                if benchmark_id:
                    results[benchmark_id] = {
                        'typical': data.get('typical', {}).get('estimate'),
                        'mean': data.get('mean', {}).get('estimate'),
                        'median': data.get('median', {}).get('estimate'),
                        'median_abs_dev': data.get('median_abs_dev', {}).get('estimate'),
                        'lower_bound': data.get('typical', {}).get('lower_bound'),
                        'upper_bound': data.get('typical', {}).get('upper_bound'),
                    }
                    
        except json.JSONDecodeError:
            continue
    
    return results


def run_single_benchmark(benchmark_name, extra_args):
    """Run a single benchmark and return the results."""
    project_root = get_project_root()
    os.chdir(project_root)
    
    with tempfile.NamedTemporaryFile(delete=False) as temp_file:
        temp_file_path = temp_file.name

    try:
        cmd = ["cargo", "criterion", benchmark_name, "--message-format=json"] + extra_args
        
        # Run the benchmark
        process = subprocess.Popen(
            cmd,
            stdout=open(temp_file_path, "w"),
            stderr=subprocess.PIPE,
            text=True,
            bufsize=1,
            universal_newlines=True
        )
        
        # Stream stderr output in real-time
        while True:
            line = process.stderr.readline()
            if not line and process.poll() is not None:
                break
            if line:
                print(line, end="")
        
        return_code = process.wait()
        
        if return_code != 0:
            print_colored(f"Error: cargo criterion failed with exit code {return_code}", Colors.RED)
            return None
            
        # Parse results
        return parse_benchmark_output(temp_file_path)
        
    finally:
        if os.path.exists(temp_file_path):
            os.unlink(temp_file_path)


def print_run_summary(run_number, results):
    """Print a summary of a single benchmark run."""
    print(f"\n{Colors.BOLD}=== Run {run_number} Results ==={Colors.RESET}")
    
    if not results:
        print_colored("No results found", Colors.RED)
        return
    
    # Table header
    print(f"\n{'Benchmark':<40} {'Estimate':<15} {'Lower Bound':<15} {'Upper Bound':<15} {'Std Dev':<15}")
    print("-" * 100)
    
    for bench_id, data in results.items():
        estimate = data.get('typical') or data.get('mean')
        lower = data.get('lower_bound')
        upper = data.get('upper_bound')
        mad = data.get('median_abs_dev', 0)
        
        if estimate:
            print(f"{bench_id:<40} {format_time(estimate):<15} "
                  f"{format_time(lower) if lower else 'N/A':<15} "
                  f"{format_time(upper) if upper else 'N/A':<15} "
                  f"{format_time(mad):<15}")


def analyze_variance(all_runs, benchmark_name):
    """Analyze variance across all benchmark runs."""
    print(f"\n{Colors.BOLD}{Colors.CYAN}=== Variance Analysis for {benchmark_name} ==={Colors.RESET}")
    
    # Collect all estimates for each benchmark
    benchmark_times = defaultdict(list)
    
    for run_results in all_runs:
        for bench_id, data in run_results.items():
            estimate = data.get('typical') or data.get('mean')
            if estimate:
                benchmark_times[bench_id].append(estimate)
    
    if not benchmark_times:
        print_colored("No benchmark data collected", Colors.RED)
        return
    
    # Analyze each benchmark
    for bench_id, times in benchmark_times.items():
        print(f"\n{Colors.BOLD}Benchmark: {bench_id}{Colors.RESET}")
        print(f"Number of runs: {len(times)}")
        
        if len(times) < 2:
            print("Not enough data for statistical analysis")
            continue
        
        # Calculate statistics
        mean = statistics.mean(times)
        median = statistics.median(times)
        std_dev = statistics.stdev(times)
        min_time = min(times)
        max_time = max(times)
        cv = (std_dev / mean) * 100  # Coefficient of variation in %
        
        # Confidence intervals
        ci_95_low, ci_95_high = calculate_confidence_interval(times, 0.95)
        ci_99_low, ci_99_high = calculate_confidence_interval(times, 0.99)
        
        # Print results
        print(f"\n{Colors.GREEN}Summary Statistics:{Colors.RESET}")
        print(f"  Mean:           {format_time(mean)}")
        print(f"  Median:         {format_time(median)}")
        print(f"  Std Deviation:  {format_time(std_dev)} ({cv:.2f}% CV)")
        print(f"  Min:            {format_time(min_time)}")
        print(f"  Max:            {format_time(max_time)}")
        print(f"  Range:          {format_time(max_time - min_time)}")
        
        print(f"\n{Colors.BLUE}Confidence Intervals:{Colors.RESET}")
        if ci_95_low and ci_95_high:
            print(f"  95% CI:         [{format_time(ci_95_low)}, {format_time(ci_95_high)}]")
        if ci_99_low and ci_99_high:
            print(f"  99% CI:         [{format_time(ci_99_low)}, {format_time(ci_99_high)}]")
        
        # Reliability assessment
        print(f"\n{Colors.MAGENTA}Reliability Assessment:{Colors.RESET}")
        if cv < 5:
            print_colored("  ✓ Excellent reliability (CV < 5%)", Colors.GREEN)
        elif cv < 10:
            print_colored("  ✓ Good reliability (CV < 10%)", Colors.GREEN)
        elif cv < 20:
            print_colored("  ⚠ Moderate reliability (CV < 20%)", Colors.YELLOW)
        else:
            print_colored("  ✗ Poor reliability (CV >= 20%)", Colors.RED)
            
        # Distribution info
        spread = (max_time - min_time) / mean * 100
        print(f"\n  Min-Max spread: {spread:.1f}% of mean")
        
        # Show individual run times
        print(f"\n{Colors.CYAN}Individual run times:{Colors.RESET}")
        for i, time in enumerate(times, 1):
            deviation = ((time - mean) / mean) * 100
            sign = "+" if deviation >= 0 else ""
            print(f"  Run {i:2d}: {format_time(time):>12} ({sign}{deviation:>6.2f}% from mean)")


def main():
    parser = argparse.ArgumentParser(
        description="Run Rust benchmarks repeatedly to analyze variance",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  %(prog)s -n 10 "ai_single_threaded/ai_single_threaded"
  %(prog)s -n 5 "my_bench" -- --sample-size 50
  %(prog)s -n 20 "bench_name" -- --warm-up-time 5
        """
    )
    
    parser.add_argument(
        "benchmark",
        help="Name of the benchmark to run"
    )
    parser.add_argument(
        "-n", "--num-runs",
        type=int,
        default=5,
        help="Number of times to run the benchmark (default: 5)"
    )
    parser.add_argument(
        "extra_args",
        nargs=argparse.REMAINDER,
        help="Additional arguments to pass to cargo criterion (after --)"
    )
    
    args = parser.parse_args()
    
    # Filter out the -- separator if present
    extra_args = [arg for arg in args.extra_args if arg != "--"]
    
    print_colored(f"Running benchmark '{args.benchmark}' {args.num_runs} times...", Colors.BOLD)
    if extra_args:
        print(f"Extra criterion args: {' '.join(extra_args)}")
    
    all_runs = []
    
    # Run benchmarks
    for i in range(1, args.num_runs + 1):
        print(f"\n{Colors.YELLOW}Starting run {i}/{args.num_runs}...{Colors.RESET}")
        
        results = run_single_benchmark(args.benchmark, extra_args)
        if results:
            all_runs.append(results)
            print_run_summary(i, results)
        else:
            print_colored(f"Run {i} failed", Colors.RED)
    
    # Analyze variance
    if all_runs:
        analyze_variance(all_runs, args.benchmark)
    else:
        print_colored("\nNo successful runs to analyze", Colors.RED)
        return 1
    
    return 0


if __name__ == "__main__":
    sys.exit(main())
