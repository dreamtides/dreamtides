#!/usr/bin/env python3
"""Persistent filesystem task manager for Codex skills."""

from __future__ import annotations

import argparse
import fcntl
import json
import os
import socket
import sys
from collections.abc import Iterable, Iterator
from contextlib import contextmanager
from datetime import datetime, timedelta, timezone
from pathlib import Path


VALID_STATUSES = ("todo", "in_progress", "blocked", "done", "canceled")
DONE_STATUS = "done"
INDEX_VERSION = 1
MIN_ID_WIDTH = 4
CLAIM_LEASE_SECONDS_DEFAULT = 4 * 60 * 60
TASK_INDEX_LOCK_FILE = "index.lock"
UTC_TIMESTAMP_FORMAT = "%Y-%m-%dT%H:%M:%SZ"
DEFAULT_TASK_STORE_ROOT = Path("/tmp/codex")
TASKS_DIR_NAME = "tasks"
TASK_ITEMS_DIR_NAME = "items"
LEGACY_TASK_FILE_PREFIX = ".codex/tasks/"


class TaskError(Exception):
    """Raised when a task operation cannot be completed safely."""


def utc_timestamp() -> str:
    """Return a stable UTC timestamp format used by task metadata."""
    return format_utc_timestamp(datetime.now(timezone.utc))


def format_utc_timestamp(value: datetime) -> str:
    """Format a timezone-aware timestamp as UTC metadata text."""
    return value.astimezone(timezone.utc).strftime(UTC_TIMESTAMP_FORMAT)


def parse_utc_timestamp(value: str) -> datetime | None:
    """Parse a UTC timestamp string or return None when invalid."""
    try:
        parsed = datetime.strptime(value, UTC_TIMESTAMP_FORMAT)
    except ValueError:
        return None
    return parsed.replace(tzinfo=timezone.utc)


def parse_task_id(task_id: str) -> int:
    """Parse a task identifier and return its integer component."""
    if len(task_id) < 2 or not task_id.startswith("T"):
        raise TaskError(f"Invalid task ID '{task_id}'. Expected format T0001.")
    digits = task_id[1:]
    if not digits.isdigit():
        raise TaskError(f"Invalid task ID '{task_id}'. Expected format T0001.")
    return int(digits)


def format_task_id(next_id: int) -> str:
    """Format the numeric identifier using the configured zero-padding width."""
    width = max(MIN_ID_WIDTH, len(str(next_id)))
    return f"T{next_id:0{width}d}"


def parse_csv_ids(raw: str | None) -> list[str]:
    """Parse comma-separated task IDs."""
    if raw is None:
        return []
    if raw.strip() == "":
        return []
    values = [value.strip() for value in raw.split(",")]
    clean_values: list[str] = []
    for value in values:
        if value == "":
            continue
        parse_task_id(value)
        clean_values.append(value)
    return dedupe_preserve_order(clean_values)


def parse_id_inputs(raw_values: Iterable[str]) -> list[str]:
    """Parse repeated comma-separated ID arguments."""
    parsed: list[str] = []
    for raw_value in raw_values:
        parsed.extend(parse_csv_ids(raw_value))
    return dedupe_preserve_order(parsed)


def dedupe_preserve_order(items: Iterable[str]) -> list[str]:
    """Remove duplicates while preserving first-seen order."""
    seen: set[str] = set()
    ordered: list[str] = []
    for item in items:
        if item in seen:
            continue
        seen.add(item)
        ordered.append(item)
    return ordered


def store_paths(root: Path) -> dict[str, Path]:
    """Resolve all task store paths relative to the provided root directory."""
    tasks_dir = root / TASKS_DIR_NAME
    items_dir = tasks_dir / TASK_ITEMS_DIR_NAME
    index_path = tasks_dir / "index.json"
    return {
        "root": root,
        "tasks": tasks_dir,
        "items": items_dir,
        "index": index_path,
    }


@contextmanager
def index_write_lock(paths: dict[str, Path]) -> Iterator[None]:
    """Hold an exclusive cross-process lock for index mutations."""
    lock_path = paths["tasks"] / TASK_INDEX_LOCK_FILE
    lock_path.parent.mkdir(parents=True, exist_ok=True)
    try:
        lock_handle = lock_path.open("a+", encoding="utf-8")
    except OSError as error:
        raise TaskError(f"Failed to open lock file '{lock_path}': {error}") from error
    try:
        try:
            fcntl.flock(lock_handle.fileno(), fcntl.LOCK_EX)
        except OSError as error:
            raise TaskError(f"Failed to lock '{lock_path}': {error}") from error
        yield
    finally:
        try:
            fcntl.flock(lock_handle.fileno(), fcntl.LOCK_UN)
        except OSError:
            pass
        lock_handle.close()


def default_claimant() -> str:
    """Build default claimant identifier for task leases."""
    user_name = os.environ.get("USER") or "unknown"
    host_name = socket.gethostname().split(".", maxsplit=1)[0]
    return f"{user_name}@{host_name}:{os.getpid()}"


def clear_claim(task: dict[str, object]) -> None:
    """Clear lease metadata on a task."""
    task["claimed_by"] = None
    task["claimed_at"] = None
    task["lease_expires_at"] = None


def claim_task(
    task: dict[str, object],
    claimant: str,
    now: datetime,
    lease_seconds: int,
) -> None:
    """Mark a task as in-progress and assign lease metadata."""
    if lease_seconds < 1:
        raise TaskError("Lease duration must be at least 1 second.")
    task["status"] = "in_progress"
    task["claimed_by"] = claimant
    task["claimed_at"] = format_utc_timestamp(now)
    task["lease_expires_at"] = format_utc_timestamp(now + timedelta(seconds=lease_seconds))
    task["updated_at"] = format_utc_timestamp(now)


