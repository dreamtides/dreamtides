#!/usr/bin/env python3

"""Plans and validates scoped `just review` execution."""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path, PurePosixPath
from typing import Any, Callable, Mapping

DEFAULT_SCOPE_MODE = "dry-run"
DEFAULT_SCOPE_CONFIG_PATH = Path(__file__).with_name("review_scope_config.json")
DEFAULT_WORKSPACE_MANIFEST = "rules_engine/Cargo.toml"
DEFAULT_BASE_BRANCH = "origin/master"

CommandRunner = Callable[[list[str], Path], tuple[int, str, str]]


class ScopePlannerError(Exception):
    """Raised when scope planning cannot safely continue."""


@dataclass(frozen=True)
class ScopeConfig:
    """Static scope policy loaded from JSON configuration."""

    required_global_full_triggers: tuple[str, ...]
    global_full_triggers: tuple[str, ...]
    parser_crate_seeds: tuple[str, ...]
    parser_path_prefixes: tuple[str, ...]
    tv_crate_seeds: tuple[str, ...]
    tv_path_prefixes: tuple[str, ...]
    always_run_steps: tuple[str, ...]
    parser_steps: tuple[str, ...]
    tv_steps: tuple[str, ...]


@dataclass(frozen=True)
class WorkspaceMetadata:
    """Workspace crate roots and reverse dependency edges."""

    crate_roots: dict[str, str]
    reverse_dependencies: dict[str, set[str]]


@dataclass(frozen=True)
class ChangedFilesResult:
    """Changed file resolution result with provenance and optional error."""

    changed_files: list[str]
    source: str
    error: str = ""


@dataclass(frozen=True)
class ScopeDecision:
    """Planner output used by review runner and telemetry."""

    mode: str
    enforce: bool
    forced_full: bool
    forced_full_reason: str
    changed_files_source: str
    changed_files: list[str]
    impacted_crates: list[str]
    domains: list[str]
    selected_steps: list[str]
    skipped_steps: dict[str, str]
    unmapped_paths: list[str]

    def event_payload(self) -> dict[str, Any]:
        """Builds telemetry payload for `scope_plan` events."""
        return {
            "scope_mode": self.mode,
            "enforce": self.enforce,
            "forced_full": self.forced_full,
            "forced_full_reason": self.forced_full_reason,
            "changed_file_count": len(self.changed_files),
            "changed_files_source": self.changed_files_source,
            "impacted_crates": self.impacted_crates,
            "domains": self.domains,
            "selected_steps": self.selected_steps,
            "skipped_steps": self.skipped_steps,
            "unmapped_paths": self.unmapped_paths,
        }


def parse_args() -> argparse.Namespace:
    """Parses CLI arguments for scope planning and validation."""
    parser = argparse.ArgumentParser(description="Plan and validate scoped review execution")
    parser.add_argument("command", choices=["plan", "validate"], nargs="?", default="plan")
    parser.add_argument("--config-path", default=os.environ.get("REVIEW_SCOPE_CONFIG_PATH", str(DEFAULT_SCOPE_CONFIG_PATH)))
    return parser.parse_args()


def normalize_scope_mode(raw_mode: str | None) -> str:
    """Normalizes configured scope mode with a safe default."""
    mode = (raw_mode or DEFAULT_SCOPE_MODE).strip().lower()
    if mode in {"dry-run", "enforce", "off"}:
        return mode
    return DEFAULT_SCOPE_MODE


def is_truthy(value: str | None) -> bool:
    """Returns true for common truthy environment variable values."""
    if value is None:
        return False
    return value.strip().lower() in {"1", "true", "yes", "on"}


def normalize_repo_path(path: str) -> str:
    """Normalizes repository-relative paths into POSIX form."""
    value = path.strip().replace("\\", "/")
    while value.startswith("./"):
        value = value[2:]
    while value.startswith("/"):
        value = value[1:]
    if not value:
        return ""
    normalized = PurePosixPath(value).as_posix()
    return "" if normalized == "." else normalized


