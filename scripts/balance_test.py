#!/usr/bin/env python3
"""Parallel balance mode testing for Dreamtides first-player advantage mitigation.

Runs AI matchups across 6 balance modes in parallel and compares P1/P2 winrates.

Usage:
    python3 scripts/balance_test.py          # Full run: 50 matches per mode, MonteCarloV8(50)
    python3 scripts/balance_test.py --smoke  # Quick test: 1 match per mode, MonteCarloV8(5)
    python3 scripts/balance_test.py --matches 20 --ai '{"MonteCarloV8":30}'
    python3 scripts/balance_test.py --no-build --mode bonus-energy-no-draw
"""

import argparse
import os
import re
import subprocess
import sys
import time
from datetime import datetime
from pathlib import Path

BALANCE_MODES = [
    "none",
    "extra-card",
    "bonus-energy",
    "bonus-energy-no-draw",
    "four-six-cards",
    "bonus-points",
    "no-sickness",
    "coin",
]
BASE_SEED = 3141592653
SEED_OFFSETS = {
    "none": 0,
    "extra-card": 1000000,
    "bonus-energy": 2000000,
    "bonus-energy-no-draw": 3000000,
    "four-six-cards": 4000000,
    "bonus-points": 5000000,
    "no-sickness": 6000000,
    "coin": 7000000,
}

PROJECT_ROOT = Path(__file__).resolve().parent.parent
MATCHUP_BINARY = PROJECT_ROOT / "rules_engine" / "target" / "release" / "run_matchup"


def build_binary():
    print("Building run_matchup binary (release)...")
    result = subprocess.run(
        [
            "cargo",
            "build",
            "--manifest-path",
            str(PROJECT_ROOT / "rules_engine" / "Cargo.toml"),
            "--release",
            "--bin",
            "run_matchup",
        ],
        capture_output=True,
        text=True,
    )
    if result.returncode != 0:
        print(f"Build failed:\n{result.stderr}", file=sys.stderr)
        sys.exit(1)
    print("Build complete.")


def parse_results(output: str):
    """Parse the match results summary from run_matchup stdout."""
    p1_wins = 0
    p2_wins = 0
    draws = 0
    total = 0

    # Multi-match summary format
    p1_match = re.search(r"Player One wins:\s*(\d+)", output)
    p2_match = re.search(r"Player Two wins:\s*(\d+)", output)
    total_match = re.search(r"Total matches:\s*(\d+)", output)
    draws_match = re.search(r"Timed-out matches:\s*(\d+)", output)

    if p1_match:
        p1_wins = int(p1_match.group(1))
    if p2_match:
        p2_wins = int(p2_match.group(1))
    if total_match:
        total = int(total_match.group(1))
    if draws_match:
        draws = int(draws_match.group(1))

    # Fallback: parse per-match Progress lines (works for any match count)
    if p1_wins == 0 and p2_wins == 0:
        progress_matches = re.findall(r"Progress: P1:(\d+) P2:(\d+)", output)
        if progress_matches:
            last = progress_matches[-1]
            p1_wins = int(last[0])
            p2_wins = int(last[1])
            total = p1_wins + p2_wins + draws

    return {"p1_wins": p1_wins, "p2_wins": p2_wins, "draws": draws, "total": total}


def read_progress(log_path: Path):
    """Read the latest Progress line from a log file to get current P1/P2 wins."""
    try:
        text = log_path.read_text()
    except (FileNotFoundError, OSError):
        return None
    matches = re.findall(r"Progress: P1:(\d+) P2:(\d+)", text)
    if matches:
        last = matches[-1]
        return int(last[0]), int(last[1])
    return None


def format_elapsed(seconds: float) -> str:
    """Format elapsed seconds as Xm Ys."""
    m = int(seconds) // 60
    s = int(seconds) % 60
    return f"{m}m{s:02d}s"