def normalize_claim_shape(task: dict[str, object]) -> None:
    """Ensure claim metadata keys exist on task dictionaries."""
    if "claimed_by" not in task:
        task["claimed_by"] = None
    if "claimed_at" not in task:
        task["claimed_at"] = None
    if "lease_expires_at" not in task:
        task["lease_expires_at"] = None


def reclaim_stale_in_progress_tasks(
    tasks: list[dict[str, object]], now: datetime
) -> list[dict[str, object]]:
    """Move expired in-progress tasks back to todo."""
    reclaimed: list[dict[str, object]] = []
    now_text = format_utc_timestamp(now)
    for task in tasks:
        if task.get("status") != "in_progress":
            continue
        lease_expires_at = task.get("lease_expires_at")
        if not isinstance(lease_expires_at, str):
            continue
        lease_expires_at_dt = parse_utc_timestamp(lease_expires_at)
        if lease_expires_at_dt is None or lease_expires_at_dt > now:
            continue
        task["status"] = "todo"
        task["updated_at"] = now_text
        clear_claim(task)
        reclaimed.append(task)
    return reclaimed


def default_index() -> dict[str, object]:
    """Return an empty task store document."""
    return {"version": INDEX_VERSION, "next_id": 1, "tasks": []}


def ensure_store_exists(paths: dict[str, Path]) -> None:
    """Ensure task store directories and index exist."""
    if not paths["index"].exists():
        raise TaskError("Task store is not initialized. Run '.codex/scripts/task.py init' first.")


def load_index(paths: dict[str, Path]) -> dict[str, object]:
    """Load and parse the index file."""
    ensure_store_exists(paths)
    try:
        raw_text = paths["index"].read_text(encoding="utf-8")
    except OSError as error:
        raise TaskError(f"Failed to read {paths['index']}: {error}") from error
    try:
        payload = json.loads(raw_text)
    except json.JSONDecodeError as error:
        raise TaskError(
            "Task index is corrupt JSON. Restore from git or a backup, then rerun."
        ) from error
    if not isinstance(payload, dict):
        raise TaskError("Task index root must be a JSON object.")
    return payload


def atomic_write_json(path: Path, payload: dict[str, object]) -> None:
    """Atomically write JSON payload to disk."""
    path.parent.mkdir(parents=True, exist_ok=True)
    tmp_path = path.with_suffix(path.suffix + ".tmp")
    content = json.dumps(payload, indent=2)
    try:
        tmp_path.write_text(f"{content}\n", encoding="utf-8")
        tmp_path.replace(path)
    except OSError as error:
        raise TaskError(f"Failed to write {path}: {error}") from error


def markdown_body_from_text(markdown_text: str) -> str:
    """Return markdown body with frontmatter stripped if present."""
    text = markdown_text.strip()
    if not text.startswith("---\n"):
        return text
    split_marker = "\n---\n"
    marker_index = text.find(split_marker, len("---\n"))
    if marker_index < 0:
        return text
    return text[marker_index + len(split_marker) :].strip()


def read_markdown_from_path(path: Path) -> str:
    """Read markdown text from a filesystem path."""
    try:
        return path.read_text(encoding="utf-8")
    except OSError as error:
        raise TaskError(f"Failed to read markdown file '{path}': {error}") from error


def read_markdown_from_stdin() -> str:
    """Read markdown text from standard input."""
    markdown_text = sys.stdin.read()
    if markdown_text.strip() == "":
        raise TaskError("Expected markdown from stdin, but stdin was empty.")
    return markdown_text


def task_file_path(root: Path, task: dict[str, object]) -> Path:
    """Resolve a task's markdown path."""
    task_file = task.get("task_file")
    if not isinstance(task_file, str) or task_file == "":
        raise TaskError("Task metadata is missing a valid task_file value.")
    raw_path = Path(task_file)
    if raw_path.is_absolute():
        return raw_path
    if task_file.startswith(LEGACY_TASK_FILE_PREFIX):
        raw_path = Path(task_file.removeprefix(LEGACY_TASK_FILE_PREFIX))
    if raw_path.parts and raw_path.parts[0] == TASKS_DIR_NAME:
        raw_path = Path(*raw_path.parts[1:])
    return store_paths(root)["tasks"] / raw_path


def render_task_markdown(task: dict[str, object], body: str) -> str:
    """Render task frontmatter and body markdown."""
    task_id = task["id"]
    title = task["title"]
    status = task["status"]
    blocked_by = task["blocked_by"]
    stripped_body = body.strip()
    lines = [
        "---",
        f"id: {task_id}",
        f"title: {json.dumps(title)}",
        f"status: {status}",
        f"blocked_by: {json.dumps(blocked_by)}",
        "---",
        "",
        stripped_body,
        "",
    ]
    return "\n".join(lines)


def split_task_markdown(markdown_text: str) -> tuple[str | None, str]:
    """Split frontmatter and body from a markdown document."""
    text = markdown_text.strip()
    if not text.startswith("---\n"):
        return (None, text)
    split_marker = "\n---\n"
    marker_index = text.find(split_marker, len("---\n"))
    if marker_index < 0:
        return (None, text)
    frontmatter = text[len("---\n") : marker_index]
    body = text[marker_index + len(split_marker) :].strip()
    return (frontmatter, body)