def normalize_rule_path(path: str) -> str:
    """Normalizes trigger/prefix rule entries while preserving directory intent."""
    stripped = path.strip()
    normalized = normalize_repo_path(stripped)
    if not normalized:
        return ""
    if stripped.endswith("/") and not normalized.endswith("/"):
        return f"{normalized}/"
    return normalized


def dedupe_keep_order(values: list[str]) -> list[str]:
    """Deduplicates while preserving input order."""
    seen: set[str] = set()
    deduped: list[str] = []
    for value in values:
        if value in seen:
            continue
        seen.add(value)
        deduped.append(value)
    return deduped


def run_command(args: list[str], cwd: Path) -> tuple[int, str, str]:
    """Runs a command and returns exit code, stdout, stderr."""
    completed = subprocess.run(args, cwd=cwd, capture_output=True, text=True)
    return (completed.returncode, completed.stdout, completed.stderr)


def parse_changed_files_override(raw_value: str) -> list[str]:
    """Parses changed file override from JSON array or newline-delimited text."""
    stripped = raw_value.strip()
    if not stripped:
        return []

    values: list[str] = []
    if stripped.startswith("["):
        try:
            payload = json.loads(stripped)
        except json.JSONDecodeError as exc:
            raise ScopePlannerError(f"invalid REVIEW_SCOPE_CHANGED_FILES JSON: {exc}") from exc
        if not isinstance(payload, list) or not all(isinstance(item, str) for item in payload):
            raise ScopePlannerError("REVIEW_SCOPE_CHANGED_FILES JSON must be an array of strings")
        values = [normalize_repo_path(item) for item in payload]
    else:
        values = [normalize_repo_path(line) for line in stripped.splitlines()]

    return dedupe_keep_order([value for value in values if value])


def run_git_lines(command_runner: CommandRunner, repo_root: Path, args: list[str]) -> list[str]:
    """Runs a git command and returns normalized path lines."""
    code, stdout, stderr = command_runner(args, repo_root)
    if code != 0:
        stderr_text = stderr.strip() or "unknown git error"
        raise ScopePlannerError(f"{' '.join(args)} failed: {stderr_text}")

    lines = [normalize_repo_path(line) for line in stdout.splitlines()]
    return dedupe_keep_order([line for line in lines if line])


def run_git_single(command_runner: CommandRunner, repo_root: Path, args: list[str]) -> str:
    """Runs a git command expected to return a single value."""
    code, stdout, stderr = command_runner(args, repo_root)
    if code != 0:
        stderr_text = stderr.strip() or "unknown git error"
        raise ScopePlannerError(f"{' '.join(args)} failed: {stderr_text}")

    value = stdout.strip().splitlines()[0].strip() if stdout.strip() else ""
    if not value:
        raise ScopePlannerError(f"{' '.join(args)} returned empty output")
    return value


