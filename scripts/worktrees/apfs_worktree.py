#!/usr/bin/env python3
"""APFS-backed ephemeral git worktrees with copy-on-write cloning.

Creates git worktrees that include APFS COW clones of all gitignored
directories (build caches, Unity Library, etc.), enabling near-instant
setup with warm caches at negligible disk cost.
"""

from __future__ import annotations

import argparse
import fnmatch
import os
import shutil
import subprocess
import sys
from pathlib import Path

REPO_ROOT: Path = Path(__file__).resolve().parent.parent.parent
DEFAULT_WORKTREE_BASE: Path = Path.home() / "dreamtides-worktrees"

EXCLUDE: set[str] = {
    ".DS_Store",
    "__pycache__",
    ".pytest_cache",
    ".worktrees",
    ".venv",
    ".logs",
    ".lattice",
    ".serena",
    ".abu-state.json",
    ".validation_marker",
    "dreamtides.log",
    "dreamtides.json",
    "log.json",
    "profile.json",
    "client/Temp",
    "client/Logs",
    "client/test_output",
    "client/.plastic",
    "tmp",
    "*.mm_profdata",
    "*.csproj",
    "*.sln",
    "*.slnx",
    "*.private.0",
}

MIN_FREE_GB: int = 2
WARN_FREE_GB: int = 5


def run_cmd(
    args: list[str],
    capture: bool = False,
    check: bool = True,
    cwd: Path | None = None,
) -> subprocess.CompletedProcess[str]:
    """Run a subprocess command."""
    return subprocess.run(
        args,
        capture_output=capture,
        text=True,
        check=check,
        cwd=cwd,
    )


def get_free_gb(path: Path) -> float:
    """Return free disk space in GB for the volume containing path."""
    stat = os.statvfs(path)
    return (stat.f_bavail * stat.f_frsize) / (1024**3)


def verify_apfs(path: Path) -> bool:
    """Verify the volume containing path is APFS."""
    result = run_cmd(
        ["diskutil", "info", str(path)],
        capture=True,
        check=False,
    )
    if result.returncode != 0:
        result = run_cmd(
            ["diskutil", "info", "/"],
            capture=True,
            check=False,
        )
    return "APFS" in result.stdout


def should_exclude(item_path: str) -> bool:
    """Check if an item should be excluded from cloning."""
    clean: str = item_path.rstrip("/")
    parts: list[str] = clean.split("/")
    for pattern in EXCLUDE:
        if "/" in pattern:
            if clean == pattern or clean.startswith(pattern + "/"):
                return True
        else:
            for part in parts:
                if fnmatch.fnmatch(part, pattern):
                    return True
    return False


def discover_untracked_items(repo: Path) -> list[str]:
    """Discover all untracked/gitignored items in the repo."""
    result = run_cmd(
        [
            "git",
            "ls-files",
            "--others",
            "--ignored",
            "--directory",
            "--exclude-standard",
        ],
        capture=True,
        cwd=repo,
    )
    ignored: set[str] = set(result.stdout.strip().splitlines()) if result.stdout.strip() else set()

    result = run_cmd(
        [
            "git",
            "ls-files",
            "--others",
            "--directory",
            "--exclude-standard",
        ],
        capture=True,
        cwd=repo,
    )
    untracked: set[str] = set(result.stdout.strip().splitlines()) if result.stdout.strip() else set()

    all_items: list[str] = sorted(ignored | untracked)
    return [item.rstrip("/") for item in all_items if item.strip()]


def clone_item(source: Path, dest: Path, dry_run: bool) -> bool:
    """APFS-clone a single item from source to dest. Returns True on success."""
    if not source.exists() and not source.is_symlink():
        return False

    if not dry_run:
        dest.parent.mkdir(parents=True, exist_ok=True)

    if source.is_symlink():
        target: Path = Path(os.readlink(source))
        if dry_run:
            print(f"  [symlink] {dest} -> {target}")
        else:
            if dest.exists() or dest.is_symlink():
                dest.unlink()
            dest.symlink_to(target)
        return True

    resolved: Path = source.resolve()
    if not resolved.exists():
        return False

    if resolved.is_dir():
        if dry_run:
            print(f"  [dir-clone] {dest}")
        else:
            result = run_cmd(
                ["cp", "-cR", str(resolved), str(dest)],
                check=False,
            )
            if result.returncode != 0:
                print(f"  Warning: APFS clone failed for {source}, skipping")
                return False
    else:
        if dry_run:
            print(f"  [file-clone] {dest}")
        else:
            result = run_cmd(
                ["cp", "-c", str(resolved), str(dest)],
                check=False,
            )
            if result.returncode != 0:
                print(f"  Warning: APFS clone failed for {source}, skipping")
                return False

    return True


