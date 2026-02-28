#!/usr/bin/env python3
"""APFS-backed ephemeral git worktrees with copy-on-write cloning.

Creates git worktrees that include APFS COW clones of all gitignored
directories (build caches, Unity Library, etc.), enabling near-instant
setup with warm caches at negligible disk cost.
"""

from __future__ import annotations

import argparse
import fnmatch
import json
import os
import shutil
import subprocess
import sys
from pathlib import Path

REPO_ROOT: Path = Path(__file__).resolve().parent.parent.parent
DEFAULT_WORKTREE_BASE: Path = Path.home() / "dreamtides-worktrees"
PORTS_FILE: Path = DEFAULT_WORKTREE_BASE / ".ports.json"
FIRST_WORKTREE_PORT: int = 10000

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
POOL_SLOTS: tuple[str, ...] = ("alpha", "beta", "gamma")


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


def read_ports() -> dict[str, int]:
    """Read the port registry from .ports.json."""
    try:
        return json.loads(PORTS_FILE.read_text())
    except (OSError, json.JSONDecodeError):
        return {}


def write_ports(ports: dict[str, int]) -> None:
    """Write the port registry to .ports.json."""
    PORTS_FILE.parent.mkdir(parents=True, exist_ok=True)
    PORTS_FILE.write_text(json.dumps(ports, indent=2) + "\n")


def allocate_port(name: str) -> int:
    """Allocate a port for a worktree and write it to .ports.json."""
    ports = read_ports()
    if name in ports:
        return ports[name]
    used: set[int] = set(ports.values())
    port: int = FIRST_WORKTREE_PORT
    while port in used:
        port += 1
    ports[name] = port
    write_ports(ports)
    return port


def deallocate_port(name: str) -> None:
    """Remove a worktree's port from .ports.json."""
    ports = read_ports()
    if name in ports:
        del ports[name]
        write_ports(ports)


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
    ignored: set[str] = (
        set(result.stdout.strip().splitlines()) if result.stdout.strip() else set()
    )

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
    untracked: set[str] = (
        set(result.stdout.strip().splitlines()) if result.stdout.strip() else set()
    )

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

    if not verify_apfs(
        worktree_path.parent if worktree_path.parent.exists() else Path.home()
    ):
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

    # Allocate a port for this worktree
    if not dry_run:
        wt_name: str = worktree_path.name
        port: int = allocate_port(wt_name)
        print(f"  Allocated port {port} for worktree '{wt_name}'")

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
            if (
                line.startswith("worktree ")
                and Path(line.split(" ", 1)[1]).resolve() == target_path
            ):
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

    # Deallocate port for this worktree
    deallocate_port(target_path.name)

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


