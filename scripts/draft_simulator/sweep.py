"""Sweep runner for batch parameter experiments.

Executes drafts across a Cartesian product of parameter values, computes
per-draft metrics, and aggregates results. Supports sequential and hashed
seeding policies for deterministic reproducibility. Stdlib-only, no
external dependencies.
"""

import hashlib
import itertools
import math
import sys
from dataclasses import dataclass, field
from typing import Any

import config
import draft_runner
import metrics
import output


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
    """Compute a deterministic SHA-256 hash of all parameter values.

    Sorts parameter keys alphabetically, serializes as "key=value"
    pairs joined by "|", and returns the hex digest.
    """
    pairs: list[str] = []
    sections = [
        ("draft", cfg.draft),
        ("cube", cfg.cube),
        ("pack_generation", cfg.pack_generation),
        ("refill", cfg.refill),
        ("cards", cfg.cards),
        ("agents", cfg.agents),
        ("scoring", cfg.scoring),
        ("commitment", cfg.commitment),
        ("metrics", cfg.metrics),
    ]
    for section_name, section_obj in sections:
        for f in sorted(
            config._SECTION_CLASSES[section_name].__dataclass_fields__.keys()
        ):
            value = getattr(section_obj, f)
            pairs.append(f"{section_name}.{f}={value}")

    pairs.sort()
    data = "|".join(pairs)
    return hashlib.sha256(data.encode()).hexdigest()


def compute_seed(
    base_seed: int,
    run_index: int,
    config_hash: str,
    seeding_policy: str,
) -> int:
    """Derive a per-run seed from the base seed and run index.

    Sequential: seed = base_seed + run_index.
    Hashed: seed = hash((base_seed, config_hash, run_index)) % 2**32.
    """
    if seeding_policy == "hashed":
        h = hash((base_seed, config_hash, run_index))
        return h % (2**32)
    return base_seed + run_index


def build_sweep_points(cfg: config.SimulatorConfig) -> list[SweepPoint]:
    """Build the Cartesian product of all sweep axes.

    If no axes are defined, returns a single point with no overrides.
    """
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
    """Create a new config by applying sweep overrides to a base config.

    Overrides use dot notation (e.g., "draft.seat_count").
    Returns a modified copy of the config.
    """
    cfg = _shallow_copy_config(base_cfg)

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
) -> tuple[list[dict[str, Any]], list[dict[str, Any]]]:
    """Execute a full parameter sweep experiment.

    Computes the Cartesian product of sweep axes, runs drafts for each
    combination, computes metrics, and aggregates results. Prints progress
    during execution.

    Returns (run_records, aggregate_records) for output serialization.
    """
    points = build_sweep_points(cfg)
    num_combinations = len(points)
    total_runs = num_combinations * runs_per_point

    print(
        f"Sweep: {len(cfg.sweep.axes)} axes, "
        f"{num_combinations} combinations, "
        f"{runs_per_point} runs each"
    )

    all_run_records: list[dict[str, Any]] = []
    all_aggregate_records: list[dict[str, Any]] = []
    completed = 0

    for point in points:
        point_cfg = apply_overrides(cfg, point.overrides)
        cfg_hash = compute_config_hash(point_cfg)

        point_run_records: list[dict[str, Any]] = []
        point_metric_values: dict[str, list[float]] = {}

        for run_i in range(runs_per_point):
            seed = compute_seed(base_seed, run_i, cfg_hash, cfg.sweep.seeding_policy)

            result = draft_runner.run_draft(point_cfg, seed)
            draft_metrics = metrics.compute_metrics(result, point_cfg)

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

        agg_record = _build_aggregate_record(
            point, point_cfg, cfg_hash, runs_per_point, point_metric_values
        )
        all_aggregate_records.append(agg_record)

    print()
    return all_run_records, all_aggregate_records


def _accumulate_metric_values(
    accumulator: dict[str, list[float]],
    run_record: dict[str, Any],
) -> None:
    """Extract numeric metric values from a run record into an accumulator."""
    metric_keys = [
        "cr_shown_near_opt_overall",
        "cr_shown_score_gap_overall",
        "cr_shown_entropy_overall",
        "cr_full_near_opt_overall",
        "conv_shown_late_mean",
        "conv_shown_late_p3",
        "splash_shown",
        "splash_full",
        "openness_shown_archetypes",
        "openness_shown_entropy",
    ]
    for key in metric_keys:
        val = run_record.get(key, "")
        if val != "" and val is not None:
            if key not in accumulator:
                accumulator[key] = []
            accumulator[key].append(float(val))

    # Also accumulate per-seat deck values
    for col_key, col_val in run_record.items():
        if col_key.endswith("_deck_value") and col_val != "" and col_val is not None:
            if col_key not in accumulator:
                accumulator[col_key] = []
            accumulator[col_key].append(float(col_val))

    # Commitment picks for validation
    for col_key, col_val in run_record.items():
        if (
            col_key.endswith("_commitment_pick")
            and col_val != ""
            and col_val is not None
        ):
            if col_key not in accumulator:
                accumulator[col_key] = []
            accumulator[col_key].append(float(col_val))


