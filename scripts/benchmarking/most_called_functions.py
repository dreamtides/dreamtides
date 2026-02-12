#!/usr/bin/env python3
"""Report the most frequently invoked user (workspace) Rust functions.

Workflow (requires building the binary with llvm source-based coverage):
  RUSTFLAGS="-Cinstrument-coverage -Ccodegen-units=1 -Clink-dead-code -Coverflow-checks=off" \
	LLVM_PROFILE_FILE="ignored" \
	cargo +nightly build --release -p <your_bench_pkg> --benches

Then run (existing binary path):
	python scripts/most_called_functions.py --binary path/to/bench_exe

Or let the script build the benchmark (from ANY working directory):
	python scripts/most_called_functions.py \
		--auto-build \
		--benchmark ai_core_11/ai_core_11 \
		-p battle_benchmarks \
		--manifest-path benchmarks/battle/Cargo.toml

This script will:
  * Execute the binary once with LLVM_PROFILE_FILE pattern to emit .profraw files
  * Merge them with llvm-profdata
  * Export JSON coverage via llvm-cov export
  * Aggregate per-function entry counts
  * Filter out std / dependency / outside-workspace code
  * Print the top N (default 10)

Notes:
  * Inlined or fully optimized-out functions will not appear
  * Generic monomorphizations appear separately unless --collapse-generics
  * Requires llvm-profdata and llvm-cov (Xcode CLT or Homebrew llvm)
  * For Homebrew llvm you may need: export PATH="/usr/local/opt/llvm/bin:$PATH" (or /opt/homebrew on ARM)
"""

from __future__ import annotations

import argparse
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path
from typing import Dict, List, Optional, Tuple


def which_llvm_tool(name: str) -> List[str]:
	path = shutil.which(name)
	if path:
		return [path]
	# Try xcrun (macOS developer tools)
	xcrun = shutil.which("xcrun")
	if xcrun:
		try:
			resolved = subprocess.run([xcrun, "-f", name], check=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True).stdout.strip()
			if resolved:
				return [xcrun, name]
		except subprocess.CalledProcessError:
			pass
	raise SystemExit(f"Required tool '{name}' not found in PATH (and xcrun fallback failed). Install Xcode CLT or 'brew install llvm'.")


def run_tool(cmd: List[str], capture: bool = True, check: bool = True) -> subprocess.CompletedProcess:
	try:
		return subprocess.run(
			cmd,
			stdout=subprocess.PIPE if capture else None,
			stderr=subprocess.PIPE if capture else None,
			text=True,
			check=check,
		)
	except subprocess.CalledProcessError as e:
		sys.stderr.write(f"[error] Command failed (exit {e.returncode}): {' '.join(cmd)}\n")
		if capture:
			if e.stdout:
				sys.stderr.write("--- tool stdout ---\n")
				sys.stderr.write(e.stdout)
				if not e.stdout.endswith("\n"):
					sys.stderr.write("\n")
			if e.stderr:
				sys.stderr.write("--- tool stderr ---\n")
				sys.stderr.write(e.stderr)
				if not e.stderr.endswith("\n"):
					sys.stderr.write("\n")
		raise


def demangle_if_requested(name: str, use_demangle: bool, demangler: Optional[List[str]]) -> str:
	if not use_demangle or not demangler:
		return name
	try:
		proc = subprocess.run(demangler, input=name, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL, text=True)
		if proc.returncode == 0 and proc.stdout.strip():
			return proc.stdout.strip()
	except Exception:
		return name
	return name


GENERIC_RE = re.compile(r"<[^<>]*>")
HASH_RE = re.compile(r"::h[0-9a-f]{16}$")


def collapse_generics(name: str) -> str:
	# Remove hash suffix and replace concrete generic args with <> to coalesce monomorphizations
	name = HASH_RE.sub("", name)
	# Repeatedly strip inner-most generics to a placeholder
	while True:
		new = GENERIC_RE.sub("<>", name)
		if new == name:
			break
		name = new
	# Collapse nested placeholders like <><> -> <>
	name = re.sub(r"(<>)+", "<>", name)
	return name