def resolve_changed_files(env: Mapping[str, str], repo_root: Path, command_runner: CommandRunner = run_command) -> ChangedFilesResult:
    """Resolves changed files using deterministic source precedence."""
    override = env.get("REVIEW_SCOPE_CHANGED_FILES", "")
    if override.strip():
        changed_files = parse_changed_files_override(override)
        return ChangedFilesResult(changed_files=changed_files, source="env:REVIEW_SCOPE_CHANGED_FILES")

    base_ref = env.get("REVIEW_SCOPE_BASE_REF", "").strip()
    head_ref = env.get("REVIEW_SCOPE_HEAD_REF", "").strip()
    if base_ref and head_ref:
        changed_files = run_git_lines(command_runner, repo_root, ["git", "diff", "--name-only", f"{base_ref}...{head_ref}"])
        return ChangedFilesResult(changed_files=changed_files, source=f"git:{base_ref}...{head_ref}")

    if is_truthy(env.get("CI")):
        merge_base = run_git_single(command_runner, repo_root, ["git", "merge-base", DEFAULT_BASE_BRANCH, "HEAD"])
        changed_files = run_git_lines(command_runner, repo_root, ["git", "diff", "--name-only", f"{merge_base}...HEAD"])
        return ChangedFilesResult(changed_files=changed_files, source=f"ci-merge-base:{DEFAULT_BASE_BRANCH}")

    merge_base = run_git_single(command_runner, repo_root, ["git", "merge-base", DEFAULT_BASE_BRANCH, "HEAD"])
    branch_diff = run_git_lines(command_runner, repo_root, ["git", "diff", "--name-only", f"{merge_base}...HEAD"])
    staged_diff = run_git_lines(command_runner, repo_root, ["git", "diff", "--name-only", "--cached", "HEAD"])
    unstaged_diff = run_git_lines(command_runner, repo_root, ["git", "diff", "--name-only"])
    untracked = run_git_lines(command_runner, repo_root, ["git", "ls-files", "--others", "--exclude-standard"])

    union = dedupe_keep_order([*branch_diff, *staged_diff, *unstaged_diff, *untracked])
    return ChangedFilesResult(changed_files=union, source="local-union")


def load_scope_config(config_path: Path | None = None) -> ScopeConfig:
    """Loads and normalizes scope configuration JSON."""
    path = config_path or DEFAULT_SCOPE_CONFIG_PATH
    if not path.exists():
        raise ScopePlannerError(f"scope config not found: {path}")

    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:
        raise ScopePlannerError(f"failed to parse scope config {path}: {exc}") from exc

    if not isinstance(payload, dict):
        raise ScopePlannerError("scope config root must be an object")

    parser_section = payload.get("parser", {})
    tv_section = payload.get("tv", {})

    if not isinstance(parser_section, dict):
        raise ScopePlannerError("scope config 'parser' must be an object")
    if not isinstance(tv_section, dict):
        raise ScopePlannerError("scope config 'tv' must be an object")

    def read_rules(value: Any, field_name: str) -> tuple[str, ...]:
        if not isinstance(value, list) or not all(isinstance(item, str) for item in value):
            raise ScopePlannerError(f"scope config '{field_name}' must be an array of strings")
        normalized = [normalize_rule_path(item) for item in value]
        return tuple(item for item in normalized if item)

    def read_names(value: Any, field_name: str) -> tuple[str, ...]:
        if not isinstance(value, list) or not all(isinstance(item, str) for item in value):
            raise ScopePlannerError(f"scope config '{field_name}' must be an array of strings")
        normalized = [item.strip() for item in value]
        return tuple(item for item in normalized if item)

    return ScopeConfig(
        required_global_full_triggers=read_rules(payload.get("required_global_full_triggers", []), "required_global_full_triggers"),
        global_full_triggers=read_rules(payload.get("global_full_triggers", []), "global_full_triggers"),
        parser_crate_seeds=read_names(parser_section.get("crate_seeds", []), "parser.crate_seeds"),
        parser_path_prefixes=read_rules(parser_section.get("path_prefixes", []), "parser.path_prefixes"),
        tv_crate_seeds=read_names(tv_section.get("crate_seeds", []), "tv.crate_seeds"),
        tv_path_prefixes=read_rules(tv_section.get("path_prefixes", []), "tv.path_prefixes"),
        always_run_steps=read_names(payload.get("always_run_steps", []), "always_run_steps"),
        parser_steps=read_names(payload.get("parser_steps", []), "parser_steps"),
        tv_steps=read_names(payload.get("tv_steps", []), "tv_steps"),
    )