def sync_tree(
    source_root: Path, dest_root: Path, dry_run: bool
) -> tuple[int, int, int]:
    """Incrementally sync a directory tree using APFS clones.

    Walks source and dest in parallel. Only replaces files whose
    mtime or size differs. Returns (cloned, deleted, unchanged) counts.
    """
    cloned: int = 0
    deleted: int = 0
    unchanged: int = 0

    resolved_source: Path = source_root.resolve()
    if not resolved_source.is_dir():
        return cloned, deleted, unchanged

    # Collect dest entries for deletion detection
    dest_entries: set[str] = set()
    if dest_root.is_dir():
        for dirpath_str, dirnames, filenames in os.walk(dest_root):
            dirpath: Path = Path(dirpath_str)
            for name in filenames:
                dest_entries.add(str((dirpath / name).relative_to(dest_root)))
            for name in dirnames:
                child: Path = dirpath / name
                if child.is_symlink():
                    dest_entries.add(str(child.relative_to(dest_root)))

    source_entries: set[str] = set()
    for dirpath_str, dirnames, filenames in os.walk(resolved_source):
        dirpath = Path(dirpath_str)
        rel_dir: Path = dirpath.relative_to(resolved_source)

        dest_dir: Path = dest_root / rel_dir
        if not dry_run:
            dest_dir.mkdir(parents=True, exist_ok=True)

        # Handle symlinked subdirectories (os.walk follows them, but we
        # want to detect symlinks in the *original* source, not resolved)
        for name in dirnames:
            src_child: Path = source_root / rel_dir / name
            if src_child.is_symlink():
                rel: str = str(rel_dir / name)
                source_entries.add(rel)
                dest_child: Path = dest_root / rel_dir / name
                link_target: Path = Path(os.readlink(src_child))
                if (
                    dest_child.is_symlink()
                    and Path(os.readlink(dest_child)) == link_target
                ):
                    unchanged += 1
                else:
                    if not dry_run:
                        if dest_child.exists() or dest_child.is_symlink():
                            dest_child.unlink()
                        dest_child.symlink_to(link_target)
                    cloned += 1

        for name in filenames:
            src_file: Path = dirpath / name
            rel = str(rel_dir / name)
            source_entries.add(rel)
            dest_file: Path = dest_root / rel_dir / name

            if dest_file.exists():
                try:
                    src_stat: os.stat_result = src_file.stat()
                    dst_stat: os.stat_result = dest_file.stat()
                    if (
                        src_stat.st_size == dst_stat.st_size
                        and abs(src_stat.st_mtime - dst_stat.st_mtime) < 0.01
                    ):
                        unchanged += 1
                        continue
                except OSError:
                    pass

            if not dry_run:
                if dest_file.exists() or dest_file.is_symlink():
                    dest_file.unlink()
                run_cmd(["cp", "-c", str(src_file), str(dest_file)], check=False)
            cloned += 1

    # Delete files in dest that don't exist in source
    for rel in dest_entries - source_entries:
        dest_file = dest_root / rel
        if dest_file.exists() or dest_file.is_symlink():
            if not dry_run:
                if dest_file.is_dir() and not dest_file.is_symlink():
                    shutil.rmtree(dest_file, ignore_errors=True)
                else:
                    dest_file.unlink()
            deleted += 1

    # Delete empty directories in dest that don't exist in source
    if not dry_run and dest_root.is_dir():
        for dirpath_str, dirnames, filenames in os.walk(dest_root, topdown=False):
            dirpath = Path(dirpath_str)
            if dirpath != dest_root and not any(dirpath.iterdir()):
                dirpath.rmdir()

    return cloned, deleted, unchanged


def refresh_one_worktree(
    worktree_path: Path,
    main_repo: Path,
    items: list[str],
    dry_run: bool,
) -> None:
    """Refresh a single worktree by incrementally syncing gitignored directories."""
    print(f"Refreshing worktree: {worktree_path}")

    total_cloned: int = 0
    total_deleted: int = 0
    total_unchanged: int = 0
    dir_count: int = 0

    for item in items:
        if should_exclude(item):
            continue

        source: Path = main_repo / item
        dest: Path = worktree_path / item

        if not (source.exists() or source.is_symlink()):
            continue

        if source.is_symlink() and not source.resolve().is_dir():
            new_target: Path = Path(os.readlink(source))
            if dest.is_symlink() and Path(os.readlink(dest)) == new_target:
                total_unchanged += 1
            elif not dry_run:
                dest.parent.mkdir(parents=True, exist_ok=True)
                if dest.exists() or dest.is_symlink():
                    dest.unlink()
                dest.symlink_to(new_target)
                total_cloned += 1
            else:
                total_cloned += 1
            continue

        resolved: Path = source.resolve()
        if not resolved.is_dir():
            continue

        if not (dest.exists() or dest.is_symlink()):
            continue

        cloned, deleted, unchanged = sync_tree(source, dest, dry_run)
        total_cloned += cloned
        total_deleted += deleted
        total_unchanged += unchanged
        if cloned > 0 or deleted > 0:
            print(
                f"  {item}: {cloned} cloned, {deleted} deleted, {unchanged} unchanged"
            )
        dir_count += 1

    print(
        f"  Synced {dir_count} directories: {total_cloned} cloned, {total_deleted} deleted, {total_unchanged} unchanged"
    )


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
            if (
                line.startswith("worktree ")
                and Path(line.split(" ", 1)[1]).resolve() == wt_path
            ):
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