def write_task_markdown(
    root: Path,
    task: dict[str, object],
    body_override: str | None = None,
    append_text: str | None = None,
) -> None:
    """Write a task markdown file while mirroring current metadata in frontmatter."""
    path = task_file_path(root, task)
    existing_body = ""
    if path.exists():
        existing_text = read_markdown_from_path(path)
        (_, existing_body) = split_task_markdown(existing_text)
    if body_override is not None:
        body = markdown_body_from_text(body_override)
    elif append_text is not None:
        append_body = markdown_body_from_text(append_text)
        if existing_body.strip() == "":
            body = append_body
        elif append_body.strip() == "":
            body = existing_body
        else:
            body = f"{existing_body.rstrip()}\n\n{append_body.lstrip()}"
    else:
        body = existing_body
    path.parent.mkdir(parents=True, exist_ok=True)
    rendered = render_task_markdown(task, body)
    try:
        path.write_text(rendered, encoding="utf-8")
    except OSError as error:
        raise TaskError(f"Failed to write task markdown '{path}': {error}") from error


def task_sort_key(task: dict[str, object]) -> int:
    """Sort tasks by numeric ID."""
    return parse_task_id(str(task["id"]))


def index_tasks(index_payload: dict[str, object]) -> list[dict[str, object]]:
    """Return tasks list from index payload."""
    tasks = index_payload.get("tasks")
    if not isinstance(tasks, list):
        raise TaskError("Task index is invalid: 'tasks' must be a list.")
    if not all(isinstance(task, dict) for task in tasks):
        raise TaskError("Task index is invalid: each task entry must be an object.")
    return tasks


def get_task_by_id(tasks: list[dict[str, object]], task_id: str) -> dict[str, object]:
    """Return a task object by its identifier."""
    parse_task_id(task_id)
    for task in tasks:
        if task.get("id") == task_id:
            return task
    raise TaskError(f"Task '{task_id}' was not found.")


def blockers_resolved(task: dict[str, object], by_id: dict[str, dict[str, object]]) -> bool:
    """Return whether all blockers are currently done."""
    for blocker_id in task["blocked_by"]:
        blocker = by_id[blocker_id]
        if blocker["status"] != DONE_STATUS:
            return False
    return True


def ready_tasks(tasks: list[dict[str, object]]) -> list[dict[str, object]]:
    """Return all ready-to-start tasks."""
    by_id = {str(task["id"]): task for task in tasks}
    results: list[dict[str, object]] = []
    for task in tasks:
        if task["status"] != "todo":
            continue
        if blockers_resolved(task, by_id):
            results.append(task)
    return sorted(results, key=task_sort_key)


def validate_index(
    index_payload: dict[str, object], root: Path, check_files: bool
) -> list[str]:
    """Validate index structure, relationships, and optional file existence."""
    errors: list[str] = []
    version = index_payload.get("version")
    if version != INDEX_VERSION:
        errors.append(
            f"Unsupported index version '{version}'. Expected {INDEX_VERSION}."
        )
    next_id = index_payload.get("next_id")
    if not isinstance(next_id, int) or next_id < 1:
        errors.append("Index field 'next_id' must be an integer >= 1.")
    tasks = index_payload.get("tasks")
    if not isinstance(tasks, list):
        errors.append("Index field 'tasks' must be a list.")
        return errors

    seen_ids: set[str] = set()
    by_id: dict[str, dict[str, object]] = {}

    for index, task in enumerate(tasks):
        task_label = f"tasks[{index}]"
        if not isinstance(task, dict):
            errors.append(f"{task_label} must be an object.")
            continue
        task_id = task.get("id")
        title = task.get("title")
        status = task.get("status")
        blocked_by = task.get("blocked_by")
        task_file = task.get("task_file")
        created_at = task.get("created_at")
        updated_at = task.get("updated_at")
        claimed_by = task.get("claimed_by")
        claimed_at = task.get("claimed_at")
        lease_expires_at = task.get("lease_expires_at")

        if not isinstance(task_id, str):
            errors.append(f"{task_label}.id must be a string.")
            continue
        try:
            parse_task_id(task_id)
        except TaskError as error:
            errors.append(f"{task_label}.id {error}")
        if task_id in seen_ids:
            errors.append(f"Duplicate task ID '{task_id}'.")
            continue
        seen_ids.add(task_id)
        by_id[task_id] = task

        if not isinstance(title, str) or title.strip() == "":
            errors.append(f"{task_label}.title must be a non-empty string.")
        if not isinstance(status, str) or status not in VALID_STATUSES:
            errors.append(
                f"{task_label}.status must be one of {', '.join(VALID_STATUSES)}."
            )
        if not isinstance(blocked_by, list) or not all(
            isinstance(blocker, str) for blocker in blocked_by
        ):
            errors.append(f"{task_label}.blocked_by must be a list of task IDs.")
        if not isinstance(task_file, str) or task_file.strip() == "":
            errors.append(f"{task_label}.task_file must be a non-empty string.")
        if not isinstance(created_at, str) or created_at.strip() == "":
            errors.append(f"{task_label}.created_at must be a non-empty string.")
        if not isinstance(updated_at, str) or updated_at.strip() == "":
            errors.append(f"{task_label}.updated_at must be a non-empty string.")
        if claimed_by is not None and (
            not isinstance(claimed_by, str) or claimed_by.strip() == ""
        ):
            errors.append(f"{task_label}.claimed_by must be null or a non-empty string.")
        if claimed_at is not None and (
            not isinstance(claimed_at, str) or parse_utc_timestamp(claimed_at) is None
        ):
            errors.append(
                f"{task_label}.claimed_at must be null or UTC timestamp {UTC_TIMESTAMP_FORMAT}."
            )
        if lease_expires_at is not None and (
            not isinstance(lease_expires_at, str)
            or parse_utc_timestamp(lease_expires_at) is None
        ):
            errors.append(
                f"{task_label}.lease_expires_at must be null or UTC timestamp {UTC_TIMESTAMP_FORMAT}."
            )
        if check_files and isinstance(task_file, str):
            try:
                file_path = task_file_path(root, task)
            except TaskError as error:
                errors.append(f"{task_label}.task_file {error}")
                continue
            if not file_path.exists():
                errors.append(f"{task_id} references missing task file '{task_file}'.")

    for task in tasks:
        if not isinstance(task, dict):
            continue
        task_id = task.get("id")
        blocked_by = task.get("blocked_by")
        if not isinstance(task_id, str) or not isinstance(blocked_by, list):
            continue
        for blocker in blocked_by:
            if blocker == task_id:
                errors.append(f"{task_id} cannot block itself.")
            if blocker not in by_id:
                errors.append(f"{task_id} depends on unknown blocker '{blocker}'.")

    graph: dict[str, list[str]] = {}
    for task in tasks:
        if not isinstance(task, dict):
            continue
        task_id = task.get("id")
        blocked_by = task.get("blocked_by")
        if not isinstance(task_id, str) or not isinstance(blocked_by, list):
            continue
        graph[task_id] = [blocker for blocker in blocked_by if blocker in by_id]
    visiting: set[str] = set()
    visited: set[str] = set()
    cycle_found = False

    def visit(node: str) -> None:
        nonlocal cycle_found
        if cycle_found:
            return
        if node in visited:
            return
        if node in visiting:
            cycle_found = True
            errors.append(f"Dependency cycle detected involving '{node}'.")
            return
        visiting.add(node)
        for dependency in graph.get(node, []):
            visit(dependency)
        visiting.remove(node)
        visited.add(node)

    for node in graph:
        visit(node)
        if cycle_found:
            break
    return errors