def find_main_repo() -> Path:
    """Find the main (non-worktree) repo root via git worktree list."""
    result = run_cmd(
        ["git", "worktree", "list", "--porcelain"],
        capture=True,
    )
    for line in result.stdout.splitlines():
        if line.startswith("worktree "):
            return Path(line.split(" ", 1)[1])
    print("Error: Could not find main worktree")
    sys.exit(1)


def get_dir_size_bytes(path: Path) -> int:
    """Return total size in bytes of a directory tree."""
    result = run_cmd(
        ["du", "-sk", str(path)],
        capture=True,
        check=False,
    )
    if result.returncode != 0:
        return 0
    try:
        return int(result.stdout.split()[0]) * 1024
    except (ValueError, IndexError):
        return 0


def cleanup_worktree(worktree_path: Path) -> None:
    """Clean up a partially created worktree."""
    run_cmd(
        ["git", "worktree", "remove", "--force", str(worktree_path)],
        check=False,
        cwd=REPO_ROOT,
    )
    if worktree_path.exists():
        shutil.rmtree(worktree_path, ignore_errors=True)


def cmd_create(args: argparse.Namespace) -> None:
    """Create a new worktree with APFS-cloned caches."""
    branch: str = args.branch
    existing: bool = args.existing
    base: str = args.base
    dry_run: bool = args.dry_run

    if args.dest:
        worktree_path: Path = Path(args.dest).expanduser().resolve()
    else:
        worktree_path = DEFAULT_WORKTREE_BASE / branch

    if not verify_apfs(worktree_path.parent if worktree_path.parent.exists() else Path.home()):
        print("Error: Filesystem is not APFS. APFS clones require an APFS volume.")
        sys.exit(1)

    free_gb: float = get_free_gb(
        worktree_path.parent if worktree_path.parent.exists() else Path.home()
    )
    if free_gb < MIN_FREE_GB:
        print(f"Error: Only {free_gb:.1f}GB free. Need at least {MIN_FREE_GB}GB.")
        sys.exit(1)
    if free_gb < WARN_FREE_GB:
        print(f"Warning: Only {free_gb:.1f}GB free. Proceeding with caution.")

    if worktree_path.exists():
        print(f"Error: Destination already exists: {worktree_path}")
        sys.exit(1)

    print(f"Creating worktree at {worktree_path}")
    if dry_run:
        print("[dry-run] Would create git worktree")
    else:
        worktree_path.parent.mkdir(parents=True, exist_ok=True)
        git_args: list[str] = ["git", "worktree", "add"]
        if existing:
            git_args.extend([str(worktree_path), branch])
        else:
            git_args.extend(["-b", branch, str(worktree_path), base])
        result = run_cmd(git_args, check=False, cwd=REPO_ROOT)
        if result.returncode != 0:
            print("Error: git worktree add failed")
            sys.exit(1)

    print("Discovering untracked/gitignored items...")
    items: list[str] = discover_untracked_items(REPO_ROOT)

    clone_count: int = 0
    skip_count: int = 0

    for item in items:
        if should_exclude(item):
            skip_count += 1
            continue

        source: Path = REPO_ROOT / item
        dest: Path = worktree_path / item

        if dest.exists() or dest.is_symlink():
            continue

        success: bool = clone_item(source, dest, dry_run)
        if success:
            clone_count += 1

    print(f"\nDone! Worktree ready at: {worktree_path}")
    print(f"  Branch: {branch}")
    print(f"  Cloned: {clone_count} items")
    print(f"  Excluded: {skip_count} items")
    if not dry_run:
        wt_free: float = get_free_gb(worktree_path)
        print(f"  Free disk: {wt_free:.1f}GB")