def cmd_activate(args: argparse.Namespace) -> None:
    """Activate an existing worktree, resetting it to match a fresh creation from the base branch.

    Much faster than remove + create because it:
    - Uses git reset --hard instead of full worktree remove/add
    - Incrementally syncs gitignored dirs (mtime/size comparison) instead of full APFS clones
    """
    branch: str = args.branch
    base: str = args.base
    dry_run: bool = args.dry_run

    worktree_path: Path = DEFAULT_WORKTREE_BASE / branch

    if not worktree_path.exists():
        print(f"Error: Worktree '{branch}' does not exist at {worktree_path}")
        print("Use 'abu worktree create' to create a new worktree.")
        sys.exit(1)

    # Check main repo is clean
    result = run_cmd(
        ["git", "status", "--porcelain"],
        capture=True,
        cwd=REPO_ROOT,
    )
    if result.stdout.strip():
        print("Error: Main repo has uncommitted changes. Commit or stash first.")
        sys.exit(1)

    main_repo: Path = find_main_repo().resolve()

    print(f"Activating worktree: {worktree_path}")

    # Step 1: Reset tracked files to match base branch
    print(f"Resetting to {base}...")
    if dry_run:
        print(f"  [dry-run] Would run: git reset --hard {base}")
        print("  [dry-run] Would run: git clean -fd")
    else:
        result = run_cmd(
            ["git", "reset", "--hard", base],
            check=False,
            cwd=worktree_path,
        )
        if result.returncode != 0:
            print(f"Error: git reset --hard {base} failed")
            sys.exit(1)
        run_cmd(["git", "clean", "-fd"], cwd=worktree_path)

    # Step 2: Sync gitignored items from main repo
    print("Syncing gitignored items...")
    main_items: list[str] = discover_untracked_items(main_repo)
    main_items_set: set[str] = set(main_items)

    total_cloned: int = 0
    total_deleted: int = 0
    total_unchanged: int = 0
    dir_count: int = 0

    for item in main_items:
        if should_exclude(item):
            continue

        source: Path = main_repo / item
        dest: Path = worktree_path / item

        if not (source.exists() or source.is_symlink()):
            continue

        # Handle symlinks
        if source.is_symlink():
            target: Path = Path(os.readlink(source))
            if dest.is_symlink() and Path(os.readlink(dest)) == target:
                total_unchanged += 1
            else:
                if not dry_run:
                    dest.parent.mkdir(parents=True, exist_ok=True)
                    if dest.exists() or dest.is_symlink():
                        if dest.is_dir() and not dest.is_symlink():
                            shutil.rmtree(dest)
                        else:
                            dest.unlink()
                    dest.symlink_to(target)
                total_cloned += 1
            continue

        resolved: Path = source.resolve()
        if not resolved.exists():
            continue

        if resolved.is_dir():
            if dest.exists() or dest.is_symlink():
                # Exists in both: incremental sync
                cloned, deleted_count, unchanged = sync_tree(source, dest, dry_run)
                total_cloned += cloned
                total_deleted += deleted_count
                total_unchanged += unchanged
                if cloned > 0 or deleted_count > 0:
                    print(
                        f"  {item}: {cloned} cloned, {deleted_count} deleted, {unchanged} unchanged"
                    )
            else:
                # Exists in main only: fresh APFS clone
                if clone_item(source, dest, dry_run):
                    total_cloned += 1
                    print(f"  {item}: cloned (new)")
            dir_count += 1
        else:
            # Single file: clone if changed or missing
            if dest.exists():
                try:
                    src_stat: os.stat_result = source.stat()
                    dst_stat: os.stat_result = dest.stat()
                    if (
                        src_stat.st_size == dst_stat.st_size
                        and abs(src_stat.st_mtime - dst_stat.st_mtime) < 0.01
                    ):
                        total_unchanged += 1
                        continue
                except OSError:
                    pass
            if clone_item(source, dest, dry_run):
                total_cloned += 1

    # Step 3: Remove gitignored items in worktree that don't exist in main repo
    worktree_items: list[str] = discover_untracked_items(worktree_path)
    removed_count: int = 0
    for item in worktree_items:
        if should_exclude(item):
            continue
        if item not in main_items_set:
            dest = worktree_path / item
            if dest.exists() or dest.is_symlink():
                if not dry_run:
                    if dest.is_dir() and not dest.is_symlink():
                        shutil.rmtree(dest, ignore_errors=True)
                    else:
                        dest.unlink()
                print(f"  Removed extra: {item}")
                removed_count += 1

    # Ensure port is allocated
    if not dry_run:
        port: int = allocate_port(branch)
        print(f"  Port: {port}")

    total_removed: int = total_deleted + removed_count
    print(f"\nDone! Worktree activated at: {worktree_path}")
    print(f"  Branch: {branch} (reset to {base})")
    print(
        f"  Synced {dir_count} directories: {total_cloned} cloned, {total_removed} deleted, {total_unchanged} unchanged"
    )
    if not dry_run:
        print(f"  Free disk: {get_free_gb(worktree_path):.1f}GB")