def print_tasks_table(tasks: list[dict[str, object]]) -> None:
    """Print tasks in compact table format."""
    if not tasks:
        print("No tasks found.")
        return
    print("ID      STATUS       BLOCKED_BY  TITLE")
    for task in tasks:
        blocked_text = ",".join(task["blocked_by"]) if task["blocked_by"] else "-"
        print(
            f"{task['id']:<7} {task['status']:<12} {blocked_text:<10} {task['title']}"
        )


def print_ready_table(tasks: list[dict[str, object]]) -> None:
    """Print minimal table for ready tasks."""
    if not tasks:
        print("No ready tasks.")
        return
    print("ID      TITLE")
    for task in tasks:
        print(f"{task['id']:<7} {task['title']}")


def cmd_init(args: argparse.Namespace) -> int:
    """Initialize the task store."""
    root = Path(args.root).resolve()
    paths = store_paths(root)
    with index_write_lock(paths):
        paths["items"].mkdir(parents=True, exist_ok=True)
        if paths["index"].exists():
            print(f"Task store already initialized at {paths['index']}.")
            return 0
        atomic_write_json(paths["index"], default_index())
    print(f"Initialized task store at {paths['index']}.")
    return 0


def load_store_or_raise(
    root: Path,
) -> tuple[dict[str, Path], dict[str, object], list[dict[str, object]]]:
    """Load store and tasks with structural validation."""
    paths = store_paths(root)
    index_payload = load_index(paths)
    tasks = index_tasks(index_payload)
    errors = validate_index(index_payload, root, check_files=False)
    if errors:
        rendered = "\n".join(f"- {error}" for error in errors)
        raise TaskError(f"Task index validation failed:\n{rendered}")
    return (paths, index_payload, tasks)


@contextmanager
def mutable_store_or_raise(
    root: Path,
) -> Iterator[tuple[dict[str, Path], dict[str, object], list[dict[str, object]]]]:
    """Load a task store under an exclusive mutation lock."""
    paths = store_paths(root)
    with index_write_lock(paths):
        index_payload = load_index(paths)
        tasks = index_tasks(index_payload)
        for task in tasks:
            normalize_claim_shape(task)
        errors = validate_index(index_payload, root, check_files=False)
        if errors:
            rendered = "\n".join(f"- {error}" for error in errors)
            raise TaskError(f"Task index validation failed:\n{rendered}")
        yield (paths, index_payload, tasks)


def cmd_add(args: argparse.Namespace) -> int:
    """Create a new task."""
    root = Path(args.root).resolve()
    if args.markdown_file is not None:
        markdown_text = read_markdown_from_path(Path(args.markdown_file))
    else:
        markdown_text = read_markdown_from_stdin()

    with mutable_store_or_raise(root) as (paths, index_payload, tasks):
        status = args.status
        if status not in VALID_STATUSES:
            raise TaskError(f"Invalid status '{status}'.")
        blocked_by = parse_csv_ids(args.blocked_by)
        existing_ids = {str(task["id"]) for task in tasks}
        for blocker in blocked_by:
            if blocker not in existing_ids:
                raise TaskError(f"Cannot add task with unknown blocker '{blocker}'.")

        next_id = index_payload.get("next_id")
        if not isinstance(next_id, int) or next_id < 1:
            raise TaskError("Task index is invalid: 'next_id' must be >= 1.")
        task_id = format_task_id(next_id)
        task_file = str(Path(TASK_ITEMS_DIR_NAME) / f"{task_id}.md")
        now_dt = datetime.now(timezone.utc)
        now = format_utc_timestamp(now_dt)
        task = {
            "id": task_id,
            "title": args.title,
            "status": status,
            "blocked_by": blocked_by,
            "task_file": task_file,
            "created_at": now,
            "updated_at": now,
        }
        clear_claim(task)
        if status == "in_progress":
            claim_task(task, default_claimant(), now_dt, CLAIM_LEASE_SECONDS_DEFAULT)

        tasks.append(task)
        index_payload["next_id"] = next_id + 1
        write_task_markdown(root, task, body_override=markdown_text)
        errors = validate_index(index_payload, root, check_files=True)
        if errors:
            raise TaskError(
                "Cannot add task:\n" + "\n".join(f"- {error}" for error in errors)
            )
        atomic_write_json(paths["index"], index_payload)
    print(f"Created {task_id} -> {task_file}")
    return 0