def load_workspace_metadata(repo_root: Path, command_runner: CommandRunner = run_command) -> WorkspaceMetadata:
    """Loads workspace crate roots and reverse dependency edges from Cargo metadata."""
    command = [
        "cargo",
        "metadata",
        "--manifest-path",
        DEFAULT_WORKSPACE_MANIFEST,
        "--format-version",
        "1",
        "--no-deps",
    ]
    code, stdout, stderr = command_runner(command, repo_root)
    if code != 0:
        stderr_text = stderr.strip() or "cargo metadata failed"
        raise ScopePlannerError(f"{' '.join(command)} failed: {stderr_text}")

    try:
        payload = json.loads(stdout)
    except json.JSONDecodeError as exc:
        raise ScopePlannerError(f"failed to parse cargo metadata output: {exc}") from exc

    packages = payload.get("packages", [])
    workspace_members = set(payload.get("workspace_members", []))

    if not isinstance(packages, list):
        raise ScopePlannerError("cargo metadata payload missing packages list")

    workspace_packages = [
        package for package in packages if not workspace_members or package.get("id") in workspace_members
    ]

    crate_roots: dict[str, str] = {}
    reverse_dependencies: dict[str, set[str]] = {}
    manifest_dir_to_name: dict[str, str] = {}

    repo_root_resolved = repo_root.resolve()

    for package in workspace_packages:
        if not isinstance(package, dict):
            continue
        name = package.get("name")
        manifest_path = package.get("manifest_path")
        if not isinstance(name, str) or not name:
            continue
        if not isinstance(manifest_path, str) or not manifest_path:
            continue

        manifest_dir = Path(manifest_path).resolve().parent
        try:
            relative_root = manifest_dir.relative_to(repo_root_resolved)
        except ValueError as exc:
            raise ScopePlannerError(f"workspace crate root is outside repository: {manifest_dir}") from exc

        normalized_root = normalize_repo_path(relative_root.as_posix())
        crate_roots[name] = normalized_root
        reverse_dependencies.setdefault(name, set())
        manifest_dir_to_name[str(manifest_dir)] = name

    for package in workspace_packages:
        if not isinstance(package, dict):
            continue
        name = package.get("name")
        if not isinstance(name, str) or name not in crate_roots:
            continue

        for dependency in package.get("dependencies", []):
            if not isinstance(dependency, dict):
                continue
            dependency_path = dependency.get("path")
            if not isinstance(dependency_path, str) or not dependency_path:
                continue

            dependency_name = manifest_dir_to_name.get(str(Path(dependency_path).resolve()))
            if dependency_name is None:
                continue
            reverse_dependencies.setdefault(dependency_name, set()).add(name)

    return WorkspaceMetadata(crate_roots=crate_roots, reverse_dependencies=reverse_dependencies)


def path_matches_rule(path: str, rule: str) -> bool:
    """Returns whether a changed path matches a configured exact rule or prefix rule."""
    if not rule:
        return False
    if rule.endswith("/"):
        directory = rule[:-1]
        return path == directory or path.startswith(rule)
    return path == rule


def first_matching_rule(path: str, rules: tuple[str, ...]) -> str:
    """Returns the first matching rule for a path, if any."""
    for rule in rules:
        if path_matches_rule(path, rule):
            return rule
    return ""


def crates_for_path(path: str, crate_roots: dict[str, str]) -> list[str]:
    """Returns workspace crates whose roots contain the path."""
    matched = [
        crate_name
        for crate_name, root in crate_roots.items()
        if root and (path == root or path.startswith(f"{root}/"))
    ]
    matched.sort()
    return matched


def expand_impacted_crates(direct_crates: set[str], reverse_dependencies: dict[str, set[str]]) -> list[str]:
    """Computes reverse dependency closure for directly touched crates."""
    impacted = set(direct_crates)
    stack = list(direct_crates)

    while stack:
        crate = stack.pop()
        for dependent in reverse_dependencies.get(crate, set()):
            if dependent in impacted:
                continue
            impacted.add(dependent)
            stack.append(dependent)

    return sorted(impacted)