def cmd_remove(args: argparse.Namespace) -> None:
    """Remove a worktree."""
    target: str = args.target
    delete_branch: bool = args.delete_branch

    target_path: Path = Path(target).expanduser().resolve()
    if not target_path.exists():
        candidate: Path = DEFAULT_WORKTREE_BASE / target
        if candidate.exists():
            target_path = candidate
        else:
            print(f"Error: Worktree not found: {target} (also checked {candidate})")
            sys.exit(1)

    branch_name: str | None = None
    if delete_branch:
        result = run_cmd(
            ["git", "worktree", "list", "--porcelain"],
            capture=True,
            cwd=REPO_ROOT,
        )
        lines: list[str] = result.stdout.splitlines()
        for i, line in enumerate(lines):
            if line.startswith("worktree ") and Path(line.split(" ", 1)[1]).resolve() == target_path:
                for j in range(i + 1, min(i + 5, len(lines))):
                    if lines[j].startswith("branch refs/heads/"):
                        branch_name = lines[j].removeprefix("branch refs/heads/")
                        break
                break

    print(f"Removing worktree: {target_path}")
    result = run_cmd(
        ["git", "worktree", "remove", "--force", str(target_path)],
        check=False,
        cwd=REPO_ROOT,
    )
    if result.returncode != 0:
        print("git worktree remove failed, falling back to rm -rf")
        shutil.rmtree(target_path, ignore_errors=True)

    if target_path.exists():
        shutil.rmtree(target_path, ignore_errors=True)

    if delete_branch and branch_name:
        print(f"Deleting branch: {branch_name}")
        run_cmd(
            ["git", "branch", "-D", branch_name],
            check=False,
            cwd=REPO_ROOT,
        )

    print("Done!")


def resolve_worktree_path(target: str | None) -> Path:
    """Resolve a worktree target (branch name, path, or None for cwd) to an absolute path."""
    if target is None:
        return Path.cwd().resolve()

    target_path: Path = Path(target).expanduser().resolve()
    if target_path.exists():
        return target_path

    candidate: Path = DEFAULT_WORKTREE_BASE / target
    if candidate.exists():
        return candidate

    print(f"Error: Worktree not found: {target} (also checked {candidate})")
    sys.exit(1)


def list_worktree_paths() -> list[Path]:
    """Return paths of all worktrees under the default base directory."""
    result = run_cmd(
        ["git", "worktree", "list", "--porcelain"],
        capture=True,
    )
    main_repo: Path = find_main_repo().resolve()
    wt_base: Path = DEFAULT_WORKTREE_BASE.resolve()
    paths: list[Path] = []
    for line in result.stdout.splitlines():
        if line.startswith("worktree "):
            wt_path: Path = Path(line.split(" ", 1)[1]).resolve()
            if wt_path != main_repo and str(wt_path).startswith(str(wt_base)):
                paths.append(wt_path)
    return paths


def refresh_one_worktree(
    worktree_path: Path,
    main_repo: Path,
    items: list[str],
    dry_run: bool,
) -> None:
    """Refresh a single worktree by re-cloning gitignored directories."""
    print(f"Refreshing worktree: {worktree_path}")

    refreshed_count: int = 0
    skip_count: int = 0
    total_old_bytes: int = 0

    for item in items:
        if should_exclude(item):
            skip_count += 1
            continue

        source: Path = main_repo / item
        dest: Path = worktree_path / item

        if not (dest.exists() or dest.is_symlink()):
            continue

        if not (source.exists() or source.is_symlink()):
            continue

        if source.is_symlink():
            new_target: Path = Path(os.readlink(source))
            if dest.is_symlink() and Path(os.readlink(dest)) == new_target:
                continue
            if dry_run:
                print(f"  [symlink] {item} -> {new_target}")
            else:
                if dest.exists() or dest.is_symlink():
                    dest.unlink()
                dest.symlink_to(new_target)
            refreshed_count += 1
            continue

        if not source.resolve().is_dir():
            continue

        old_size: int = get_dir_size_bytes(dest)
        total_old_bytes += old_size
        size_mb: float = old_size / (1024 * 1024)

        if dry_run:
            print(f"  [refresh] {item} ({size_mb:.0f}MB)")
        else:
            print(f"  Refreshing {item} ({size_mb:.0f}MB)...")
            shutil.rmtree(dest, ignore_errors=True)
            clone_item(source, dest, dry_run=False)
        refreshed_count += 1

    total_old_mb: float = total_old_bytes / (1024 * 1024)
    print(f"  Refreshed {refreshed_count} items ({total_old_mb:.0f}MB replaced with fresh clones)")
    print(f"  Skipped: {skip_count} excluded items")