def print_progress_table(
    modes: list[str], processes: dict, results: dict, matches: int, start_time: float
):
    """Print a compact progress table for all active/completed modes."""
    elapsed = time.time() - start_time
    print(f"\n--- Progress ({format_elapsed(elapsed)} elapsed) ---")
    all_modes = list(results.keys()) + list(processes.keys())
    for mode in modes:
        if mode not in all_modes:
            continue
        if mode in results:
            r = results[mode]
            completed = r["p1_wins"] + r["p2_wins"] + r["draws"]
            p1_pct = (
                f"{r['p1_wins'] / (r['p1_wins'] + r['p2_wins']) * 100:.0f}%"
                if r["p1_wins"] + r["p2_wins"] > 0
                else "N/A"
            )
            print(f"  {mode:14s}  DONE  {completed}/{matches}  P1: {p1_pct}")
        elif mode in processes:
            info = processes[mode]
            progress = read_progress(info["log_path"])
            if progress:
                p1, p2 = progress
                completed = p1 + p2
                p1_pct = f"{p1 / completed * 100:.0f}%" if completed > 0 else "N/A"
                print(f"  {mode:14s}  ...   {completed}/{matches}  P1: {p1_pct}")
            else:
                print(f"  {mode:14s}  ...   0/{matches}")
        else:
            print(f"  {mode:14s}  ???")
    sys.stdout.flush()


def launch_mode(mode: str, matches: int, ai_config: str, output_dir: Path):
    """Launch a single balance mode test process."""
    log_path = output_dir / f"{mode}.log"
    seed = BASE_SEED + SEED_OFFSETS[mode]
    cmd = [
        str(MATCHUP_BINARY),
        ai_config,
        ai_config,
        "--seed",
        str(seed),
        "--matches",
        str(matches),
        "--balance",
        mode,
        "--deck",
        "core11",
        "-v",
        "one-line",
    ]

    log_file = open(log_path, "w")
    proc = subprocess.Popen(
        cmd,
        stdout=log_file,
        stderr=subprocess.STDOUT,
        text=True,
    )
    return {
        "proc": proc,
        "log_file": log_file,
        "log_path": log_path,
        "start": time.time(),
    }


def run_balance_tests(
    matches: int,
    ai_config: str,
    output_dir: Path,
    concurrency: int,
    modes: list[str],
):
    """Launch balance mode tests with limited concurrency and collect results."""
    global_start = time.time()
    pending = list(modes)
    active = {}
    results = {}
    last_progress_print = 0

    print(
        f"Running {len(modes)} balance test groups ({matches} matches each, {concurrency} concurrent)..."
    )
    print(f"Logs: {output_dir.relative_to(PROJECT_ROOT)}/\n")

    while pending or active:
        # Launch new processes up to concurrency limit
        while pending and len(active) < concurrency:
            mode = pending.pop(0)
            info = launch_mode(mode, matches, ai_config, output_dir)
            active[mode] = info
            print(f"  [{mode:14s}] STARTED  PID {info['proc'].pid}")
            sys.stdout.flush()

        # Check for completed processes
        for mode in list(active.keys()):
            info = active[mode]
            ret = info["proc"].poll()
            if ret is not None:
                elapsed = time.time() - info["start"]
                info["log_file"].close()
                output = info["log_path"].read_text()
                parsed = parse_results(output)
                results[mode] = parsed

                completed = parsed["p1_wins"] + parsed["p2_wins"]
                if ret != 0:
                    status = f"FAILED (exit {ret})"
                elif completed > 0:
                    p1_pct = parsed["p1_wins"] / completed * 100
                    status = f"P1: {p1_pct:.1f}% ({completed} matches)"
                else:
                    status = "no completed matches"
                remaining = len(pending) + len(active) - 1
                print(
                    f"  [{mode:14s}] DONE in {format_elapsed(elapsed)} -- {status}  ({remaining} remaining)"
                )
                sys.stdout.flush()

                del active[mode]

        now = time.time()
        if active and now - last_progress_print >= 30:
            print_progress_table(modes, active, results, matches, global_start)
            last_progress_print = now

        if active:
            time.sleep(5)

    return results


