"""Sweep runner for batch parameter experiments (v2).

v2 changes: removed refill axes. Stdlib-only, no external dependencies.
"""

import hashlib
import itertools
import math
import sys
from dataclasses import dataclass, field
from typing import Any

import colors
import config
import draft_runner
import metrics
import output
import validation


@dataclass(frozen=True)
class SweepPoint:
    """One combination of swept parameter values."""

    overrides: dict[str, Any]
    label: str


@dataclass(frozen=True)
class AggregateRecord:
    """Aggregated statistics for one parameter combination."""

    overrides: dict[str, Any]
    config_hash: str
    num_runs: int
    metric_stats: dict[str, dict[str, float]]
    validation_flags: dict[str, bool]


def compute_config_hash(cfg: config.SimulatorConfig) -> str:
    """Compute a deterministic SHA-256 hash of all parameter values."""
    pairs = config.config_to_sorted_pairs(cfg, exclude_sections={"sweep"})
    data = "|".join(pairs)
    return hashlib.sha256(data.encode()).hexdigest()


def compute_seed(
    base_seed: int,
    run_index: int,
    config_hash: str,
    seeding_policy: str,
) -> int:
    """Derive a per-run seed from the base seed and run index."""
    if seeding_policy == "hashed":
        payload = f"{base_seed}|{config_hash}|{run_index}"
        h = int(hashlib.sha256(payload.encode()).hexdigest(), 16)
        return h % (2**32)
    return base_seed + run_index


def build_sweep_points(cfg: config.SimulatorConfig) -> list[SweepPoint]:
    """Build the Cartesian product of all sweep axes."""
    axes = cfg.sweep.axes
    if not axes:
        return [SweepPoint(overrides={}, label="default")]

    keys = sorted(axes.keys())
    value_lists = [axes[k] for k in keys]
    points: list[SweepPoint] = []

    for combo in itertools.product(*value_lists):
        overrides: dict[str, Any] = {}
        label_parts: list[str] = []
        for key, val in zip(keys, combo):
            overrides[key] = val
            label_parts.append(f"{key}={val}")
        points.append(SweepPoint(overrides=overrides, label=", ".join(label_parts)))

    return points


def apply_overrides(
    base_cfg: config.SimulatorConfig,
    overrides: dict[str, Any],
) -> config.SimulatorConfig:
    """Create a new config by applying sweep overrides to a base config."""
    cfg = config.clone_config(base_cfg)

    for key, value in overrides.items():
        parts = key.split(".")
        if len(parts) != 2:
            raise ValueError(f"Override key must be section.param (got {key!r})")
        section_name, param_name = parts
        if not hasattr(cfg, section_name):
            raise ValueError(f"Unknown config section: {section_name!r}")
        section = getattr(cfg, section_name)
        if not hasattr(section, param_name):
            raise ValueError(
                f"Unknown parameter {param_name!r} in section {section_name!r}"
            )
        setattr(section, param_name, value)

    return cfg


def run_sweep(
    cfg: config.SimulatorConfig,
    base_seed: int,
    runs_per_point: int,
    output_dir: str,
    skip_comparisons: bool = False,
) -> tuple[list[dict[str, Any]], list[dict[str, Any]]]:
    """Execute a full parameter sweep experiment."""
    points = build_sweep_points(cfg)
    num_combinations = len(points)
    total_runs = num_combinations * runs_per_point

    print(
        f"{colors.section('Sweep:')} {colors.num(len(cfg.sweep.axes))} axes, "
        f"{colors.num(num_combinations)} combinations, "
        f"{colors.num(runs_per_point)} runs each"
    )

    all_run_records: list[dict[str, Any]] = []
    all_aggregate_records: list[dict[str, Any]] = []
    completed = 0

    for point in points:
        point_cfg = apply_overrides(cfg, point.overrides)
        cfg_hash = compute_config_hash(point_cfg)

        point_run_records: list[dict[str, Any]] = []
        point_metric_values: dict[str, list[float]] = {}
        adaptive_deck_values: list[float] = []
        seeds_used: list[int] = []

        for run_i in range(runs_per_point):
            seed = compute_seed(base_seed, run_i, cfg_hash, cfg.sweep.seeding_policy)
            seeds_used.append(seed)

            result = draft_runner.run_draft(point_cfg, seed)

            for sr in result.seat_results:
                adaptive_deck_values.append(sr.deck_value)

            if skip_comparisons:
                force_dvs = None
                ignorant_dvs = None
                aware_dvs = None
            else:
                force_dvs, ignorant_dvs, aware_dvs = _run_comparison_drafts(
                    point_cfg, seed
                )

            draft_metrics = metrics.compute_metrics(
                result,
                point_cfg,
                force_deck_values=force_dvs,
                adaptive_deck_values=adaptive_deck_values[-len(result.seat_results) :],
                aware_deck_values=aware_dvs,
                ignorant_deck_values=ignorant_dvs,
            )

            run_record = output.build_run_record(
                run_id=completed,
                config_hash=cfg_hash,
                seed=seed,
                result=result,
                draft_metrics=draft_metrics,
                cfg=point_cfg,
            )
            point_run_records.append(run_record)
            all_run_records.append(run_record)

            _accumulate_metric_values(point_metric_values, run_record)

            completed += 1
            _print_progress(completed, total_runs)

        point_report = validation.run_validation([], point_run_records)
        val_flags: dict[str, bool] = {}
        for check in point_report.checks:
            val_flags[check.name] = check.passed

        agg_record = _build_aggregate_record(
            point,
            point_cfg,
            cfg_hash,
            runs_per_point,
            point_metric_values,
            validation_flags=val_flags,
        )
        all_aggregate_records.append(agg_record)

    print()
    return all_run_records, all_aggregate_records


