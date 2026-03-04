"""Output serialization for the draft simulator.

Writes run-level CSV, aggregate CSV, per-pick trace JSON, and config
metadata JSON to a specified output directory. All output is deterministic
given identical inputs. Stdlib-only, no external dependencies.
"""

import csv
import json
import os
from dataclasses import fields
from typing import Any

import config
import draft_runner
import metrics

VERSION = "0.1.0"


def ensure_output_dir(output_dir: str) -> None:
    """Create the output directory if it does not exist."""
    os.makedirs(output_dir, exist_ok=True)


def write_run_level_csv(
    output_dir: str,
    run_records: list[dict[str, Any]],
) -> str:
    """Write one row per simulation run to a CSV file.

    Each record contains run_id, config_hash, seed, all per-draft metric
    values, and per-seat data (commitment_pick, deck_value, final_archetype).
    Returns the path to the written file.
    """
    path = os.path.join(output_dir, "run_level.csv")
    if not run_records:
        with open(path, "w", newline="") as f:
            f.write("")
        return path

    fieldnames = _collect_all_fieldnames(run_records)
    with open(path, "w", newline="") as f:
        writer = csv.DictWriter(f, fieldnames=fieldnames, restval="")
        writer.writeheader()
        for record in run_records:
            writer.writerow(record)

    return path


def write_aggregate_csv(
    output_dir: str,
    aggregate_records: list[dict[str, Any]],
) -> str:
    """Write one row per parameter combination to a CSV file.

    Each record contains fixed and swept parameter values, per-metric
    mean/std/percentiles, and acceptance target pass/fail flags.
    Returns the path to the written file.
    """
    path = os.path.join(output_dir, "aggregate.csv")
    if not aggregate_records:
        with open(path, "w", newline="") as f:
            f.write("")
        return path

    fieldnames = _collect_all_fieldnames(aggregate_records)
    with open(path, "w", newline="") as f:
        writer = csv.DictWriter(f, fieldnames=fieldnames, restval="")
        writer.writeheader()
        for record in aggregate_records:
            writer.writerow(record)

    return path


def write_config_metadata(
    output_dir: str,
    cfg: config.SimulatorConfig,
) -> str:
    """Write the full configuration as a JSON file for reproducibility.

    Includes the simulator version string. Returns the path to the
    written file.
    """
    path = os.path.join(output_dir, "config_metadata.json")

    metadata: dict[str, Any] = {
        "version": VERSION,
        "draft": _dataclass_to_dict(cfg.draft),
        "cube": _dataclass_to_dict(cfg.cube),
        "pack_generation": _dataclass_to_dict(cfg.pack_generation),
        "refill": _dataclass_to_dict(cfg.refill),
        "cards": _dataclass_to_dict(cfg.cards),
        "agents": _dataclass_to_dict(cfg.agents),
        "scoring": _dataclass_to_dict(cfg.scoring),
        "commitment": _dataclass_to_dict(cfg.commitment),
        "metrics": _dataclass_to_dict(cfg.metrics),
        "sweep": _dataclass_to_dict(cfg.sweep),
    }

    with open(path, "w") as f:
        json.dump(metadata, f, indent=2, default=str)

    return path


def write_trace_json(
    output_dir: str,
    traces: list[draft_runner.PickTrace],
    seed: int,
) -> str:
    """Write per-pick trace records to a JSON file.

    Output path: {output_dir}/trace_seed{seed}.json. Each record contains
    round_index, pick_index, seat_index, pack_id, pack_card_ids,
    shown_card_ids, chosen_card_id, agent_w, and card_score.
    Returns the path to the written file.
    """
    path = os.path.join(output_dir, f"trace_seed{seed}.json")

    records: list[dict[str, Any]] = []
    for trace in traces:
        records.append(
            {
                "round_index": trace.round_index,
                "pick_index": trace.pick_index,
                "seat_index": trace.seat_index,
                "pack_id": trace.pack_id,
                "pack_card_ids": trace.pack_card_ids,
                "shown_card_ids": trace.shown_card_ids,
                "chosen_card_id": trace.chosen_card_id,
                "agent_w": [round(v, 6) for v in trace.agent_w_snapshot],
                "card_score": round(trace.card_score, 6),
            }
        )

    with open(path, "w") as f:
        json.dump(records, f, indent=2)

    return path