def select_steps(
    step_names: list[str],
    config: ScopeConfig,
    forced_full: bool,
    parser_impacted: bool,
    tv_impacted: bool,
) -> tuple[list[str], dict[str, str]]:
    """Selects planned steps and per-step skip reasons."""
    if forced_full:
        return (list(step_names), {})

    always_steps = set(config.always_run_steps)
    parser_steps = set(config.parser_steps)
    tv_steps = set(config.tv_steps)

    selected: list[str] = []
    skipped: dict[str, str] = {}

    for step_name in step_names:
        if step_name in parser_steps:
            if parser_impacted:
                selected.append(step_name)
            else:
                skipped[step_name] = "parser domain not impacted"
            continue

        if step_name in tv_steps:
            if tv_impacted:
                selected.append(step_name)
            else:
                skipped[step_name] = "tv domain not impacted"
            continue

        if step_name in always_steps:
            selected.append(step_name)
            continue

        selected.append(step_name)

    return (selected, skipped)


def plan_review_scope(
    step_names: list[str],
    env: Mapping[str, str] | None = None,
    repo_root: Path | None = None,
    config: ScopeConfig | None = None,
    metadata: WorkspaceMetadata | None = None,
    command_runner: CommandRunner = run_command,
) -> ScopeDecision:
    """Plans scoped step execution and returns a deterministic decision."""
    effective_env = dict(os.environ if env is None else env)
    effective_repo_root = (repo_root or Path.cwd()).resolve()
    mode = normalize_scope_mode(effective_env.get("REVIEW_SCOPE_MODE"))

    if mode == "off":
        return ScopeDecision(
            mode=mode,
            enforce=False,
            forced_full=False,
            forced_full_reason="",
            changed_files_source="scope-mode-off",
            changed_files=[],
            impacted_crates=[],
            domains=["core"],
            selected_steps=list(step_names),
            skipped_steps={},
            unmapped_paths=[],
        )

    scope_config = config or load_scope_config(Path(effective_env.get("REVIEW_SCOPE_CONFIG_PATH", str(DEFAULT_SCOPE_CONFIG_PATH))))
    changed_files_result = resolve_changed_files(effective_env, effective_repo_root, command_runner)

    force_full_by_env = is_truthy(effective_env.get("REVIEW_SCOPE_FORCE_FULL"))
    forced_full_reason = "forced by REVIEW_SCOPE_FORCE_FULL" if force_full_by_env else ""

    if changed_files_result.error and not forced_full_reason:
        forced_full_reason = changed_files_result.error

    changed_files = changed_files_result.changed_files
    if not changed_files and not forced_full_reason:
        forced_full_reason = "no changed files detected"

    impacted_crates: list[str] = []
    parser_impacted = False
    tv_impacted = False
    unmapped_paths: list[str] = []

    if not forced_full_reason and changed_files:
        workspace_metadata = metadata or load_workspace_metadata(effective_repo_root, command_runner)

        direct_crates: set[str] = set()
        parser_path_impact = False
        tv_path_impact = False

        for changed_path in changed_files:
            full_trigger_match = first_matching_rule(changed_path, scope_config.global_full_triggers)
            parser_rule_match = first_matching_rule(changed_path, scope_config.parser_path_prefixes)
            tv_rule_match = first_matching_rule(changed_path, scope_config.tv_path_prefixes)
            mapped_crates = crates_for_path(changed_path, workspace_metadata.crate_roots)

            if full_trigger_match and not forced_full_reason:
                forced_full_reason = f"matched global full trigger '{full_trigger_match}'"

            if parser_rule_match:
                parser_path_impact = True

            if tv_rule_match:
                tv_path_impact = True

            if mapped_crates:
                direct_crates.update(mapped_crates)

            if not full_trigger_match and not parser_rule_match and not tv_rule_match and not mapped_crates:
                unmapped_paths.append(changed_path)

        if unmapped_paths and not forced_full_reason:
            forced_full_reason = f"unmapped changed path '{unmapped_paths[0]}'"

        impacted_crates = expand_impacted_crates(direct_crates, workspace_metadata.reverse_dependencies)
        parser_impacted = parser_path_impact or bool(set(impacted_crates) & set(scope_config.parser_crate_seeds))
        tv_impacted = tv_path_impact or bool(set(impacted_crates) & set(scope_config.tv_crate_seeds))

    forced_full = bool(forced_full_reason)

    selected_steps, skipped_steps = select_steps(
        step_names,
        scope_config,
        forced_full=forced_full,
        parser_impacted=parser_impacted,
        tv_impacted=tv_impacted,
    )

    domains = ["core"]
    if parser_impacted:
        domains.append("parser")
    if tv_impacted:
        domains.append("tv")
    if forced_full:
        domains.append("full")

    return ScopeDecision(
        mode=mode,
        enforce=mode == "enforce",
        forced_full=forced_full,
        forced_full_reason=forced_full_reason,
        changed_files_source=changed_files_result.source,
        changed_files=changed_files,
        impacted_crates=impacted_crates,
        domains=domains,
        selected_steps=selected_steps,
        skipped_steps=skipped_steps,
        unmapped_paths=unmapped_paths,
    )