def filtered_tasks(args: argparse.Namespace, tasks: list[dict[str, object]]) -> list[dict[str, object]]:
    """Apply list command filters."""
    values = list(tasks)
    if args.status is not None:
        values = [task for task in values if task["status"] == args.status]
    elif args.ready:
        values = ready_tasks(values)
        return values
    elif not args.all:
        values = [
            task
            for task in values
            if task["status"] not in ("done", "canceled")
        ]
    return sorted(values, key=task_sort_key)


def cmd_list(args: argparse.Namespace) -> int:
    """List tasks with optional filters."""
    root = Path(args.root).resolve()
    (_, _, tasks) = load_store_or_raise(root)
    results = filtered_tasks(args, tasks)
    if args.json:
        print(json.dumps(results, indent=2))
        return 0
    print_tasks_table(results)
    return 0


def cmd_get(args: argparse.Namespace) -> int:
    """Get a single task."""
    root = Path(args.root).resolve()
    (_, _, tasks) = load_store_or_raise(root)
    task = get_task_by_id(tasks, args.task_id)
    if not args.body and args.json:
        print(json.dumps(task, indent=2))
        return 0
    if not args.body:
        print(f"id: {task['id']}")
        print(f"title: {task['title']}")
        print(f"status: {task['status']}")
        print(f"blocked_by: {json.dumps(task['blocked_by'])}")
        print(f"task_file: {task['task_file']}")
        print(f"created_at: {task['created_at']}")
        print(f"updated_at: {task['updated_at']}")
        return 0
    markdown_path = task_file_path(root, task)
    body = read_markdown_from_path(markdown_path)
    if args.json:
        payload = dict(task)
        payload["markdown"] = body
        print(json.dumps(payload, indent=2))
        return 0
    print(body)
    return 0


def cmd_update(args: argparse.Namespace) -> int:
    """Update task metadata and markdown content."""
    root = Path(args.root).resolve()
    markdown_override: str | None = None
    append_text: str | None = None
    if args.replace_markdown_file is not None:
        markdown_override = read_markdown_from_path(Path(args.replace_markdown_file))
    if args.append_markdown_file is not None:
        append_text = read_markdown_from_path(Path(args.append_markdown_file))

    with mutable_store_or_raise(root) as (paths, index_payload, tasks):
        task = get_task_by_id(tasks, args.task_id)

        blockers = list(task["blocked_by"])
        if args.set_blocked_by is not None:
            blockers = parse_csv_ids(args.set_blocked_by)
        blockers.extend(parse_id_inputs(args.add_blocker))
        blockers = dedupe_preserve_order(blockers)
        blockers = [
            blocker
            for blocker in blockers
            if blocker not in parse_id_inputs(args.remove_blocker)
        ]

        existing_ids = {str(item["id"]) for item in tasks}
        for blocker in blockers:
            if blocker not in existing_ids:
                raise TaskError(f"Unknown blocker '{blocker}'.")
        if task["id"] in blockers:
            raise TaskError("A task cannot block itself.")

        if args.title is not None:
            task["title"] = args.title
        if args.status is not None:
            if args.status not in VALID_STATUSES:
                raise TaskError(f"Invalid status '{args.status}'.")
            task["status"] = args.status
        task["blocked_by"] = blockers

        if task["status"] == "in_progress":
            now_dt = datetime.now(timezone.utc)
            claimed_by = task.get("claimed_by")
            claimed_at = task.get("claimed_at")
            lease_expires_at = task.get("lease_expires_at")
            if (
                not isinstance(claimed_by, str)
                or claimed_by.strip() == ""
                or not isinstance(claimed_at, str)
                or parse_utc_timestamp(claimed_at) is None
                or not isinstance(lease_expires_at, str)
                or parse_utc_timestamp(lease_expires_at) is None
            ):
                claim_task(
                    task,
                    default_claimant(),
                    now_dt,
                    CLAIM_LEASE_SECONDS_DEFAULT,
                )
            else:
                task["updated_at"] = format_utc_timestamp(now_dt)
        else:
            clear_claim(task)
            task["updated_at"] = utc_timestamp()

        errors = validate_index(index_payload, root, check_files=True)
        if errors:
            raise TaskError(
                "Cannot update task because validation failed:\n"
                + "\n".join(f"- {error}" for error in errors)
            )
        write_task_markdown(
            root,
            task,
            body_override=markdown_override,
            append_text=append_text,
        )
        atomic_write_json(paths["index"], index_payload)
    print(f"Updated {task['id']}.")
    return 0