def cmd_refresh(args: argparse.Namespace) -> None:
    """Re-clone gitignored directories from the main repo to reduce COW divergence."""
    dry_run: bool = args.dry_run
    build: bool = args.build
    refresh_all: bool = args.all

    main_repo: Path = find_main_repo().resolve()

    if refresh_all:
        worktree_paths: list[Path] = list_worktree_paths()
        if not worktree_paths:
            print(f"No worktrees found under {DEFAULT_WORKTREE_BASE}")
            return
    else:
        wt_path: Path = resolve_worktree_path(args.target)
        if wt_path == main_repo:
            print("Error: Cannot refresh the main repo itself.")
            sys.exit(1)
        result = run_cmd(
            ["git", "worktree", "list", "--porcelain"],
            capture=True,
        )
        is_worktree: bool = False
        for line in result.stdout.splitlines():
            if line.startswith("worktree ") and Path(line.split(" ", 1)[1]).resolve() == wt_path:
                is_worktree = True
                break
        if not is_worktree:
            print(f"Error: {wt_path} is not a known git worktree")
            sys.exit(1)
        worktree_paths = [wt_path]

    print(f"Clone source: {main_repo}")
    if len(worktree_paths) > 1:
        print(f"Refreshing {len(worktree_paths)} worktrees\n")

    if build:
        print("Building on master to warm the cache...")
        if dry_run:
            print("[dry-run] Would run: cargo check in main repo\n")
        else:
            build_result = run_cmd(
                [
                    "cargo",
                    "check",
                    "--manifest-path",
                    str(main_repo / "rules_engine" / "Cargo.toml"),
                    "--workspace",
                    "--all-targets",
                    "--all-features",
                ],
                check=False,
            )
            if build_result.returncode != 0:
                print("Warning: cargo check failed, continuing with refresh anyway\n")
            else:
                print("Build complete.\n")

    items: list[str] = discover_untracked_items(main_repo)

    for i, worktree_path in enumerate(worktree_paths):
        if i > 0:
            print()
        refresh_one_worktree(worktree_path, main_repo, items, dry_run)

    if not dry_run:
        print(f"\nFree disk: {get_free_gb(main_repo):.1f}GB")


def cmd_list(args: argparse.Namespace) -> None:
    """List worktrees under the default base directory."""
    result = run_cmd(
        ["git", "worktree", "list"],
        capture=True,
        cwd=REPO_ROOT,
    )
    wt_base: str = str(DEFAULT_WORKTREE_BASE)
    lines: list[str] = result.stdout.strip().splitlines()
    found: int = 0
    for line in lines:
        if wt_base in line:
            print(line)
            found += 1
    if found == 0:
        print(f"No worktrees found under {DEFAULT_WORKTREE_BASE}")


def main() -> None:
    parser: argparse.ArgumentParser = argparse.ArgumentParser(
        description="APFS-backed ephemeral git worktrees",
    )
    subparsers = parser.add_subparsers(dest="command", required=True)

    create_parser = subparsers.add_parser("create", help="Create a new worktree")
    create_parser.add_argument("branch", help="Branch name")
    create_parser.add_argument(
        "--existing",
        action="store_true",
        help="Check out an existing branch instead of creating a new one",
    )
    create_parser.add_argument(
        "--base",
        default="master",
        help="Base branch for new branches (default: master)",
    )
    create_parser.add_argument(
        "--dest",
        help="Custom worktree destination path",
    )
    create_parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Show what would be done without doing it",
    )

    remove_parser = subparsers.add_parser("remove", help="Remove a worktree")
    remove_parser.add_argument(
        "target",
        help="Branch name or path to worktree",
    )
    remove_parser.add_argument(
        "--delete-branch",
        action="store_true",
        help="Also delete the git branch",
    )

    refresh_parser = subparsers.add_parser(
        "refresh",
        help="Re-clone gitignored dirs from main repo to reduce disk usage",
    )
    refresh_parser.add_argument(
        "target",
        nargs="?",
        default=None,
        help="Branch name or path to worktree (default: current directory)",
    )
    refresh_parser.add_argument(
        "--all",
        action="store_true",
        help="Refresh all worktrees under ~/dreamtides-worktrees/",
    )
    refresh_parser.add_argument(
        "--build",
        action="store_true",
        help="Run cargo check on main repo first to warm the cache",
    )
    refresh_parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Show what would be done without doing it",
    )

    subparsers.add_parser("list", help="List worktrees")

    parsed_args: argparse.Namespace = parser.parse_args()

    if parsed_args.command == "create":
        cmd_create(parsed_args)
    elif parsed_args.command == "remove":
        cmd_remove(parsed_args)
    elif parsed_args.command == "refresh":
        cmd_refresh(parsed_args)
    elif parsed_args.command == "list":
        cmd_list(parsed_args)


if __name__ == "__main__":
    main()