def _eprint(*args: object) -> None:
    """Print to stderr."""
    print(*args, file=sys.stderr)


def _worktree_branch(worktree_path: Path) -> str | None:
    """Get the branch checked out in a worktree, or None if detached/unknown."""
    result = run_cmd(
        ["git", "worktree", "list", "--porcelain"],
        capture=True,
        cwd=REPO_ROOT,
    )
    resolved: Path = worktree_path.resolve()
    lines: list[str] = result.stdout.splitlines()
    for i, line in enumerate(lines):
        if (
            line.startswith("worktree ")
            and Path(line.split(" ", 1)[1]).resolve() == resolved
        ):
            for j in range(i + 1, min(i + 5, len(lines))):
                if lines[j].startswith("branch refs/heads/"):
                    return lines[j].removeprefix("branch refs/heads/")
            return None
    return None


def _is_worktree_available(worktree_path: Path, base: str) -> bool:
    """Check if a worktree is available (merged into base and clean)."""
    result = run_cmd(
        ["git", "merge-base", "--is-ancestor", "HEAD", base],
        capture=True,
        check=False,
        cwd=worktree_path,
    )
    if result.returncode != 0:
        return False

    result = run_cmd(
        ["git", "status", "--porcelain"],
        capture=True,
        cwd=worktree_path,
    )
    for line in result.stdout.splitlines():
        if not line.startswith("??"):
            return False

    return True


def _sync_gitignored(worktree_path: Path, main_repo: Path) -> None:
    """Sync gitignored items from main repo to worktree, printing diagnostics to stderr."""
    _eprint("Syncing gitignored items...")
    main_items: list[str] = discover_untracked_items(main_repo)
    main_items_set: set[str] = set(main_items)

    total_cloned: int = 0
    total_deleted: int = 0
    total_unchanged: int = 0
    dir_count: int = 0

    for item in main_items:
        if should_exclude(item):
            continue

        source: Path = main_repo / item
        dest: Path = worktree_path / item

        if not (source.exists() or source.is_symlink()):
            continue

        if source.is_symlink():
            target: Path = Path(os.readlink(source))
            if dest.is_symlink() and Path(os.readlink(dest)) == target:
                total_unchanged += 1
            else:
                dest.parent.mkdir(parents=True, exist_ok=True)
                if dest.exists() or dest.is_symlink():
                    if dest.is_dir() and not dest.is_symlink():
                        shutil.rmtree(dest)
                    else:
                        dest.unlink()
                dest.symlink_to(target)
                total_cloned += 1
            continue

        resolved: Path = source.resolve()
        if not resolved.exists():
            continue

        if resolved.is_dir():
            if dest.exists() or dest.is_symlink():
                cloned, deleted_count, unchanged = sync_tree(source, dest, False)
                total_cloned += cloned
                total_deleted += deleted_count
                total_unchanged += unchanged
                if cloned > 0 or deleted_count > 0:
                    _eprint(
                        f"  {item}: {cloned} cloned, {deleted_count} deleted, {unchanged} unchanged"
                    )
            else:
                if clone_item(source, dest, False):
                    total_cloned += 1
                    _eprint(f"  {item}: cloned (new)")
            dir_count += 1
        else:
            if dest.exists():
                try:
                    src_stat: os.stat_result = source.stat()
                    dst_stat: os.stat_result = dest.stat()
                    if (
                        src_stat.st_size == dst_stat.st_size
                        and abs(src_stat.st_mtime - dst_stat.st_mtime) < 0.01
                    ):
                        total_unchanged += 1
                        continue
                except OSError:
                    pass
            if clone_item(source, dest, False):
                total_cloned += 1

    worktree_items: list[str] = discover_untracked_items(worktree_path)
    removed_count: int = 0
    for item in worktree_items:
        if should_exclude(item):
            continue
        if item not in main_items_set:
            dest = worktree_path / item
            if dest.exists() or dest.is_symlink():
                if dest.is_dir() and not dest.is_symlink():
                    shutil.rmtree(dest, ignore_errors=True)
                else:
                    dest.unlink()
                _eprint(f"  Removed extra: {item}")
                removed_count += 1

    total_removed: int = total_deleted + removed_count
    _eprint(
        f"  Synced {dir_count} directories: {total_cloned} cloned, {total_removed} deleted, {total_unchanged} unchanged"
    )


