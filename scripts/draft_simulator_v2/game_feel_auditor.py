#!/usr/bin/env python3
"""Game feel auditor for draft quality evaluation.

Runs quest-mode drafts via draft_runner.run_draft() and post-processes
traces with player-experience metrics. Evaluates draft quality from the
player's perspective using stricter thresholds and tracking frustration
patterns. Stdlib-only, no external dependencies.
"""

import argparse
import math
import os
import random
import sys
from dataclasses import dataclass, field
from typing import Optional

import colors
import config
import draft_runner
import metrics
from draft_models import CardInstance
from utils import argmax

ARCHETYPE_NAMES: list[str] = [
    "Flash",
    "Awaken",
    "Flicker",
    "Ignite",
    "Shatter",
    "Endure",
    "Submerge",
    "Surge",
]

ARCHETYPE_RESONANCE: dict[str, tuple[str, str]] = {
    "Flash": ("Thunder", "Tide"),
    "Awaken": ("Thunder", "Flame"),
    "Flicker": ("Flame", "Thunder"),
    "Ignite": ("Flame", "Stone"),
    "Shatter": ("Stone", "Flame"),
    "Endure": ("Stone", "Tide"),
    "Submerge": ("Tide", "Stone"),
    "Surge": ("Tide", "Thunder"),
}


# ---------------------------------------------------------------------------
# Data structures
# ---------------------------------------------------------------------------


@dataclass(frozen=True)
class PickFeelScore:
    """Metrics for a single pick from the player's perspective."""

    pick_index: int
    shown_count: int
    on_plan_strict: int
    on_plan_permissive: int
    best_fitness: float
    is_frustrating: bool
    post_resonance_on_plan: int


@dataclass(frozen=True)
class DraftFeelReport:
    """Feel metrics for a single draft run."""

    seed: int
    committed_archetype: Optional[int]
    commitment_pick: Optional[int]
    pick_scores: list[PickFeelScore]
    frustration_streak_max: int
    frustration_rate: float
    zero_option_picks: int
    mean_on_plan_strict: float
    resonance_impact: float


@dataclass(frozen=True)
class AuditSummary:
    """Aggregated feel metrics across many draft runs."""

    total_runs: int
    mean_frustration_rate: float
    p90_frustration_rate: float
    mean_max_streak: float
    p90_max_streak: float
    mean_zero_option_picks: float
    per_pick_mean_on_plan: dict[int, float]
    tail_runs: list[DraftFeelReport]
    resonance_impact_mean: float


# ---------------------------------------------------------------------------
# Quest-mode config builder
# ---------------------------------------------------------------------------


def _quest_mode_config() -> config.SimulatorConfig:
    """Build a SimulatorConfig matching quest mode settings."""
    cfg = config.SimulatorConfig()
    cfg.draft.seat_count = 6
    cfg.draft.pack_size = 20
    cfg.draft.human_seats = 1
    cfg.draft.alternate_direction = False
    cfg.agents.show_n = 4
    cfg.agents.show_n_strategy = "sharpened_preference"
    cfg.agents.policy = "adaptive"
    cfg.agents.ai_optimality = 0.80
    cfg.agents.learning_rate = 3.0
    cfg.agents.openness_window = 3
    cfg.cards.archetype_count = 8
    cfg.cube.distinct_cards = 360
    cfg.cube.copies_per_card = 1
    cfg.cube.consumption_mode = "with_replacement"
    cfg.rarity.enabled = True
    cfg.pack_generation.strategy = "seeded_themed"

    script_dir = os.path.dirname(os.path.abspath(__file__))
    cfg.cards.rendered_toml_path = os.path.join(
        script_dir, "..", "..", "rules_engine", "tabula", "rendered-cards.toml"
    )

    return cfg


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------


def _mean(values: list[float]) -> float:
    if not values:
        return 0.0
    return sum(values) / len(values)


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


def _resonance_pair_for_archetype(arch_index: int) -> Optional[tuple[str, str]]:
    """Get the resonance pair for an archetype index."""
    if 0 <= arch_index < len(ARCHETYPE_NAMES):
        name = ARCHETYPE_NAMES[arch_index]
        return ARCHETYPE_RESONANCE.get(name)
    return None


def _passes_resonance_filter(
    card: CardInstance,
    resonance_pair: Optional[tuple[str, str]],
) -> bool:
    """Check if a card passes the resonance filter for a given pair."""
    if resonance_pair is None:
        return True
    res = card.design.resonance
    if len(res) < 2:
        return True
    return set(res) == set(resonance_pair)


