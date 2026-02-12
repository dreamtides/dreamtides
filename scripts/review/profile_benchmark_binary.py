#!/usr/bin/env python3
"""Locate the compiled Criterion benchmark binary for a given benchmark filter.

Usage example:
  python scripts/review/profile_benchmark_binary.py \
	  --benchmark ai_core_11/ai_core_11 \
	  --package battle_benchmarks \
	  --manifest-path benchmarks/battle/Cargo.toml \
	  --samply

Prints the absolute path to the executable benchmark binary to stdout.

Notes:
* We invoke `cargo criterion --no-run` to ensure the bench target is built.
* The resulting executable is typically in: target/release/deps/<bench_name>-<hash>
* We determine the bench target name via `cargo metadata` (the bench target whose
  name matches the provided package OR the first bench target if ambiguous).
* We choose the most recently modified matching executable (excluding *.d, *.rlib, etc.).
"""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
from pathlib import Path
from typing import Optional, List


def run(cmd: List[str], cwd: Optional[str] = None) -> subprocess.CompletedProcess:
	return subprocess.run(cmd, cwd=cwd, check=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)


def cargo_metadata(manifest_path: str) -> dict:
	cmd = ["cargo", "metadata", "--format-version", "1", "--no-deps", "--manifest-path", manifest_path]
	return json.loads(run(cmd).stdout)


def find_bench_target(metadata: dict, package_name: str) -> str:
	for pkg in metadata.get("packages", []):
		if pkg.get("name") == package_name:
			benches = [t for t in pkg.get("targets", []) if "bench" in t.get("kind", [])]
			if not benches:
				raise SystemExit(f"No bench targets found in package '{package_name}'")
			if len(benches) == 1:
				return benches[0]["name"]
			# Prefer a bench target with same name as package; else first.
			for t in benches:
				if t.get("name") == package_name:
					return t["name"]
			return benches[0]["name"]
	raise SystemExit(f"Package '{package_name}' not found in metadata")


def build_benchmark(manifest_path: str, package: str, benchmark_filter: str) -> None:
	# Use cargo criterion for consistent build flags with existing workflow.
	cmd = [
		"cargo",
		"criterion",
		"--no-run",
		"--manifest-path",
		manifest_path,
		"-p",
		package,
		"--",
		benchmark_filter,
	]
	# We don't care about stdout; failures will raise.
	run(cmd)


def candidate_paths(target_dir: Path, bench_target: str) -> List[Path]:
	deps_dir = target_dir / "release" / "deps"
	if not deps_dir.is_dir():
		return []
	return [p for p in deps_dir.glob(f"{bench_target}-*") if p.is_file()]


def is_executable_candidate(path: Path) -> bool:
	if path.suffix in {".d", ".rlib", ".rmeta"}:
		return False
	name = path.name
	if name.endswith(".d") or name.endswith(".rlib") or name.endswith(".rmeta"):
		return False
	# Exclude debug symbol bundles (.dSYM directories) and similar
	if name.endswith(".dSYM"):
		return False
	return os.access(path, os.X_OK)


def pick_latest(candidates: List[Path]) -> Optional[Path]:
	execs = [c for c in candidates if is_executable_candidate(c)]
	if not execs:
		return None
	execs.sort(key=lambda p: p.stat().st_mtime, reverse=True)
	return execs[0]


def main(argv: List[str]) -> int:
	parser = argparse.ArgumentParser(description="Locate compiled Criterion benchmark binary")
	parser.add_argument("--benchmark", required=True, help="Benchmark filter (e.g. group/function) passed after -- to cargo criterion")
	parser.add_argument("--package", "-p", default="battle_benchmarks", help="Cargo package name containing the bench target")
	parser.add_argument("--manifest-path", default="benchmarks/battle/Cargo.toml", help="Path to Cargo.toml for the benchmark package (relative to project root if not absolute)")
	parser.add_argument("--target-dir", default=None, help="Cargo target directory (defaults to metadata value if omitted; relative paths resolved to project root)")
	parser.add_argument("--samply", action="store_true", help="If set, invoke 'samply record <binary> <arg>' instead of printing the path")
	args = parser.parse_args(argv)

	# Determine project root relative to this script so invocation CWD doesn't matter.
	script_dir = Path(__file__).resolve().parent
	project_root = script_dir.parent.parent  # rules_engine root

	manifest_path = Path(args.manifest_path)
	if not manifest_path.is_absolute():
		manifest_path = (project_root / manifest_path).resolve()

	package = args.package
	benchmark_filter = args.benchmark

	try:
		metadata = cargo_metadata(str(manifest_path))
		# Determine target directory either from arg or metadata.
		if args.target_dir:
			target_dir_path = Path(args.target_dir)
			if not target_dir_path.is_absolute():
				target_dir_path = (project_root / target_dir_path).resolve()
		else:
			target_dir_path = Path(metadata.get("target_directory", project_root / "target")).resolve()
		bench_target = find_bench_target(metadata, package)
		build_benchmark(str(manifest_path), package, benchmark_filter)
		candidates = candidate_paths(target_dir_path, bench_target)
		picked = pick_latest(candidates)
		if not picked:
			raise SystemExit(f"Could not find built bench binary for target '{bench_target}' in {target_dir_path}/release/deps")
		if args.samply:
			binary_path = str(picked.resolve())
			try:
				completed = subprocess.run(["samply", "record", binary_path, benchmark_filter])
				return completed.returncode
			except FileNotFoundError:
				raise SystemExit("'samply' command not found in PATH. Install samply or omit --samply.")
		else:
			print(str(picked.resolve()))
			return 0
	except subprocess.CalledProcessError as e:
		sys.stderr.write(e.stdout)
		sys.stderr.write(e.stderr)
		return e.returncode


if __name__ == "__main__":
	raise SystemExit(main(sys.argv[1:]))