def fallback_full_scope_decision(step_names: list[str], mode: str, reason: str) -> ScopeDecision:
    """Builds a fail-closed full-run scope decision used on planner errors."""
    return ScopeDecision(
        mode=mode,
        enforce=mode == "enforce",
        forced_full=True,
        forced_full_reason=reason,
        changed_files_source="planner-error",
        changed_files=[],
        impacted_crates=[],
        domains=["core", "full"],
        selected_steps=list(step_names),
        skipped_steps={},
        unmapped_paths=[],
    )


def find_duplicates(values: tuple[str, ...]) -> list[str]:
    """Returns sorted duplicate entries from a tuple."""
    seen: set[str] = set()
    duplicates: set[str] = set()
    for value in values:
        if value in seen:
            duplicates.add(value)
            continue
        seen.add(value)
    return sorted(duplicates)


def validate_scope_configuration(config: ScopeConfig, metadata: WorkspaceMetadata) -> list[str]:
    """Validates scope config coverage and rule consistency."""
    errors: list[str] = []

    required = set(config.required_global_full_triggers)
    configured = set(config.global_full_triggers)
    missing_required = sorted(required - configured)
    if missing_required:
        errors.append(f"missing required global full triggers: {', '.join(missing_required)}")

    for field_name, values in (
        ("global_full_triggers", config.global_full_triggers),
        ("parser.path_prefixes", config.parser_path_prefixes),
        ("tv.path_prefixes", config.tv_path_prefixes),
        ("always_run_steps", config.always_run_steps),
        ("parser_steps", config.parser_steps),
        ("tv_steps", config.tv_steps),
    ):
        duplicates = find_duplicates(values)
        if duplicates:
            errors.append(f"{field_name} contains duplicate entries: {', '.join(duplicates)}")

    parser_rules = set(config.parser_path_prefixes)
    tv_rules = set(config.tv_path_prefixes)
    full_rules = set(config.global_full_triggers)

    parser_tv_overlap = sorted(parser_rules & tv_rules)
    if parser_tv_overlap:
        errors.append(f"parser and tv path rules overlap exactly: {', '.join(parser_tv_overlap)}")

    parser_full_overlap = sorted(parser_rules & full_rules)
    if parser_full_overlap:
        errors.append(f"parser path rules conflict with full-run rules: {', '.join(parser_full_overlap)}")

    tv_full_overlap = sorted(tv_rules & full_rules)
    if tv_full_overlap:
        errors.append(f"tv path rules conflict with full-run rules: {', '.join(tv_full_overlap)}")

    parser_seed_set = set(config.parser_crate_seeds)
    tv_seed_set = set(config.tv_crate_seeds)

    overlapping_seeds = sorted(parser_seed_set & tv_seed_set)
    if overlapping_seeds:
        errors.append(f"crate seeds appear in multiple domains: {', '.join(overlapping_seeds)}")

    parser_step_set = set(config.parser_steps)
    tv_step_set = set(config.tv_steps)
    always_step_set = set(config.always_run_steps)

    if parser_step_set & tv_step_set:
        overlap = sorted(parser_step_set & tv_step_set)
        errors.append(f"parser and tv step sets overlap: {', '.join(overlap)}")

    if always_step_set & parser_step_set:
        overlap = sorted(always_step_set & parser_step_set)
        errors.append(f"always-run and parser step sets overlap: {', '.join(overlap)}")

    if always_step_set & tv_step_set:
        overlap = sorted(always_step_set & tv_step_set)
        errors.append(f"always-run and tv step sets overlap: {', '.join(overlap)}")

    for crate_name in sorted(metadata.crate_roots):
        domains = []
        if crate_name in parser_seed_set:
            domains.append("parser")
        if crate_name in tv_seed_set:
            domains.append("tv")
        if not domains:
            domains.append("core")
        if len(domains) != 1:
            errors.append(f"crate '{crate_name}' is classifiable into multiple domains: {', '.join(domains)}")

    return errors