# ---------------------------------------------------------------------------
# Core evaluation
# ---------------------------------------------------------------------------


def evaluate_draft_feel(
    result: draft_runner.DraftResult,
    cfg: config.SimulatorConfig,
    resonance_pair: Optional[tuple[str, str]],
    strict_threshold: float,
) -> DraftFeelReport:
    """Evaluate draft feel for the human seat in a single draft run."""
    human_seat = 0
    sr = result.seat_results[human_seat]

    committed_arch = sr.committed_archetype
    commitment_pick = sr.commitment_pick

    pick_scores: list[PickFeelScore] = []
    frustrating_count = 0
    zero_option_count = 0
    post_commitment_count = 0
    on_plan_strict_values: list[float] = []
    resonance_reductions: list[float] = []

    for trace in result.traces:
        if trace.seat_index != human_seat:
            continue
        if commitment_pick is not None and trace.pick_index <= commitment_pick:
            continue
        if committed_arch is None:
            continue

        post_commitment_count += 1
        shown_ids = trace.shown_card_ids
        shown_cards = [
            result.card_pool[cid] for cid in shown_ids if cid in result.card_pool
        ]

        strict_count = sum(
            1
            for c in shown_cards
            if c.design.fitness[committed_arch] >= strict_threshold
        )
        permissive_count = sum(
            1 for c in shown_cards if c.design.fitness[committed_arch] >= 0.3
        )
        best_fit = max(
            (c.design.fitness[committed_arch] for c in shown_cards),
            default=0.0,
        )

        post_res_cards = [
            c for c in shown_cards if _passes_resonance_filter(c, resonance_pair)
        ]
        post_res_strict = sum(
            1
            for c in post_res_cards
            if c.design.fitness[committed_arch] >= strict_threshold
        )

        resonance_reduction = strict_count - post_res_strict
        resonance_reductions.append(float(resonance_reduction))

        is_frustrating = post_res_strict <= 1

        if is_frustrating:
            frustrating_count += 1
        if post_res_strict == 0:
            zero_option_count += 1

        on_plan_strict_values.append(float(post_res_strict))

        pick_scores.append(
            PickFeelScore(
                pick_index=trace.pick_index,
                shown_count=len(shown_cards),
                on_plan_strict=strict_count,
                on_plan_permissive=permissive_count,
                best_fitness=best_fit,
                is_frustrating=is_frustrating,
                post_resonance_on_plan=post_res_strict,
            )
        )

    frustration_rate = (
        frustrating_count / post_commitment_count if post_commitment_count > 0 else 0.0
    )

    max_streak = _max_frustration_streak(pick_scores)

    return DraftFeelReport(
        seed=result.seed,
        committed_archetype=committed_arch,
        commitment_pick=commitment_pick,
        pick_scores=pick_scores,
        frustration_streak_max=max_streak,
        frustration_rate=frustration_rate,
        zero_option_picks=zero_option_count,
        mean_on_plan_strict=_mean(on_plan_strict_values),
        resonance_impact=_mean(resonance_reductions),
    )


def _max_frustration_streak(scores: list[PickFeelScore]) -> int:
    """Compute the longest consecutive run of frustrating picks."""
    max_streak = 0
    current = 0
    for s in scores:
        if s.is_frustrating:
            current += 1
            max_streak = max(max_streak, current)
        else:
            current = 0
    return max_streak


# ---------------------------------------------------------------------------
# Aggregation
# ---------------------------------------------------------------------------