def build_run_record(
    run_id: int,
    config_hash: str,
    seed: int,
    result: draft_runner.DraftResult,
    draft_metrics: metrics.DraftMetrics,
    cfg: config.SimulatorConfig,
) -> dict[str, Any]:
    """Build a flat dictionary for one simulation run.

    Contains run_id, config_hash, seed, all per-draft metric values,
    and per-seat data.
    """
    record: dict[str, Any] = {
        "run_id": run_id,
        "config_hash": config_hash,
        "seed": seed,
    }

    # Choice richness shown
    cr = draft_metrics.choice_richness_shown
    record["cr_shown_near_opt_early"] = round(cr.near_optimal.early, 4)
    record["cr_shown_near_opt_mid"] = round(cr.near_optimal.mid, 4)
    record["cr_shown_near_opt_late"] = round(cr.near_optimal.late, 4)
    record["cr_shown_near_opt_overall"] = round(cr.near_optimal.overall, 4)
    record["cr_shown_score_gap_overall"] = round(cr.score_gap_mean.overall, 4)
    record["cr_shown_entropy_overall"] = round(cr.choice_entropy.overall, 4)

    # Choice richness full
    crf = draft_metrics.choice_richness_full
    record["cr_full_near_opt_overall"] = round(crf.near_optimal.overall, 4)
    record["cr_full_score_gap_overall"] = round(crf.score_gap_mean.overall, 4)
    record["cr_full_entropy_overall"] = round(crf.choice_entropy.overall, 4)

    # Convergence
    record["conv_shown_late_mean"] = round(
        draft_metrics.convergence_shown.on_plan_density_late_mean, 4
    )
    record["conv_shown_late_p3"] = round(
        draft_metrics.convergence_shown.on_plan_prob_gte_3_late, 4
    )
    record["conv_full_late_mean"] = round(
        draft_metrics.convergence_full.on_plan_density_late_mean, 4
    )

    # Splashability
    record["splash_shown"] = round(draft_metrics.splashability_shown.splash_fraction, 4)
    record["splash_full"] = round(draft_metrics.splashability_full.splash_fraction, 4)

    # Early openness
    record["openness_shown_archetypes"] = round(
        draft_metrics.early_openness_shown.archetypes_exposed, 4
    )
    record["openness_shown_entropy"] = round(
        draft_metrics.early_openness_shown.preference_entropy, 4
    )

    # Signal benefit and forceability
    record["signal_benefit"] = (
        round(draft_metrics.signal_benefit, 4)
        if draft_metrics.signal_benefit is not None
        else ""
    )
    record["forceability_max"] = (
        round(draft_metrics.forceability, 4)
        if draft_metrics.forceability is not None
        else ""
    )

    # Per-seat data
    for seat_idx, sr in enumerate(result.seat_results):
        record[f"seat{seat_idx}_deck_value"] = round(sr.deck_value, 4)
        record[f"seat{seat_idx}_commitment_pick"] = (
            sr.commitment_pick if sr.commitment_pick is not None else ""
        )
        record[f"seat{seat_idx}_archetype"] = (
            sr.committed_archetype if sr.committed_archetype is not None else ""
        )

    return record


def _collect_all_fieldnames(records: list[dict[str, Any]]) -> list[str]:
    """Collect the superset of all keys across records, preserving order.

    Keys appear in the order they are first encountered across all
    records. This handles cases where different records have different
    columns (e.g., varying seat counts in sweep mode).
    """
    seen: set[str] = set()
    fieldnames: list[str] = []
    for record in records:
        for key in record.keys():
            if key not in seen:
                seen.add(key)
                fieldnames.append(key)
    return fieldnames


def _dataclass_to_dict(obj: Any) -> dict[str, Any]:
    """Convert a dataclass to a plain dict, handling nested types."""
    result: dict[str, Any] = {}
    for f in fields(obj):
        value = getattr(obj, f.name)
        if hasattr(value, "__dataclass_fields__"):
            result[f.name] = _dataclass_to_dict(value)
        else:
            result[f.name] = value
    return result