def _run_comparison_drafts(
    point_cfg: config.SimulatorConfig,
    seed: int,
) -> tuple[
    dict[int, list[float]] | None,
    list[float] | None,
    list[float] | None,
]:
    """Run comparison drafts for forceability and signal benefit."""
    archetype_count = point_cfg.cards.archetype_count

    ignorant_result = draft_runner.run_draft(
        point_cfg, seed, human_seat_policy="signal_ignorant"
    )
    ignorant_dvs = [ignorant_result.seat_results[0].deck_value]

    aware_result = draft_runner.run_draft(point_cfg, seed)
    aware_dvs = [aware_result.seat_results[0].deck_value]

    force_dvs: dict[int, list[float]] = {}
    for arch in range(archetype_count):
        force_cfg = config.clone_config(point_cfg)
        force_cfg.agents.policy = "force"
        force_cfg.agents.force_archetype = arch
        force_result = draft_runner.run_draft(force_cfg, seed)
        force_dvs[arch] = [sr.deck_value for sr in force_result.seat_results]

    return force_dvs, ignorant_dvs, aware_dvs


def _accumulate_metric_values(
    accumulator: dict[str, list[float]],
    run_record: dict[str, Any],
) -> None:
    """Extract numeric metric values from a run record into an accumulator."""
    skip_keys = {"run_id", "config_hash", "seed"}
    for key, val in run_record.items():
        if key in skip_keys:
            continue
        if val == "" or val is None:
            continue
        try:
            float_val = float(val)
        except (ValueError, TypeError):
            continue
        if key not in accumulator:
            accumulator[key] = []
        accumulator[key].append(float_val)


def _build_aggregate_record(
    point: SweepPoint,
    point_cfg: config.SimulatorConfig,
    cfg_hash: str,
    num_runs: int,
    metric_values: dict[str, list[float]],
    validation_flags: dict[str, bool] | None = None,
) -> dict[str, Any]:
    """Build an aggregate record for one parameter combination."""
    record: dict[str, Any] = {}

    swept_keys = set(point.overrides.keys())
    for pair in config.config_to_sorted_pairs(point_cfg, exclude_sections={"sweep"}):
        eq_idx = pair.index("=")
        param_key = pair[:eq_idx]
        param_val = pair[eq_idx + 1 :]
        if param_key not in swept_keys:
            record[f"fixed_{param_key}"] = param_val

    for key, val in sorted(point.overrides.items()):
        record[f"swept_{key}"] = val

    record["config_hash"] = cfg_hash
    record["num_runs"] = num_runs

    for metric_key, values in sorted(metric_values.items()):
        if not values:
            continue
        record[f"{metric_key}_mean"] = round(_mean(values), 6)
        record[f"{metric_key}_std"] = round(_std(values), 6)
        record[f"{metric_key}_p5"] = round(_percentile(values, 5), 6)
        record[f"{metric_key}_p25"] = round(_percentile(values, 25), 6)
        record[f"{metric_key}_p50"] = round(_percentile(values, 50), 6)
        record[f"{metric_key}_p75"] = round(_percentile(values, 75), 6)
        record[f"{metric_key}_p95"] = round(_percentile(values, 95), 6)

    if validation_flags:
        for check_name, passed in sorted(validation_flags.items()):
            record[f"validation_{check_name}"] = "PASS" if passed else "FAIL"

    return record


def _print_progress(completed: int, total: int) -> None:
    """Print a progress bar to stderr."""
    bar_width = 28
    fraction = completed / total if total > 0 else 1.0
    filled = int(bar_width * fraction)
    bar = "=" * filled + " " * (bar_width - filled)
    sys.stdout.write(f"\r[{bar}] {completed}/{total} runs complete")
    sys.stdout.flush()


def _mean(values: list[float]) -> float:
    if not values:
        return 0.0
    return sum(values) / len(values)


def _std(values: list[float]) -> float:
    if len(values) < 2:
        return 0.0
    m = _mean(values)
    variance = sum((v - m) ** 2 for v in values) / len(values)
    return math.sqrt(variance)


def _percentile(values: list[float], p: float) -> float:
    if not values:
        return 0.0
    s = sorted(values)
    k = (p / 100.0) * (len(s) - 1)
    f = math.floor(k)
    c = math.ceil(k)
    if f == c:
        return s[int(k)]
    return s[f] * (c - k) + s[c] * (k - f)