def cmd_done(args: argparse.Namespace) -> int:
    """Mark a task as done."""
    root = Path(args.root).resolve()
    with mutable_store_or_raise(root) as (paths, index_payload, tasks):
        task = get_task_by_id(tasks, args.task_id)
        by_id = {str(item["id"]): item for item in tasks}
        unresolved = [
            blocker
            for blocker in task["blocked_by"]
            if by_id[blocker]["status"] != DONE_STATUS
        ]
        if unresolved and not args.force:
            raise TaskError(
                "Cannot mark task done while blockers are unresolved: "
                + ", ".join(unresolved)
                + ". Use --force to override."
            )
        task["status"] = DONE_STATUS
        task["updated_at"] = utc_timestamp()
        clear_claim(task)
        write_task_markdown(root, task)
        errors = validate_index(index_payload, root, check_files=True)
        if errors:
            raise TaskError(
                "Cannot mark task done because validation failed:\n"
                + "\n".join(f"- {error}" for error in errors)
            )
        atomic_write_json(paths["index"], index_payload)
    print(f"Marked {task['id']} as done.")
    return 0


def cmd_start(args: argparse.Namespace) -> int:
    """Claim the oldest ready task and mark it in progress."""
    root = Path(args.root).resolve()
    if args.lease_seconds < 1:
        raise TaskError("Lease duration must be at least 1 second.")
    with mutable_store_or_raise(root) as (paths, index_payload, tasks):
        now_dt = datetime.now(timezone.utc)
        reclaimed = reclaim_stale_in_progress_tasks(tasks, now_dt)
        for reclaimed_task in reclaimed:
            write_task_markdown(root, reclaimed_task)

        values = ready_tasks(tasks)
        if not values:
            if reclaimed:
                errors = validate_index(index_payload, root, check_files=True)
                if errors:
                    raise TaskError(
                        "Cannot reclaim stale tasks because validation failed:\n"
                        + "\n".join(f"- {error}" for error in errors)
                    )
                atomic_write_json(paths["index"], index_payload)
            if args.id_only:
                return 0
            if args.json:
                print("null")
                return 0
            print("No ready tasks.")
            return 0

        task = values[0]
        claim_task(task, args.claimant or default_claimant(), now_dt, args.lease_seconds)
        write_task_markdown(root, task)
        errors = validate_index(index_payload, root, check_files=True)
        if errors:
            raise TaskError(
                "Cannot start task because validation failed:\n"
                + "\n".join(f"- {error}" for error in errors)
            )
        atomic_write_json(paths["index"], index_payload)

    if args.id_only:
        print(task["id"])
        return 0
    if args.json:
        payload = dict(task)
        if args.body:
            payload["markdown"] = read_markdown_from_path(task_file_path(root, task))
        print(json.dumps(payload, indent=2))
        return 0
    if args.body:
        print(read_markdown_from_path(task_file_path(root, task)))
        return 0
    print(f"Started {task['id']}.")
    return 0


def cmd_release(args: argparse.Namespace) -> int:
    """Return an active task to todo status."""
    root = Path(args.root).resolve()
    with mutable_store_or_raise(root) as (paths, index_payload, tasks):
        task = get_task_by_id(tasks, args.task_id)
        if task["status"] in ("done", "canceled") and not args.force:
            raise TaskError(
                f"Cannot release task '{task['id']}' from status '{task['status']}' without --force."
            )

        task["status"] = "todo"
        task["updated_at"] = utc_timestamp()
        clear_claim(task)
        write_task_markdown(root, task)
        errors = validate_index(index_payload, root, check_files=True)
        if errors:
            raise TaskError(
                "Cannot release task because validation failed:\n"
                + "\n".join(f"- {error}" for error in errors)
            )
        atomic_write_json(paths["index"], index_payload)
    print(f"Released {task['id']} to todo.")
    return 0


def cmd_finish(args: argparse.Namespace) -> int:
    """Alias for done to support start/finish workflow."""
    return cmd_done(args)


def cmd_ready(args: argparse.Namespace) -> int:
    """List ready tasks with minimal output."""
    root = Path(args.root).resolve()
    (_, _, tasks) = load_store_or_raise(root)
    values = ready_tasks(tasks)
    if args.json:
        print(json.dumps(values, indent=2))
        return 0
    print_ready_table(values)
    return 0


def cmd_validate(args: argparse.Namespace) -> int:
    """Validate task store consistency."""
    root = Path(args.root).resolve()
    paths = store_paths(root)
    index_payload = load_index(paths)
    errors = validate_index(index_payload, root, check_files=True)
    if errors:
        print("Task store is invalid:", file=sys.stderr)
        for error in errors:
            print(f"- {error}", file=sys.stderr)
        return 1
    print("Task store is valid.")
    return 0


def topological_tasks(tasks: list[dict[str, object]]) -> list[dict[str, object]]:
    """Return tasks in topological order, tie-breaking by numeric task ID."""
    by_id = {str(task["id"]): task for task in tasks}
    dependents: dict[str, list[str]] = {task_id: [] for task_id in by_id}
    indegree: dict[str, int] = {task_id: 0 for task_id in by_id}

    for task in tasks:
        task_id = str(task["id"])
        blockers = [str(blocker) for blocker in task["blocked_by"]]
        indegree[task_id] = len(blockers)
        for blocker in blockers:
            dependents[blocker].append(task_id)

    ready = sorted(
        [task_id for task_id, degree in indegree.items() if degree == 0],
        key=parse_task_id,
    )
    ordered_ids: list[str] = []
    while ready:
        task_id = ready.pop(0)
        ordered_ids.append(task_id)
        for dependent_id in sorted(dependents[task_id], key=parse_task_id):
            indegree[dependent_id] -= 1
            if indegree[dependent_id] == 0:
                ready.append(dependent_id)
                ready.sort(key=parse_task_id)

    if len(ordered_ids) != len(tasks):
        raise TaskError("Cannot renumber tasks: dependency cycle detected.")
    return [by_id[task_id] for task_id in ordered_ids]