def parse_function_count(fn: Dict) -> int:
	# Prefer explicit count if present
	if "count" in fn and isinstance(fn["count"], (int, float)):
		return int(fn["count"])  # type: ignore[arg-type]
	# Fallback: first region with is_entry flag (region layout documented by llvm-cov)
	regions = fn.get("regions", [])
	# region entry format: [line_start, col_start, line_end, col_end, execution_count, is_gap_region, file_id, expanded_file_id, kind]
	# Some versions: [l1,c1,l2,c2,count, is_gap, file_id] ; we attempt best-effort by taking max count
	best = 0
	for r in regions:
		if isinstance(r, list) and len(r) >= 5:
			cnt = r[4]
			if isinstance(cnt, (int, float)) and cnt > best:
				best = int(cnt)
	return best


EXCLUDE_PATH_SNIPPETS = ["/rustc/", "/.cargo/registry/", "/.cargo/git/"]


def is_user_path(path: str, workspace_root: Path) -> bool:
	if any(s in path for s in EXCLUDE_PATH_SNIPPETS):
		return False
	try:
		p = Path(path).resolve()
		return workspace_root in p.parents or p == workspace_root
	except Exception:
		return False


def aggregate_functions(data: dict, workspace_root: Path, collapse: bool, demangle: bool, demangler: Optional[List[str]]) -> List[Tuple[str, int, str]]:
	results: Dict[Tuple[str, str], int] = {}
	functions = []
	# Coverage export: top-level key 'data' -> list -> each has 'functions'
	for entry in data.get("data", []):
		functions.extend(entry.get("functions", []))
	for fn in functions:
		filenames = fn.get("filenames", [])
		if not filenames:
			continue
		# Choose first file that is a user path
		file_for_fn = None
		for candidate in filenames:
			if is_user_path(candidate, workspace_root):
				file_for_fn = candidate
				break
		if not file_for_fn:
			continue
		raw_name = fn.get("name", "<unknown>")
		name = demangle_if_requested(raw_name, demangle, demangler)
		if collapse:
			name = collapse_generics(name)
		count = parse_function_count(fn)
		key = (file_for_fn, name)
		results[key] = results.get(key, 0) + count
	aggregated = [ (name, cnt, file) for (file, name), cnt in results.items() ]
	aggregated.sort(key=lambda t: t[1], reverse=True)
	return aggregated


def find_demangler() -> Optional[List[str]]:
	path = shutil.which("rustfilt")
	if path:
		return [path]
	return None


def run_binary(binary: Path, prof_pattern: str, passthrough: List[str], verbose: bool) -> int:
	env = os.environ.copy()
	env["LLVM_PROFILE_FILE"] = prof_pattern
	if verbose:
		print(f"[info] Running binary with LLVM_PROFILE_FILE={prof_pattern}")
	proc = subprocess.run([str(binary), *passthrough], env=env)
	return proc.returncode


def merge_profraw(profraw_files: List[Path], out_path: Path, verbose: bool) -> None:
	if verbose:
		print(f"[info] Merging {len(profraw_files)} profraw files -> {out_path}")
	tool = which_llvm_tool("llvm-profdata")
	cmd = [*tool, "merge", "-sparse", *[str(f) for f in profraw_files], "-o", str(out_path)]
	run_tool(cmd)