def print_summary(results: dict, modes: list[str]):
    """Print a formatted comparison table."""
    print("\n===== Balance Test Results =====")
    print(
        f"{'Mode':15s} | {'P1 Wins':>7s} | {'P2 Wins':>7s} | {'P1%':>6s} | {'P2%':>6s} | {'Delta from 50%':>14s}"
    )
    print(
        "-" * 15
        + "-|-"
        + "-" * 7
        + "-|-"
        + "-" * 7
        + "-|-"
        + "-" * 6
        + "-|-"
        + "-" * 6
        + "-|-"
        + "-" * 14
    )

    for mode in modes:
        if mode not in results:
            print(
                f"{mode:15s} | {'N/A':>7s} | {'N/A':>7s} | {'N/A':>6s} | {'N/A':>6s} | {'N/A':>14s}"
            )
            continue

        r = results[mode]
        completed = r["p1_wins"] + r["p2_wins"]
        if completed == 0:
            print(
                f"{mode:15s} | {0:>7d} | {0:>7d} | {'N/A':>6s} | {'N/A':>6s} | {'N/A':>14s}"
            )
            continue

        p1_pct = r["p1_wins"] / completed * 100
        p2_pct = r["p2_wins"] / completed * 100
        delta = p1_pct - 50.0
        delta_str = f"+{delta:.1f}%" if delta >= 0 else f"{delta:.1f}%"
        print(
            f"{mode:15s} | {r['p1_wins']:>7d} | {r['p2_wins']:>7d} | {p1_pct:>5.1f}% | {p2_pct:>5.1f}% | {delta_str:>14s}"
        )

    if (
        results.get("none")
        and results["none"]["p1_wins"] + results["none"]["p2_wins"] > 0
    ):
        control_completed = results["none"]["p1_wins"] + results["none"]["p2_wins"]
        control_p1 = results["none"]["p1_wins"] / control_completed * 100
        print(f"\nControl (none) P1 winrate: {control_p1:.1f}%")
        print("Lower delta = better balance (0% = perfectly balanced)")


def parse_args(argv: list[str] | None = None):
    parser = argparse.ArgumentParser(
        description="Parallel balance mode testing for Dreamtides"
    )
    parser.add_argument(
        "--smoke",
        action="store_true",
        help="Quick smoke test: 1 match per mode with MonteCarloV8(5)",
    )
    parser.add_argument(
        "--matches", type=int, default=50, help="Matches per balance mode (default: 50)"
    )
    parser.add_argument(
        "--ai",
        type=str,
        default='{"MonteCarloV8":50}',
        help='AI config JSON (default: {"MonteCarloV8":50})',
    )
    parser.add_argument(
        "--concurrency",
        type=int,
        default=2,
        help="Max concurrent processes (default: 2)",
    )
    parser.add_argument(
        "--skip",
        type=str,
        nargs="+",
        default=[],
        help="Balance modes to skip (e.g. --skip none extra-card)",
    )
    parser.add_argument(
        "--mode",
        choices=BALANCE_MODES,
        help="Run only a single balance mode",
    )
    parser.add_argument(
        "--no-build",
        action="store_true",
        help="Skip building the binary (use existing release build)",
    )
    return parser.parse_args(argv)


def selected_modes(mode: str | None, skip: list[str]) -> list[str]:
    """Return the ordered set of balance modes to execute."""
    if mode is not None:
        if mode in skip:
            print(f"Selected mode '{mode}' cannot also be skipped", file=sys.stderr)
            sys.exit(1)
        return [mode]
    return [balance_mode for balance_mode in BALANCE_MODES if balance_mode not in skip]


def main():
    args = parse_args()

    if args.smoke:
        matches = 1
        ai_config = '{"MonteCarloV8":5}'
        concurrency = 6  # smoke is fast, run all at once
        print("=== SMOKE TEST MODE ===")
    else:
        matches = args.matches
        ai_config = args.ai
        concurrency = args.concurrency

    skip = args.skip
    for s in skip:
        if s not in BALANCE_MODES:
            print(f"Unknown mode to skip: {s}", file=sys.stderr)
            print(f"Valid modes: {', '.join(BALANCE_MODES)}", file=sys.stderr)
            sys.exit(1)

    modes = selected_modes(args.mode, skip)

    print(f"AI: {ai_config}")
    print(f"Matches per mode: {matches}")
    print(f"Concurrency: {concurrency}")
    print(f"Modes: {', '.join(modes)}")

    if args.no_build:
        if not MATCHUP_BINARY.exists():
            print(f"Binary not found at {MATCHUP_BINARY}", file=sys.stderr)
            sys.exit(1)
        print(f"Using existing binary: {MATCHUP_BINARY}")
    else:
        build_binary()

    timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
    output_dir = PROJECT_ROOT / "scripts" / "balance_test_output" / f"run_{timestamp}"
    output_dir.mkdir(parents=True, exist_ok=True)

    results = run_balance_tests(matches, ai_config, output_dir, concurrency, modes)
    print_summary(results, modes)


if __name__ == "__main__":
    main()