def ordered_tasks_for_renumber(
    tasks: list[dict[str, object]], ordering: str
) -> list[dict[str, object]]:
    """Return task ordering used by renumber command."""
    if ordering == "id":
        return sorted(tasks, key=task_sort_key)
    if ordering == "created":
        return sorted(tasks, key=lambda task: (str(task["created_at"]), task_sort_key(task)))
    return topological_tasks(tasks)


def cmd_renumber(args: argparse.Namespace) -> int:
    """Renumber task IDs sequentially and rewrite blockers/file paths."""
    root = Path(args.root).resolve()
    with mutable_store_or_raise(root) as (paths, index_payload, tasks):
        ordered_tasks = ordered_tasks_for_renumber(tasks, args.by)
        old_ids = [str(task["id"]) for task in ordered_tasks]
        new_ids = [format_task_id(index + 1) for index in range(len(ordered_tasks))]
        id_map = {old_id: new_id for old_id, new_id in zip(old_ids, new_ids, strict=True)}

        if args.json:
            print(json.dumps(id_map, indent=2))
            if args.dry_run:
                return 0
        elif args.dry_run:
            for old_id in old_ids:
                print(f"{old_id} -> {id_map[old_id]}")
            return 0

        bodies_by_old_id: dict[str, str] = {}
        for task in tasks:
            task_id = str(task["id"])
            path = task_file_path(root, task)
            markdown_text = read_markdown_from_path(path)
            (_, body) = split_task_markdown(markdown_text)
            bodies_by_old_id[task_id] = body

        now = utc_timestamp()
        remapped_tasks: list[dict[str, object]] = []
        for task in ordered_tasks:
            old_id = str(task["id"])
            new_id = id_map[old_id]
            remapped_task = dict(task)
            remapped_task["id"] = new_id
            remapped_task["blocked_by"] = [
                id_map[str(blocker)] for blocker in task["blocked_by"]
            ]
            remapped_task["task_file"] = str(Path(TASK_ITEMS_DIR_NAME) / f"{new_id}.md")
            remapped_task["updated_at"] = now
            remapped_tasks.append(remapped_task)

        index_payload["tasks"] = remapped_tasks
        index_payload["next_id"] = len(remapped_tasks) + 1
        errors = validate_index(index_payload, root, check_files=False)
        if errors:
            raise TaskError(
                "Cannot renumber tasks because validation failed:\n"
                + "\n".join(f"- {error}" for error in errors)
            )

        for task in remapped_tasks:
            old_id = next(old for old, new in id_map.items() if new == task["id"])
            write_task_markdown(root, task, body_override=bodies_by_old_id[old_id])

        new_task_files = {str(task["task_file"]) for task in remapped_tasks}
        old_task_files = {
            str(task["task_file"])
            for task in tasks
            if isinstance(task.get("task_file"), str)
        }
        stale_files = sorted(old_task_files - new_task_files)
        for stale_file in stale_files:
            stale_path = root / stale_file
            if stale_path.exists():
                stale_path.unlink()

        errors = validate_index(index_payload, root, check_files=True)
        if errors:
            raise TaskError(
                "Cannot renumber tasks because file validation failed:\n"
                + "\n".join(f"- {error}" for error in errors)
            )
        atomic_write_json(paths["index"], index_payload)
    print(f"Renumbered {len(remapped_tasks)} tasks.")
    return 0


def cmd_help(args: argparse.Namespace, parser: argparse.ArgumentParser) -> int:
    """Print parser help."""
    if args.command_name is None:
        parser.print_help()
        return 0
    command_name = args.command_name
    for action in parser._actions:
        if isinstance(action, argparse._SubParsersAction):
            command_parser = action.choices.get(command_name)
            if command_parser is None:
                raise TaskError(f"Unknown command '{command_name}'.")
            command_parser.print_help()
            return 0
    parser.print_help()
    return 0