def _claim_reuse(worktree_path: Path, slot: str, branch: str, base: str) -> None:
    """Reuse an available worktree slot for a new branch."""
    old_branch: str | None = _worktree_branch(worktree_path)
    _eprint(f"Reusing slot '{slot}' (was branch '{old_branch}')")

    run_cmd(
        ["git", "checkout", "-B", branch, base],
        capture=True,
        cwd=worktree_path,
    )
    run_cmd(["git", "clean", "-fd"], capture=True, cwd=worktree_path)

    if old_branch and old_branch != branch:
        run_cmd(
            ["git", "branch", "-D", old_branch],
            capture=True,
            check=False,
            cwd=REPO_ROOT,
        )

    main_repo: Path = find_main_repo().resolve()
    _sync_gitignored(worktree_path, main_repo)
    allocate_port(slot)


def _claim_create(worktree_path: Path, slot: str, branch: str, base: str) -> None:
    """Create a new worktree slot."""
    _eprint(f"Creating slot '{slot}'")

    worktree_path.parent.mkdir(parents=True, exist_ok=True)
    result = run_cmd(
        ["git", "worktree", "add", "-b", branch, str(worktree_path), base],
        capture=True,
        check=False,
        cwd=REPO_ROOT,
    )
    if result.returncode != 0:
        _eprint("Error: git worktree add failed")
        sys.exit(1)

    allocate_port(slot)

    main_repo: Path = find_main_repo().resolve()
    _eprint("Discovering untracked/gitignored items...")
    items: list[str] = discover_untracked_items(main_repo)

    clone_count: int = 0
    skip_count: int = 0

    for item in items:
        if should_exclude(item):
            skip_count += 1
            continue

        source: Path = main_repo / item
        dest: Path = worktree_path / item

        if dest.exists() or dest.is_symlink():
            continue

        success: bool = clone_item(source, dest, False)
        if success:
            clone_count += 1

    _eprint(f"  Cloned: {clone_count} items, excluded: {skip_count} items")


def cmd_reset(args: argparse.Namespace) -> None:
    """Remove all worktrees and delete their branches, resetting to a clean state."""
    worktree_paths: list[Path] = list_worktree_paths()

    if not worktree_paths:
        print(f"No worktrees found under {DEFAULT_WORKTREE_BASE}")
        # Still prune and clean up ports in case of stale state
        run_cmd(["git", "worktree", "prune"], cwd=REPO_ROOT)
        write_ports({})
        print("Pruned stale worktree entries and cleared port registry.")
        return

    print(f"Removing {len(worktree_paths)} worktree(s):\n")

    branches_to_delete: list[str] = []

    for worktree_path in worktree_paths:
        branch: str | None = _worktree_branch(worktree_path)
        slot_name: str = worktree_path.name
        print(f"  Removing {slot_name}" + (f" (branch: {branch})" if branch else ""))

        result = run_cmd(
            ["git", "worktree", "remove", "--force", str(worktree_path)],
            check=False,
            cwd=REPO_ROOT,
        )
        if result.returncode != 0:
            print(f"    git worktree remove failed, falling back to rm -rf")
            shutil.rmtree(worktree_path, ignore_errors=True)

        if worktree_path.exists():
            shutil.rmtree(worktree_path, ignore_errors=True)

        if branch and branch != "master" and branch != "main":
            branches_to_delete.append(branch)

    # Prune stale worktree entries
    run_cmd(["git", "worktree", "prune"], cwd=REPO_ROOT)

    # Delete branches
    if branches_to_delete:
        print(f"\nDeleting {len(branches_to_delete)} branch(es):")
        for branch in branches_to_delete:
            print(f"  {branch}")
            run_cmd(
                ["git", "branch", "-D", branch],
                check=False,
                cwd=REPO_ROOT,
            )

    # Clear port registry
    write_ports({})

    print("\nDone! All worktrees removed and ports cleared.")