def export_json(binary: Path, profdata: Path, verbose: bool) -> dict:
	# Detect json support first to avoid noisy error output on older Apple llvm-cov.
	tool = which_llvm_tool("llvm-cov")
	help_out = run_tool([*tool, "export", "--help"], capture=True, check=False)
	help_text = help_out.stdout + help_out.stderr
	json_supported = "json" in help_text.lower()
	if json_supported:
		if verbose:
			print(f"[info] Exporting coverage (json)")
		cmd = [*tool, "export", "--format=json", f"--instr-profile={profdata}", str(binary)]
		proc = run_tool(cmd)
		try:
			return json.loads(proc.stdout)
		except json.JSONDecodeError as e:
			sys.stderr.write(proc.stdout[:5000])
			raise SystemExit(f"Failed to parse llvm-cov JSON: {e}")
	# Fallback: use lcov format and parse function counts.
	if verbose:
		print(f"[info] JSON format not supported; falling back to lcov parsing")
	cmd = [*tool, "export", "--format=lcov", f"--instr-profile={profdata}", str(binary)]
	proc = run_tool(cmd)
	lcov_text = proc.stdout
	data = parse_lcov_export(lcov_text)
	if verbose:
		print(f"[info] Parsed {sum(len(entry.get('functions', [])) for entry in data.get('data', []))} functions from lcov export")
	return data


def parse_lcov_export(text: str) -> dict:
	# lcov blocks are separated by end_of_record; each file has SF: lines and FN / FNDA pairs.
	# We'll accumulate functions into pseudo JSON structure similar to llvm-cov export json.
	functions = []
	current_file: Optional[str] = None
	fn_decl_lines: Dict[str, int] = {}
	# Example lines:
	# SF:/path/to/file.rs
	# FN:123,_ZN... (line,function)
	# FNDA:45,_ZN... (count,function)
	for raw in text.splitlines():
		line = raw.strip()
		if not line:
			continue
		if line.startswith("SF:"):
			current_file = line[3:]
			continue
		if line.startswith("FN:"):
			try:
				rest = line[3:]
				lineno_str, name = rest.split(",", 1)
				fn_decl_lines[name] = int(lineno_str)
			except ValueError:
				pass
			continue
		if line.startswith("FNDA:"):
			try:
				rest = line[5:]
				count_str, name = rest.split(",", 1)
				count = int(float(count_str))
				if current_file:
					functions.append({
						"name": name,
						"count": count,
						"filenames": [current_file],
						"regions": [],
					})
			except ValueError:
				pass
			continue
		if line == "end_of_record":
			current_file = None
			fn_decl_lines.clear()
			continue
	return {"data": [{"functions": functions}]}


# --------------------------- Auto-build support ---------------------------

def cargo_metadata(manifest_path: Path, verbose: bool) -> dict:
	cmd = ["cargo", "metadata", "--format-version", "1", "--no-deps", "--manifest-path", str(manifest_path)]
	if verbose:
		print(f"[info] Running: {' '.join(cmd)}")
	proc = run_tool(cmd)
	return json.loads(proc.stdout)


def find_bench_target(metadata: dict, package_name: str) -> str:
	for pkg in metadata.get("packages", []):
		if pkg.get("name") == package_name:
			benches = [t for t in pkg.get("targets", []) if "bench" in t.get("kind", [])]
			if not benches:
				raise SystemExit(f"No bench targets found in package '{package_name}'")
			if len(benches) == 1:
				return benches[0]["name"]
			for t in benches:
				if t.get("name") == package_name:
					return t["name"]
			return benches[0]["name"]
	raise SystemExit(f"Package '{package_name}' not found in metadata")


def build_benchmark(manifest_path: Path, package: str, benchmark_filter: Optional[str], toolchain: str, inject_instrument: bool, verbose: bool) -> None:
	cmd: List[str] = ["cargo"]
	if toolchain:
		cmd.append(toolchain)
	cmd += ["criterion", "--no-run", "--manifest-path", str(manifest_path), "-p", package]
	if benchmark_filter:
		cmd += ["--", benchmark_filter]
	env = os.environ.copy()
	if inject_instrument:
		inst_flags = "-Cinstrument-coverage -Ccodegen-units=1 -Clink-dead-code -Coverflow-checks=off"
		existing = env.get("RUSTFLAGS", "")
		if "-Cinstrument-coverage" not in existing:
			env["RUSTFLAGS"] = (inst_flags + (" " + existing if existing else "")).strip()
	if verbose:
		print(f"[info] Building benchmark: {' '.join(cmd)}")
		if inject_instrument:
			print(f"[info] RUSTFLAGS={env.get('RUSTFLAGS','')}")
	try:
		subprocess.run(cmd, env=env, check=True)
	except subprocess.CalledProcessError as e:
		raise SystemExit(f"Benchmark build failed (exit {e.returncode})")