def build_parser() -> argparse.ArgumentParser:
    """Build CLI parser."""
    parser = argparse.ArgumentParser(
        prog="task.py",
        description="Persistent filesystem task manager for Codex.",
    )
    parser.add_argument(
        "--root",
        default=str(DEFAULT_TASK_STORE_ROOT),
        help=(
            f"Task store root containing {TASKS_DIR_NAME}/ "
            f"(default: {DEFAULT_TASK_STORE_ROOT})."
        ),
    )

    subparsers = parser.add_subparsers(dest="command", required=True)

    init_parser = subparsers.add_parser("init", help=f"Initialize {TASKS_DIR_NAME}/ store.")
    init_parser.set_defaults(handler=cmd_init)

    add_parser = subparsers.add_parser("add", help="Add a new task.")
    add_parser.add_argument("--title", required=True, help="Task title.")
    source_group = add_parser.add_mutually_exclusive_group(required=True)
    source_group.add_argument(
        "--markdown-file",
        help="Path to markdown file containing task detail text.",
    )
    source_group.add_argument(
        "--markdown-stdin",
        action="store_true",
        help="Read markdown detail text from stdin.",
    )
    add_parser.add_argument(
        "--blocked-by",
        help="Comma-separated task IDs that block this task.",
    )
    add_parser.add_argument(
        "--status",
        default="todo",
        choices=VALID_STATUSES,
        help="Initial task status (default: todo).",
    )
    add_parser.set_defaults(handler=cmd_add)

    list_parser = subparsers.add_parser("list", help="List tasks.")
    list_parser.add_argument(
        "--status",
        choices=VALID_STATUSES,
        help="Filter tasks by status.",
    )
    list_parser.add_argument(
        "--ready",
        action="store_true",
        help="Show only ready tasks (status=todo and blockers done).",
    )
    list_parser.add_argument(
        "--all",
        action="store_true",
        help="Include done/canceled tasks.",
    )
    list_parser.add_argument(
        "--json",
        action="store_true",
        help="Output JSON.",
    )
    list_parser.set_defaults(handler=cmd_list)

    get_parser = subparsers.add_parser("get", help="Get a task by ID.")
    get_parser.add_argument("task_id", help="Task ID (for example T0001).")
    get_parser.add_argument(
        "--body",
        action="store_true",
        help="Include markdown task body.",
    )
    get_parser.add_argument(
        "--json",
        action="store_true",
        help="Output JSON.",
    )
    get_parser.set_defaults(handler=cmd_get)

    update_parser = subparsers.add_parser("update", help="Update a task.")
    update_parser.add_argument("task_id", help="Task ID (for example T0001).")
    update_parser.add_argument("--title", help="New title.")
    update_parser.add_argument(
        "--status",
        choices=VALID_STATUSES,
        help="New status.",
    )
    update_parser.add_argument(
        "--set-blocked-by",
        help="Replace blockers with comma-separated IDs (empty string clears blockers).",
    )
    update_parser.add_argument(
        "--add-blocker",
        action="append",
        default=[],
        help="Add blocker IDs (comma-separated allowed). Repeat to add more.",
    )
    update_parser.add_argument(
        "--remove-blocker",
        action="append",
        default=[],
        help="Remove blocker IDs (comma-separated allowed). Repeat to remove more.",
    )
    markdown_group = update_parser.add_mutually_exclusive_group()
    markdown_group.add_argument(
        "--replace-markdown-file",
        help="Replace task markdown body from file content.",
    )
    markdown_group.add_argument(
        "--append-markdown-file",
        help="Append markdown body from file content.",
    )
    update_parser.set_defaults(handler=cmd_update)

    done_parser = subparsers.add_parser("done", help="Mark task as done.")
    done_parser.add_argument("task_id", help="Task ID (for example T0001).")
    done_parser.add_argument(
        "--force",
        action="store_true",
        help="Allow marking done even when blockers are unresolved.",
    )
    done_parser.set_defaults(handler=cmd_done)

    start_parser = subparsers.add_parser(
        "start",
        help="Claim oldest ready task and mark it in_progress.",
    )
    start_output_group = start_parser.add_mutually_exclusive_group()
    start_output_group.add_argument(
        "--json",
        action="store_true",
        help="Output task object as JSON (or null when no ready tasks).",
    )
    start_output_group.add_argument(
        "--id-only",
        action="store_true",
        help="Print only task ID (prints nothing when no ready tasks).",
    )
    start_parser.add_argument(
        "--body",
        action="store_true",
        help="Include markdown body in output (JSON mode adds 'markdown').",
    )
    start_parser.add_argument(
        "--claimant",
        help="Claim owner label for lease metadata (default: current user/host/pid).",
    )
    start_parser.add_argument(
        "--lease-seconds",
        type=int,
        default=CLAIM_LEASE_SECONDS_DEFAULT,
        help=f"Lease duration in seconds (default: {CLAIM_LEASE_SECONDS_DEFAULT}).",
    )
    start_parser.set_defaults(handler=cmd_start)

    release_parser = subparsers.add_parser(
        "release",
        help="Move a task back to todo status.",
    )
    release_parser.add_argument("task_id", help="Task ID (for example T0001).")
    release_parser.add_argument(
        "--force",
        action="store_true",
        help="Allow releasing done/canceled tasks.",
    )
    release_parser.set_defaults(handler=cmd_release)

    finish_parser = subparsers.add_parser("finish", help="Alias for done.")
    finish_parser.add_argument("task_id", help="Task ID (for example T0001).")
    finish_parser.add_argument(
        "--force",
        action="store_true",
        help="Allow marking done even when blockers are unresolved.",
    )
    finish_parser.set_defaults(handler=cmd_finish)

    ready_parser = subparsers.add_parser(
        "ready", help="Show unblocked todo tasks sorted oldest-first."
    )
    ready_parser.add_argument(
        "--json",
        action="store_true",
        help="Output JSON.",
    )
    ready_parser.set_defaults(handler=cmd_ready)

    validate_parser = subparsers.add_parser(
        "validate", help="Validate index/task consistency."
    )
    validate_parser.set_defaults(handler=cmd_validate)

    renumber_parser = subparsers.add_parser(
        "renumber",
        help="Renumber tasks sequentially and rewrite blockers/file paths.",
    )
    renumber_parser.add_argument(
        "--by",
        choices=("topo", "id", "created"),
        default="topo",
        help="Ordering used for reassigned IDs (default: topo).",
    )
    renumber_parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Show proposed renumber mapping without writing changes.",
    )
    renumber_parser.add_argument(
        "--json",
        action="store_true",
        help="Output renumber mapping as JSON.",
    )
    renumber_parser.set_defaults(handler=cmd_renumber)

    help_parser = subparsers.add_parser("help", help="Show command help.")
    help_parser.add_argument("command_name", nargs="?", help="Specific command name.")
    help_parser.set_defaults(handler="help")

    return parser


def main() -> int:
    """Entry point."""
    parser = build_parser()
    args = parser.parse_args()
    try:
        if args.handler == "help":
            return cmd_help(args, parser)
        return args.handler(args)
    except TaskError as error:
        print(f"error: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