def aggregate_reports(reports: list[DraftFeelReport]) -> AuditSummary:
    """Aggregate per-run feel reports into a summary."""
    if not reports:
        return AuditSummary(
            total_runs=0,
            mean_frustration_rate=0.0,
            p90_frustration_rate=0.0,
            mean_max_streak=0.0,
            p90_max_streak=0.0,
            mean_zero_option_picks=0.0,
            per_pick_mean_on_plan={},
            tail_runs=[],
            resonance_impact_mean=0.0,
        )

    frust_rates = [r.frustration_rate for r in reports]
    max_streaks = [float(r.frustration_streak_max) for r in reports]
    zero_opts = [float(r.zero_option_picks) for r in reports]
    res_impacts = [r.resonance_impact for r in reports]

    per_pick_accum: dict[int, list[float]] = {}
    for report in reports:
        for ps in report.pick_scores:
            per_pick_accum.setdefault(ps.pick_index, []).append(
                float(ps.post_resonance_on_plan)
            )

    per_pick_mean = {k: _mean(v) for k, v in sorted(per_pick_accum.items())}

    sorted_by_frust = sorted(reports, key=lambda r: r.frustration_rate, reverse=True)
    tail_count = max(1, len(reports) // 10)
    tail_runs = sorted_by_frust[:tail_count]

    return AuditSummary(
        total_runs=len(reports),
        mean_frustration_rate=_mean(frust_rates),
        p90_frustration_rate=_percentile(frust_rates, 90),
        mean_max_streak=_mean(max_streaks),
        p90_max_streak=_percentile(max_streaks, 90),
        mean_zero_option_picks=_mean(zero_opts),
        per_pick_mean_on_plan=per_pick_mean,
        tail_runs=tail_runs,
        resonance_impact_mean=_mean(res_impacts),
    )


# ---------------------------------------------------------------------------
# Formatted output
# ---------------------------------------------------------------------------


def _status(
    value: float,
    green_thresh: float,
    yellow_thresh: float,
    direction: str = "lt",
) -> str:
    """Return a colored PASS/WARN/FAIL indicator."""
    if direction == "lt":
        if value < green_thresh:
            return colors.ok("[PASS]")
        if value < yellow_thresh:
            return colors.warn("[WARN]")
        return colors.fail("[FAIL]")
    else:
        if value >= green_thresh:
            return colors.ok("[PASS]")
        if value >= yellow_thresh:
            return colors.warn("[WARN]")
        return colors.fail("[FAIL]")


def format_audit_summary(
    summary: AuditSummary,
    strict_threshold: float,
    label: str = "",
) -> str:
    """Format an AuditSummary as colored terminal output."""
    lines: list[str] = []

    title = f"=== Game Feel Audit ({summary.total_runs} runs)"
    if label:
        title += f" — {label}"
    title += " ==="
    lines.append(colors.header(title))
    lines.append("")

    lines.append(colors.section("Frustration Metrics:"))
    lines.append(
        f"  {_status(summary.mean_frustration_rate, 0.20, 0.35)}  "
        f"{colors.label('Mean frustration rate:')}"
        f"       {colors.num(f'{summary.mean_frustration_rate:.2f}')}  "
        f"{colors.dim('(target: < 0.20)')}"
    )
    lines.append(
        f"  {_status(summary.p90_frustration_rate, 0.35, 0.50)}  "
        f"{colors.label('P90 frustration rate:')}"
        f"        {colors.num(f'{summary.p90_frustration_rate:.2f}')}  "
        f"{colors.dim('(target: < 0.35)')}"
    )
    lines.append(
        f"  {_status(summary.mean_max_streak, 3.0, 5.0)}  "
        f"{colors.label('Mean max frustration streak:')}"
        f" {colors.num(f'{summary.mean_max_streak:.1f}')}  "
        f"{colors.dim('(target: < 3)')}"
    )
    lines.append(
        f"  {_status(summary.p90_max_streak, 5.0, 7.0)}  "
        f"{colors.label('P90 max frustration streak:')}"
        f"  {colors.num(f'{summary.p90_max_streak:.1f}')}  "
        f"{colors.dim('(target: < 5)')}"
    )
    lines.append(
        f"  {'      '}  "
        f"{colors.label('Mean zero-option picks:')}"
        f"      {colors.num(f'{summary.mean_zero_option_picks:.1f}')}"
    )

    lines.append("")
    lines.append(
        colors.section(
            f"Per-Pick On-Plan Density "
            f"(strict >= {strict_threshold}, post-commitment):"
        )
    )

    picks = sorted(summary.per_pick_mean_on_plan.keys())
    if picks:
        chunk_size = 4
        for i in range(0, len(picks), chunk_size):
            chunk = picks[i : i + chunk_size]
            parts = [
                f"Pick {colors.num(f'{p:2d}')}: {colors.num(f'{summary.per_pick_mean_on_plan[p]:.1f}')}"
                for p in chunk
            ]
            lines.append("  " + "    ".join(parts))

    lines.append("")
    lines.append(colors.section("Resonance Impact:"))
    lines.append(
        f"  {colors.label('Mean on-plan reduction:')}"
        f" {colors.num(f'-{summary.resonance_impact_mean:.1f}')} "
        f"{colors.dim('cards/pick')}"
    )

    if summary.tail_runs:
        lines.append("")
        lines.append(colors.section(f"Worst Runs (top {len(summary.tail_runs)}):"))
        for report in summary.tail_runs:
            arch_name = (
                ARCHETYPE_NAMES[report.committed_archetype]
                if report.committed_archetype is not None
                and 0 <= report.committed_archetype < len(ARCHETYPE_NAMES)
                else "?"
            )
            lines.append(
                f"  {colors.dim(f'seed={report.seed}:')} "
                f"{colors.label('frustration=')}"
                f"{colors.num(f'{report.frustration_rate:.2f}')}, "
                f"{colors.label('max_streak=')}"
                f"{colors.num(str(report.frustration_streak_max))}, "
                f"{colors.label('archetype=')}"
                f"{colors.num(arch_name)}"
            )

    return "\n".join(lines)


# ---------------------------------------------------------------------------
# Runner
# ---------------------------------------------------------------------------


def run_audit(
    runs: int,
    seed: int,
    strict_threshold: float,
    resonance_pair: Optional[tuple[str, str]],
    verbose: bool = False,
    pack_boost: int = 0,
    sharpening_decay: float = 0.0,
    mercy_reshuffle: bool = False,
) -> tuple[AuditSummary, list[DraftFeelReport]]:
    """Run the audit for a number of draft runs and return results."""
    cfg = _quest_mode_config()
    cfg.draft.pack_size += pack_boost
    cfg.agents.sharpening_decay = sharpening_decay
    cfg.agents.mercy_reshuffle = mercy_reshuffle
    rng = random.Random(seed)

    reports: list[DraftFeelReport] = []
    for i in range(runs):
        run_seed = rng.randint(0, 2**32 - 1)
        sys.stderr.write(
            colors.format_progress_bar(i, runs, label="drafts complete") + "\r"
        )
        sys.stderr.flush()

        result = draft_runner.run_draft(cfg, run_seed, trace_enabled=True)

        effective_pair = resonance_pair
        if effective_pair is None:
            sr = result.seat_results[0]
            if sr.committed_archetype is not None:
                effective_pair = _resonance_pair_for_archetype(sr.committed_archetype)

        report = evaluate_draft_feel(result, cfg, effective_pair, strict_threshold)
        reports.append(report)

    sys.stderr.write(
        colors.format_progress_bar(runs, runs, label="drafts complete") + "\n"
    )
    sys.stderr.flush()

    summary = aggregate_reports(reports)
    return summary, reports


def run_all_resonances(
    runs: int,
    seed: int,
    strict_threshold: float,
    verbose: bool = False,
) -> None:
    """Run the audit once per archetype resonance pair and print each."""
    all_reports: list[DraftFeelReport] = []

    for arch_idx, arch_name in enumerate(ARCHETYPE_NAMES):
        pair = ARCHETYPE_RESONANCE[arch_name]
        print(f"\n{colors.dim(f'--- Running {arch_name} ({pair[0]}/{pair[1]}) ---')}")

        summary, reports = run_audit(runs, seed, strict_threshold, pair, verbose)
        all_reports.extend(reports)
        print(format_audit_summary(summary, strict_threshold, label=arch_name))

    combined = aggregate_reports(all_reports)
    print(f"\n{colors.dim('--- Combined across all archetypes ---')}")
    print(format_audit_summary(combined, strict_threshold, label="all archetypes"))


def run_compare(
    runs: int,
    seed: int,
    strict_threshold: float,
) -> None:
    """Run audit and also print v2 convergence metrics for comparison."""
    cfg = _quest_mode_config()
    rng = random.Random(seed)

    feel_reports: list[DraftFeelReport] = []
    draft_metrics_list: list[metrics.DraftMetrics] = []

    for i in range(runs):
        run_seed = rng.randint(0, 2**32 - 1)
        sys.stderr.write(
            colors.format_progress_bar(i, runs, label="drafts complete") + "\r"
        )
        sys.stderr.flush()

        result = draft_runner.run_draft(cfg, run_seed, trace_enabled=True)

        sr = result.seat_results[0]
        pair = None
        if sr.committed_archetype is not None:
            pair = _resonance_pair_for_archetype(sr.committed_archetype)

        feel_reports.append(evaluate_draft_feel(result, cfg, pair, strict_threshold))
        draft_metrics_list.append(metrics.compute_metrics(result, cfg))

    sys.stderr.write(
        colors.format_progress_bar(runs, runs, label="drafts complete") + "\n"
    )
    sys.stderr.flush()

    feel_summary = aggregate_reports(feel_reports)
    print(format_audit_summary(feel_summary, strict_threshold))

    if draft_metrics_list:
        avg = metrics.average_metrics(draft_metrics_list)
        cis = metrics.compute_goal_cis(draft_metrics_list)
        print(f"\n{colors.dim('--- v2 Convergence Metrics (for comparison) ---')}")
        print(metrics.format_goal_metrics(avg, cis))


# ---------------------------------------------------------------------------
# CLI
# ---------------------------------------------------------------------------


def build_parser() -> argparse.ArgumentParser:
    """Build the argument parser."""
    parser = argparse.ArgumentParser(
        description="Game feel auditor for draft quality evaluation.",
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    parser.add_argument(
        "--runs",
        type=int,
        default=50,
        help="Number of draft runs (default: 50)",
    )
    parser.add_argument(
        "--seed",
        type=int,
        default=None,
        help="Random seed (default: random)",
    )
    parser.add_argument(
        "--strict-threshold",
        type=float,
        default=0.5,
        help="Fitness threshold for strict on-plan (default: 0.5)",
    )
    parser.add_argument(
        "--resonance",
        type=str,
        default=None,
        help="Archetype name to use for resonance filter (e.g. 'Flash')",
    )
    parser.add_argument(
        "--all-resonances",
        action="store_true",
        default=False,
        help="Run once per archetype, report each + average",
    )
    parser.add_argument(
        "--compare",
        action="store_true",
        default=False,
        help="Also print v2 convergence metrics for comparison",
    )
    parser.add_argument(
        "--verbose",
        action="store_true",
        default=False,
        help="Per-run detail for worst runs",
    )
    parser.add_argument(
        "--pack-boost",
        type=int,
        default=0,
        help="Increase pack size by N (default: 0)",
    )
    parser.add_argument(
        "--sharpening-decay",
        type=float,
        default=0.0,
        help="Sharpening exponent decay per pick in pack (default: 0.0)",
    )
    parser.add_argument(
        "--mercy-reshuffle",
        action="store_true",
        default=False,
        help="Redraw shown cards when all have 0 fitness for committed archetype",
    )
    return parser


def main() -> None:
    """Entry point."""
    parser = build_parser()
    args = parser.parse_args()

    seed = args.seed if args.seed is not None else random.randint(0, 2**32 - 1)

    resonance_pair: Optional[tuple[str, str]] = None
    if args.resonance:
        if args.resonance not in ARCHETYPE_RESONANCE:
            print(
                f"Unknown archetype: {args.resonance!r}. "
                f"Valid: {', '.join(ARCHETYPE_NAMES)}",
                file=sys.stderr,
            )
            sys.exit(1)
        resonance_pair = ARCHETYPE_RESONANCE[args.resonance]

    fix_label_parts: list[str] = []
    if args.pack_boost:
        fix_label_parts.append(f"pack_boost={args.pack_boost}")
    if args.sharpening_decay:
        fix_label_parts.append(f"sharpening_decay={args.sharpening_decay}")
    if args.mercy_reshuffle:
        fix_label_parts.append("mercy_reshuffle")
    fix_label = ", ".join(fix_label_parts) if fix_label_parts else "baseline"

    print(
        colors.dim(
            f"Game Feel Auditor — seed={seed}, runs={args.runs}, "
            f"strict_threshold={args.strict_threshold}, fixes=[{fix_label}]"
        )
    )

    if args.all_resonances:
        run_all_resonances(args.runs, seed, args.strict_threshold, args.verbose)
    elif args.compare:
        run_compare(args.runs, seed, args.strict_threshold)
    else:
        summary, reports = run_audit(
            args.runs,
            seed,
            args.strict_threshold,
            resonance_pair,
            args.verbose,
            pack_boost=args.pack_boost,
            sharpening_decay=args.sharpening_decay,
            mercy_reshuffle=args.mercy_reshuffle,
        )
        print(format_audit_summary(summary, args.strict_threshold))

        if args.verbose and summary.tail_runs:
            print(f"\n{colors.section('Verbose: Worst Run Details')}")
            for report in summary.tail_runs:
                print(f"\n  {colors.label(f'seed={report.seed}:')}")
                for ps in report.pick_scores:
                    marker = colors.fail("!") if ps.is_frustrating else " "
                    print(
                        f"    {marker} pick {ps.pick_index:2d}: "
                        f"strict={ps.post_resonance_on_plan} "
                        f"permissive={ps.on_plan_permissive} "
                        f"best_fit={ps.best_fitness:.2f}"
                    )


if __name__ == "__main__":
    main()