def locate_bench_binary(target_dir: Path, bench_target: str, verbose: bool) -> Path:
	deps_dir = target_dir / "release" / "deps"
	if not deps_dir.is_dir():
		raise SystemExit(f"Expected deps dir not found: {deps_dir}")
	candidates = [p for p in deps_dir.glob(f"{bench_target}-*") if p.is_file() and os.access(p, os.X_OK)]
	candidates = [c for c in candidates if not any(c.name.endswith(ext) for ext in (".d", ".rlib", ".rmeta"))]
	if not candidates:
		raise SystemExit(f"No candidate binaries matching {bench_target}-* in {deps_dir}")
	candidates.sort(key=lambda p: p.stat().st_mtime, reverse=True)
	picked = candidates[0]
	if verbose:
		print(f"[info] Using bench binary: {picked}")
	return picked


def print_table(rows: List[Tuple[str, int, str]], limit: int, workspace_root: Path):
	print(f"Top {min(limit, len(rows))} functions by call count (workspace: {workspace_root})")
	print(f"{'Rank':>4}  {'Count':>12}  Function (File:Line?)")
	print("-" * 80)
	for idx, (name, count, file) in enumerate(rows[:limit], start=1):
		rel = os.path.relpath(file, workspace_root)
		print(f"{idx:>4}  {count:>12}  {name}  ({rel})")