def _build_aggregate_record(
    point: SweepPoint,
    point_cfg: config.SimulatorConfig,
    cfg_hash: str,
    num_runs: int,
    metric_values: dict[str, list[float]],
) -> dict[str, Any]:
    """Build an aggregate record for one parameter combination."""
    record: dict[str, Any] = {}

    # Swept parameter values
    for key, val in sorted(point.overrides.items()):
        record[f"swept_{key}"] = val

    record["config_hash"] = cfg_hash
    record["num_runs"] = num_runs

    # Per-metric statistics
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
    """Mean of a list of floats, or 0.0 if empty."""
    if not values:
        return 0.0
    return sum(values) / len(values)


def _std(values: list[float]) -> float:
    """Population standard deviation of a list of floats."""
    if len(values) < 2:
        return 0.0
    m = _mean(values)
    variance = sum((v - m) ** 2 for v in values) / len(values)
    return math.sqrt(variance)


def _percentile(values: list[float], p: float) -> float:
    """Compute the p-th percentile (0-100) of a list of floats."""
    if not values:
        return 0.0
    s = sorted(values)
    k = (p / 100.0) * (len(s) - 1)
    f = math.floor(k)
    c = math.ceil(k)
    if f == c:
        return s[int(k)]
    return s[f] * (c - k) + s[c] * (k - f)


def _shallow_copy_config(
    cfg: config.SimulatorConfig,
) -> config.SimulatorConfig:
    """Create a shallow copy of SimulatorConfig with copied sub-sections."""
    new_cfg = config.SimulatorConfig()
    new_cfg.draft = config.DraftConfig(
        seat_count=cfg.draft.seat_count,
        round_count=cfg.draft.round_count,
        picks_per_round=list(cfg.draft.picks_per_round),
        pack_size=cfg.draft.pack_size,
        alternate_direction=cfg.draft.alternate_direction,
        human_seats=cfg.draft.human_seats,
    )
    new_cfg.cube = config.CubeConfig(
        distinct_cards=cfg.cube.distinct_cards,
        copies_per_card=cfg.cube.copies_per_card,
        consumption_mode=cfg.cube.consumption_mode,
    )
    new_cfg.pack_generation = config.PackGenerationConfig(
        strategy=cfg.pack_generation.strategy,
        archetype_target_count=cfg.pack_generation.archetype_target_count,
        primary_density=cfg.pack_generation.primary_density,
        bridge_density=cfg.pack_generation.bridge_density,
        variance=cfg.pack_generation.variance,
    )
    new_cfg.refill = config.RefillConfig(
        strategy=cfg.refill.strategy,
        fingerprint_source=cfg.refill.fingerprint_source,
        fidelity=cfg.refill.fidelity,
        commit_bias=cfg.refill.commit_bias,
    )
    new_cfg.cards = config.CardsConfig(
        source=cfg.cards.source,
        file_path=cfg.cards.file_path,
        archetype_count=cfg.cards.archetype_count,
        cards_per_archetype=cfg.cards.cards_per_archetype,
        bridge_fraction=cfg.cards.bridge_fraction,
    )
    new_cfg.agents = config.AgentsConfig(
        policy=cfg.agents.policy,
        show_n=cfg.agents.show_n,
        show_n_strategy=cfg.agents.show_n_strategy,
        ai_optimality=cfg.agents.ai_optimality,
        ai_signal_weight=cfg.agents.ai_signal_weight,
        openness_window=cfg.agents.openness_window,
        learning_rate=cfg.agents.learning_rate,
        force_archetype=cfg.agents.force_archetype,
    )
    new_cfg.scoring = config.ScoringConfig(
        weight_power=cfg.scoring.weight_power,
        weight_coherence=cfg.scoring.weight_coherence,
        weight_focus=cfg.scoring.weight_focus,
        secondary_weight=cfg.scoring.secondary_weight,
        focus_threshold=cfg.scoring.focus_threshold,
        focus_saturation=cfg.scoring.focus_saturation,
    )
    new_cfg.commitment = config.CommitmentConfig(
        commitment_threshold=cfg.commitment.commitment_threshold,
        stability_window=cfg.commitment.stability_window,
        entropy_threshold=cfg.commitment.entropy_threshold,
    )
    new_cfg.metrics = config.MetricsConfig(
        richness_gap=cfg.metrics.richness_gap,
        tau=cfg.metrics.tau,
        on_plan_threshold=cfg.metrics.on_plan_threshold,
        splash_power_threshold=cfg.metrics.splash_power_threshold,
        splash_flex_threshold=cfg.metrics.splash_flex_threshold,
        exposure_threshold=cfg.metrics.exposure_threshold,
    )
    new_cfg.sweep = config.SweepConfig(
        runs_per_point=cfg.sweep.runs_per_point,
        base_seed=cfg.sweep.base_seed,
        seeding_policy=cfg.sweep.seeding_policy,
        trace_enabled=cfg.sweep.trace_enabled,
        axes=dict(cfg.sweep.axes),
    )
    return new_cfg