def cmd_reset_worktrees(args: argparse.Namespace) -> None:
    """Reset all worktrees to latest master and sync gitignored items."""
    worktree_paths: list[Path] = list_worktree_paths()

    if not worktree_paths:
        print(f"No worktrees found under {DEFAULT_WORKTREE_BASE}")
        return

    # Check main repo is clean
    result = run_cmd(
        ["git", "status", "--porcelain"],
        capture=True,
        cwd=REPO_ROOT,
    )
    if result.stdout.strip():
        print("Error: Main repo has uncommitted changes. Commit or stash first.")
        sys.exit(1)

    main_repo: Path = find_main_repo().resolve()

    # Pull latest master
    print("Pulling latest master...")
    run_cmd(["git", "pull", "--ff-only"], cwd=main_repo)

    print(f"\nResetting {len(worktree_paths)} worktree(s) to master:\n")

    branches_to_delete: list[str] = []

    for worktree_path in worktree_paths:
        slot_name: str = worktree_path.name
        branch: str | None = _worktree_branch(worktree_path)
        print(f"--- {slot_name}" + (f" (branch: {branch})" if branch else "") + " ---")

        # Reset tracked files to master
        print("  Resetting to master...")
        result = run_cmd(
            ["git", "reset", "--hard", "master"],
            check=False,
            cwd=worktree_path,
        )
        if result.returncode != 0:
            print(f"  Error: git reset --hard master failed for {slot_name}, skipping")
            continue
        run_cmd(["git", "clean", "-fd"], cwd=worktree_path)

        # Detach HEAD so the old branch can be deleted
        if branch and branch != "master" and branch != "main":
            run_cmd(
                ["git", "checkout", "--detach"],
                check=False,
                cwd=worktree_path,
            )
            branches_to_delete.append(branch)

        # Sync gitignored items
        _sync_gitignored(worktree_path, main_repo)

        # Ensure port is allocated
        port: int = allocate_port(slot_name)
        print(f"  Port: {port}\n")

    # Delete old branches now that worktrees are detached
    if branches_to_delete:
        for branch in branches_to_delete:
            run_cmd(
                ["git", "branch", "-D", branch],
                check=False,
                cwd=REPO_ROOT,
            )

    print(f"Done! All worktrees reset to master.")
    print(f"  Free disk: {get_free_gb(main_repo):.1f}GB")


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


def cmd_claim(args: argparse.Namespace) -> None:
    """Claim an available worktree slot for a new branch."""
    branch: str = args.branch
    base: str = args.base

    if not verify_apfs(Path.home()):
        _eprint("Error: Filesystem is not APFS. APFS clones require an APFS volume.")
        sys.exit(1)

    free_gb: float = get_free_gb(Path.home())
    if free_gb < MIN_FREE_GB:
        _eprint(f"Error: Only {free_gb:.1f}GB free. Need at least {MIN_FREE_GB}GB.")
        sys.exit(1)
    if free_gb < WARN_FREE_GB:
        _eprint(f"Warning: Only {free_gb:.1f}GB free. Proceeding with caution.")

    for slot in POOL_SLOTS:
        slot_path: Path = DEFAULT_WORKTREE_BASE / slot
        if slot_path.exists():
            slot_branch: str | None = _worktree_branch(slot_path)
            if slot_branch == branch:
                _eprint(
                    f"Error: Branch '{branch}' is already checked out in slot '{slot}'"
                )
                sys.exit(1)

    available: list[tuple[str, Path]] = []
    occupied: list[tuple[str, Path]] = []
    empty: list[str] = []

    for slot in POOL_SLOTS:
        slot_path = DEFAULT_WORKTREE_BASE / slot
        if not slot_path.exists():
            empty.append(slot)
        elif _is_worktree_available(slot_path, base):
            available.append((slot, slot_path))
        else:
            occupied.append((slot, slot_path))

    if available:
        slot, slot_path = available[0]
        _claim_reuse(slot_path, slot, branch, base)
        print(slot_path)
    elif empty:
        slot = empty[0]
        slot_path = DEFAULT_WORKTREE_BASE / slot
        _claim_create(slot_path, slot, branch, base)
        print(slot_path)
    else:
        _eprint("Error: All worktree slots are occupied and none are available.")
        _eprint("Occupied slots:")
        for slot, slot_path in occupied:
            slot_branch = _worktree_branch(slot_path)
            _eprint(f"  {slot}: {slot_branch}")
        _eprint("Use 'abu worktree remove <slot>' to free a slot.")
        sys.exit(1)