def default_step_names(config: ScopeConfig) -> list[str]:
    """Builds default step order for CLI planning output."""
    return dedupe_keep_order([*config.always_run_steps, *config.parser_steps, *config.tv_steps])


def render_scope_plan(decision: ScopeDecision) -> str:
    """Renders a concise human-auditable planner summary."""
    lines = [
        f"scope mode: {decision.mode}",
        f"enforce: {'yes' if decision.enforce else 'no'}",
        f"forced full: {'yes' if decision.forced_full else 'no'}",
    ]

    if decision.forced_full and decision.forced_full_reason:
        lines.append(f"forced full reason: {decision.forced_full_reason}")

    lines.append(f"changed files source: {decision.changed_files_source}")
    lines.append(f"changed files: {len(decision.changed_files)}")
    lines.append(f"domains: {', '.join(decision.domains)}")

    if decision.impacted_crates:
        lines.append(f"impacted crates: {', '.join(decision.impacted_crates)}")

    if decision.skipped_steps:
        lines.append("skipped steps:")
        for step_name in sorted(decision.skipped_steps):
            lines.append(f"  - {step_name}: {decision.skipped_steps[step_name]}")

    lines.append(f"selected steps: {', '.join(decision.selected_steps)}")

    if decision.unmapped_paths:
        lines.append("unmapped paths:")
        for path in decision.unmapped_paths:
            lines.append(f"  - {path}")

    return "\n".join(lines)


def main() -> int:
    """Entrypoint for scope planner utility."""
    args = parse_args()
    config_path = Path(args.config_path)
    if not config_path.is_absolute():
        config_path = (Path.cwd() / config_path).resolve()

    try:
        config = load_scope_config(config_path)
    except Exception as exc:
        print(f"scope config error: {exc}", file=sys.stderr)
        return 1

    if args.command == "validate":
        try:
            metadata = load_workspace_metadata(Path.cwd().resolve())
            errors = validate_scope_configuration(config, metadata)
        except Exception as exc:
            print(f"scope validation failed: {exc}", file=sys.stderr)
            return 1

        if errors:
            print("scope validation failed:", file=sys.stderr)
            for error in errors:
                print(f"- {error}", file=sys.stderr)
            return 1

        print("scope validation passed")
        return 0

    try:
        decision = plan_review_scope(
            step_names=default_step_names(config),
            env=os.environ,
            repo_root=Path.cwd().resolve(),
            config=config,
        )
    except Exception as exc:
        print(f"scope planning failed: {exc}", file=sys.stderr)
        return 1

    print(render_scope_plan(decision))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