def main(argv: List[str]) -> int:
	parser = argparse.ArgumentParser(description="Report most frequently invoked user Rust functions using LLVM coverage")
	parser.add_argument("--binary", required=False, help="Path to already-built benchmark (skip auto-build)")
	parser.add_argument("--limit", type=int, default=10, help="Number of top functions to display")
	parser.add_argument("--workspace-root", default=None, help="Workspace root (defaults to script parent directory)")
	parser.add_argument("--demangle", action="store_true", help="Demangle Rust symbol names using rustfilt if available")
	parser.add_argument("--collapse-generics", action="store_true", help="Collapse generic monomorphizations into a single synthetic entry")
	parser.add_argument("--keep-profraw", action="store_true", help="Do not delete temporary profraw files (prints their directory)")
	parser.add_argument("--merged-profdata", default=None, help="Write merged profdata to this path instead of temp file")
	parser.add_argument("--json", action="store_true", help="Output full JSON rows instead of human table")
	parser.add_argument("--verbose", action="store_true", help="Verbose progress messages")
	parser.add_argument("--no-run", action="store_true", help="Skip running the binary; assume profraws already collected in --profraw-dir")
	parser.add_argument("--profraw-dir", default=None, help="Directory containing existing *.profraw (used with --no-run)")
	parser.add_argument("--profile-pattern", default="prof-%p-%m.profraw", help="LLVM_PROFILE_FILE basename pattern (placed inside temp dir)")
	parser.add_argument("--pass-through", nargs=argparse.REMAINDER, help="Arguments after -- passed directly to the binary")
	# Auto-build options
	parser.add_argument("--auto-build", action="store_true", help="Build (criterion --no-run) the benchmark instead of providing --binary")
	parser.add_argument("--benchmark", default=None, help="Benchmark filter passed after -- to cargo criterion when auto-building (e.g. group/func)")
	parser.add_argument("--manifest-path", default="benchmarks/battle/Cargo.toml", help="Path to benchmark Cargo.toml (for auto-build)")
	parser.add_argument("--package", "-p", default="battle_benchmarks", help="Benchmark package name (for auto-build)")
	parser.add_argument("--toolchain", default="+nightly", help="Rust toolchain prefix to use (e.g. +nightly, empty for default)")
	parser.add_argument("--no-inject-instrument", action="store_true", help="Do not inject coverage RUSTFLAGS when auto-building")
	args = parser.parse_args(argv)

	script_dir = Path(__file__).resolve().parent
	workspace_root = Path(args.workspace_root).resolve() if args.workspace_root else script_dir.parent

	# Auto-build path resolution
	if args.auto_build:
		if args.binary:
			raise SystemExit("Provide either --binary or --auto-build, not both")
		manifest_path = Path(args.manifest_path)
		if not manifest_path.is_absolute():
			manifest_path = (workspace_root / manifest_path).resolve()
		metadata = cargo_metadata(manifest_path, args.verbose)
		bench_target = find_bench_target(metadata, args.package)
		target_dir = Path(metadata.get("target_directory", workspace_root / "target")).resolve()
		build_benchmark(manifest_path, args.package, args.benchmark, args.toolchain, not args.no_inject_instrument, args.verbose)
		binary = locate_bench_binary(target_dir, bench_target, args.verbose)
	else:
		if not args.binary:
			raise SystemExit("Must provide --binary or use --auto-build")
		binary = Path(args.binary).resolve()
		if not binary.is_file():
			raise SystemExit(f"Binary not found: {binary}")
		if not os.access(binary, os.X_OK):
			raise SystemExit(f"Binary is not executable: {binary}")

	demangler = find_demangler() if args.demangle else None
	if args.demangle and not demangler:
		raise SystemExit("--demangle requested, but 'rustfilt' was not found in PATH. Install with 'cargo install rustfilt' or via package manager and retry.")

	temp_dir_ctx = tempfile.TemporaryDirectory() if not args.keep_profraw else None
	try:
		if args.profraw_dir:
			profraw_dir = Path(args.profraw_dir).resolve()
		else:
			profraw_dir = Path(temp_dir_ctx.name) if temp_dir_ctx else Path(tempfile.mkdtemp(prefix="most_called_profraw_"))
		if not profraw_dir.exists():
			profraw_dir.mkdir(parents=True)

		if not args.no_run:
			pattern = str(profraw_dir / args.profile_pattern)
			rc = run_binary(binary, pattern, args.pass_through or [], args.verbose)
			if rc != 0:
				print(f"[warn] Binary exited with code {rc}; proceeding to process coverage anyway")
		else:
			if args.verbose:
				print("[info] --no-run specified; using existing profraw files")

		profraw_files = list(profraw_dir.glob("*.profraw"))
		if not profraw_files:
			hint = (
				"No .profraw files found. Ensure the binary was built with: "
				"RUSTFLAGS=\"-Cinstrument-coverage -Ccodegen-units=1 -Clink-dead-code -Coverflow-checks=off\" and not stripped."
			)
			raise SystemExit(hint)

		if args.verbose:
			print(f"[info] Found {len(profraw_files)} profraw files")

		merged_profdata = Path(args.merged_profdata).resolve() if args.merged_profdata else (profraw_dir / "merged.profdata")
		merge_profraw(profraw_files, merged_profdata, args.verbose)
		export = export_json(binary, merged_profdata, args.verbose)
		rows = aggregate_functions(export, workspace_root, args.collapse_generics, args.demangle, demangler)
		if args.json:
			# Emit JSON array of objects
			out = [ {"function": n, "count": c, "file": str(f)} for (n,c,f) in rows[:args.limit] ]
			json.dump(out, sys.stdout, indent=2)
			print()
		else:
			print_table(rows, args.limit, workspace_root)
		if args.keep_profraw:
			print(f"[info] profraw directory retained: {profraw_dir}")
		return 0
	finally:
		if temp_dir_ctx is not None and not args.keep_profraw:
			temp_dir_ctx.cleanup()


if __name__ == "__main__":
	raise SystemExit(main(sys.argv[1:]))