def register_subcommands(parent_subparsers: argparse._SubParsersAction) -> None:  # type: ignore[type-arg]
    """Register the 'worktree' parent command and its sub-subcommands."""
    worktree_parser = parent_subparsers.add_parser(
        "worktree", help="Manage APFS-backed worktrees"
    )
    wt_sub = worktree_parser.add_subparsers(dest="worktree_command", required=True)

    create_parser = wt_sub.add_parser("create", help="Create a new worktree")
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

    remove_parser = wt_sub.add_parser("remove", help="Remove a worktree")
    remove_parser.add_argument(
        "target",
        help="Branch name or path to worktree",
    )
    remove_parser.add_argument(
        "--delete-branch",
        action="store_true",
        help="Also delete the git branch",
    )

    refresh_parser = wt_sub.add_parser(
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

    activate_parser = wt_sub.add_parser(
        "activate",
        help="Reset an existing worktree to match a fresh creation from the base branch",
    )
    activate_parser.add_argument("branch", help="Worktree name to activate")
    activate_parser.add_argument(
        "--base",
        default="master",
        help="Base branch to reset to (default: master)",
    )
    activate_parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Show what would be done without doing it",
    )

    wt_sub.add_parser("list", help="List worktrees")
    wt_sub.add_parser("reset", help="Remove all worktrees and delete their branches")

    claim_parser = wt_sub.add_parser("claim", help="Claim an available worktree slot")
    claim_parser.add_argument("branch", help="Branch name to create")
    claim_parser.add_argument(
        "--base",
        default="master",
        help="Base branch (default: master)",
    )


def dispatch(args: argparse.Namespace) -> None:
    """Route to the correct worktree subcommand handler."""
    cmd: str = args.worktree_command
    if cmd == "create":
        cmd_create(args)
    elif cmd == "remove":
        cmd_remove(args)
    elif cmd == "refresh":
        cmd_refresh(args)
    elif cmd == "activate":
        cmd_activate(args)
    elif cmd == "list":
        cmd_list(args)
    elif cmd == "claim":
        cmd_claim(args)
    elif cmd == "reset":
        cmd_reset(args)


def main() -> None:
    """Standalone entry point for running worktree commands directly."""
    parser: argparse.ArgumentParser = argparse.ArgumentParser(
        description="APFS-backed ephemeral git worktrees",
    )
    subparsers = parser.add_subparsers(dest="worktree_command", required=True)

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

    activate_parser = subparsers.add_parser(
        "activate",
        help="Reset an existing worktree to match a fresh creation from the base branch",
    )
    activate_parser.add_argument("branch", help="Worktree name to activate")
    activate_parser.add_argument(
        "--base",
        default="master",
        help="Base branch to reset to (default: master)",
    )
    activate_parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Show what would be done without doing it",
    )

    subparsers.add_parser("list", help="List worktrees")
    subparsers.add_parser(
        "reset", help="Remove all worktrees and delete their branches"
    )

    claim_parser = subparsers.add_parser(
        "claim", help="Claim an available worktree slot"
    )
    claim_parser.add_argument("branch", help="Branch name to create")
    claim_parser.add_argument(
        "--base",
        default="master",
        help="Base branch (default: master)",
    )

    parsed_args: argparse.Namespace = parser.parse_args()
    dispatch(parsed_args)


if __name__ == "__main__":
    main()
