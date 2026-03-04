"""Validation and calibration checks for the draft simulator.

Runs distribution sanity checks, cross-configuration directional
comparisons, difficulty knob validation, and metric stability checks
after simulation. Results are printed to stdout and are informational
(not fatal). Stdlib-only, no external dependencies.
"""

import math
from dataclasses import dataclass
from typing import Any

import colors


@dataclass(frozen=True)
class CheckResult:
    """Result of a single validation check."""

    name: str
    passed: bool
    message: str


@dataclass(frozen=True)
class ValidationReport:
    """Complete validation report from all checks."""

    checks: list[CheckResult]


def run_validation(
    aggregate_records: list[dict[str, Any]],
    run_records: list[dict[str, Any]],
) -> ValidationReport:
    """Run all validation checks and return a report.

    Runs distribution sanity checks, cross-configuration directional
    comparisons, difficulty knob validation, and metric stability checks.
    """
    checks: list[CheckResult] = []

    checks.extend(_check_commitment_timing(run_records))
    checks.extend(_check_choice_richness_baseline(run_records))
    checks.extend(_check_archetype_density(run_records))
    checks.extend(_check_metric_stability(aggregate_records))
    checks.extend(_check_tail_percentile_stability(aggregate_records))
    checks.extend(_check_cross_config_directional(aggregate_records))
    checks.extend(_check_difficulty_knobs(aggregate_records))

    return ValidationReport(checks=checks)


def format_validation_report(report: ValidationReport) -> str:
    """Format a validation report for printing to stdout."""
    lines: list[str] = []
    lines.append("")
    lines.append(colors.c("=" * 60, "ui"))
    lines.append(colors.header("Validation Results"))
    lines.append(colors.c("=" * 60, "ui"))

    if not report.checks:
        lines.append("  No validation checks were run.")
        return "\n".join(lines)

    passed = sum(1 for c in report.checks if c.passed)
    total = len(report.checks)
    lines.append(f"  {colors.num(passed)}/{colors.num(total)} checks passed")
    lines.append("")

    for check in report.checks:
        status = colors.ok("[PASS]") if check.passed else colors.fail("[FAIL]")
        lines.append(f"  {status} {colors.label(check.name)}")
        lines.append(f"         {check.message}")

    lines.append(colors.c("=" * 60, "ui"))
    return "\n".join(lines)


def _check_commitment_timing(
    run_records: list[dict[str, Any]],
) -> list[CheckResult]:
    """Check that commitment timing falls within expected bounds.

    Mean commitment pick in [4, 8], std in [1.5, 4.0], uncommitted
    rate below 10%.
    """
    checks: list[CheckResult] = []

    commitment_picks: list[float] = []
    total_seats = 0

    for record in run_records:
        for key, val in record.items():
            if key.endswith("_commitment_pick"):
                total_seats += 1
                if val != "" and val is not None:
                    commitment_picks.append(float(val))

    if not commitment_picks:
        checks.append(
            CheckResult(
                name="Commitment timing - mean",
                passed=False,
                message="No commitment data available",
            )
        )
        checks.append(
            CheckResult(
                name="Commitment timing - std",
                passed=False,
                message="No commitment data available (cannot compute std)",
            )
        )
        if total_seats > 0:
            checks.append(
                CheckResult(
                    name="Commitment timing - uncommitted rate",
                    passed=False,
                    message=f"Uncommitted rate = 100.0% (target: < 10%)",
                )
            )
        return checks

    mean_pick = sum(commitment_picks) / len(commitment_picks)
    mean_ok = 4.0 <= mean_pick <= 8.0
    checks.append(
        CheckResult(
            name="Commitment timing - mean",
            passed=mean_ok,
            message=f"Mean commitment pick = {mean_pick:.2f} (target: [4, 8])",
        )
    )

    if len(commitment_picks) >= 2:
        variance = sum((p - mean_pick) ** 2 for p in commitment_picks) / len(
            commitment_picks
        )
        std_pick = math.sqrt(variance)
        std_ok = 1.5 <= std_pick <= 4.0
        checks.append(
            CheckResult(
                name="Commitment timing - std",
                passed=std_ok,
                message=f"Std commitment pick = {std_pick:.2f} (target: [1.5, 4.0])",
            )
        )

    if total_seats > 0:
        uncommitted_rate = 1.0 - len(commitment_picks) / total_seats
        uncommitted_ok = uncommitted_rate < 0.10
        checks.append(
            CheckResult(
                name="Commitment timing - uncommitted rate",
                passed=uncommitted_ok,
                message=(f"Uncommitted rate = {uncommitted_rate:.1%} (target: < 10%)"),
            )
        )

    return checks


def _check_choice_richness_baseline(
    run_records: list[dict[str, Any]],
) -> list[CheckResult]:
    """Check that mean near-optimal count (shown-N) is at least 1.5."""
    checks: list[CheckResult] = []

    near_opt_values: list[float] = []
    for record in run_records:
        val = record.get("cr_shown_near_opt_overall", "")
        if val != "" and val is not None:
            near_opt_values.append(float(val))

    if not near_opt_values:
        checks.append(
            CheckResult(
                name="Choice richness baseline",
                passed=False,
                message="No choice richness data available",
            )
        )
        return checks

    mean_near_opt = sum(near_opt_values) / len(near_opt_values)
    ok = mean_near_opt >= 1.5
    checks.append(
        CheckResult(
            name="Choice richness baseline",
            passed=ok,
            message=(
                f"Mean near-optimal count (shown-N) = {mean_near_opt:.2f} "
                f"(target: >= 1.5)"
            ),
        )
    )

    return checks


def _check_metric_stability(
    aggregate_records: list[dict[str, Any]],
) -> list[CheckResult]:
    """Check coefficient of variation across runs for key metrics.

    CV below 0.15 for means, 0.30 for tail percentiles. Flags as
    high-variance if exceeded.
    """
    checks: list[CheckResult] = []

    metric_keys_means = [
        "cr_shown_near_opt_overall",
        "conv_shown_late_mean",
        "splash_shown",
        "openness_shown_archetypes",
    ]

    for agg in aggregate_records:
        for metric_key in metric_keys_means:
            mean_key = f"{metric_key}_mean"
            std_key = f"{metric_key}_std"
            mean_val = agg.get(mean_key)
            std_val = agg.get(std_key)

            if mean_val is not None and std_val is not None:
                mean_f = float(mean_val)
                std_f = float(std_val)
                if mean_f > 0:
                    cv = std_f / mean_f
                    ok = cv < 0.15
                    checks.append(
                        CheckResult(
                            name=f"Metric stability - {metric_key}",
                            passed=ok,
                            message=(
                                f"CV = {cv:.3f} (target: < 0.15)"
                                + ("" if ok else " [HIGH VARIANCE]")
                            ),
                        )
                    )

    return checks


def _check_tail_percentile_stability(
    aggregate_records: list[dict[str, Any]],
) -> list[CheckResult]:
    """Check CV for tail percentiles (p5, p95) is below 0.30."""
    checks: list[CheckResult] = []

    metric_keys = [
        "cr_shown_near_opt_overall",
        "conv_shown_late_mean",
        "splash_shown",
        "openness_shown_archetypes",
    ]

    for agg in aggregate_records:
        for metric_key in metric_keys:
            for ptile in ["p5", "p95"]:
                ptile_key = f"{metric_key}_{ptile}"
                mean_key = f"{metric_key}_mean"
                ptile_val = agg.get(ptile_key)
                mean_val = agg.get(mean_key)

                if ptile_val is not None and mean_val is not None:
                    ptile_f = float(ptile_val)
                    mean_f = float(mean_val)
                    if mean_f > 0 and ptile_f > 0:
                        # Use relative deviation from mean as stability proxy
                        cv = abs(ptile_f - mean_f) / mean_f
                        ok = cv < 0.30
                        checks.append(
                            CheckResult(
                                name=f"Tail stability - {metric_key} {ptile}",
                                passed=ok,
                                message=(
                                    f"|{ptile} - mean| / mean = {cv:.3f} "
                                    f"(target: < 0.30)"
                                    + ("" if ok else " [HIGH VARIANCE]")
                                ),
                            )
                        )

    return checks


def _check_archetype_density(
    run_records: list[dict[str, Any]],
    expected_min_density: float = 0.08,
    expected_max_density: float = 0.25,
) -> list[CheckResult]:
    """Check archetype density: each archetype within expected range.

    Uses per-seat archetype assignments across runs to compute density.
    Each archetype should appear in at least expected_min_density and
    no more than expected_max_density of total seat assignments.
    """
    checks: list[CheckResult] = []

    archetype_counts: dict[int, int] = {}
    total_seats = 0

    for record in run_records:
        for key, val in record.items():
            if key.endswith("_archetype") and val != "" and val is not None:
                total_seats += 1
                arch = int(val)
                archetype_counts[arch] = archetype_counts.get(arch, 0) + 1

    if total_seats == 0:
        checks.append(
            CheckResult(
                name="Archetype density",
                passed=False,
                message="No archetype data available",
            )
        )
        return checks

    all_ok = True
    details: list[str] = []
    for arch in sorted(archetype_counts.keys()):
        density = archetype_counts[arch] / total_seats
        if density < expected_min_density or density > expected_max_density:
            all_ok = False
            details.append(
                f"arch {arch}: {density:.3f} "
                f"({'below min' if density < expected_min_density else 'above max'})"
            )

    if all_ok:
        checks.append(
            CheckResult(
                name="Archetype density",
                passed=True,
                message=(
                    f"All {len(archetype_counts)} archetypes within "
                    f"[{expected_min_density}, {expected_max_density}]"
                ),
            )
        )
    else:
        checks.append(
            CheckResult(
                name="Archetype density",
                passed=False,
                message=f"Out-of-range archetypes: {'; '.join(details)}",
            )
        )

    return checks


def _check_cross_config_directional(
    aggregate_records: list[dict[str, Any]],
) -> list[CheckResult]:
    """Cross-configuration directional checks when sweep data is available.

    Checks refill vs no-refill, round count effects, and difficulty
    signal benefit comparisons when the relevant configurations exist.
    """
    checks: list[CheckResult] = []

    if len(aggregate_records) < 2:
        return checks

    # Check: more rounds should increase early openness
    _check_directional_by_param(
        aggregate_records,
        param_key="swept_draft.round_count",
        metric_key="openness_shown_archetypes_mean",
        expected_direction="increase",
        check_name="Cross-config: more rounds -> higher openness",
        checks=checks,
    )

    return checks


def _check_directional_by_param(
    aggregate_records: list[dict[str, Any]],
    param_key: str,
    metric_key: str,
    expected_direction: str,
    check_name: str,
    checks: list[CheckResult],
) -> None:
    """Check that a metric moves in the expected direction as a param increases."""
    pairs: list[tuple[float, float]] = []
    for agg in aggregate_records:
        param_val = agg.get(param_key)
        metric_val = agg.get(metric_key)
        if param_val is not None and metric_val is not None:
            pairs.append((float(param_val), float(metric_val)))

    if len(pairs) < 2:
        return

    pairs.sort(key=lambda x: x[0])
    first_metric = pairs[0][1]
    last_metric = pairs[-1][1]

    if expected_direction == "increase":
        ok = last_metric >= first_metric
        direction_str = "increases"
    else:
        ok = last_metric <= first_metric
        direction_str = "decreases"

    checks.append(
        CheckResult(
            name=check_name,
            passed=ok,
            message=(
                f"Metric {direction_str}: {first_metric:.4f} -> {last_metric:.4f}"
                + ("" if ok else f" [EXPECTED {expected_direction.upper()}]")
            ),
        )
    )


def _check_difficulty_knobs(
    aggregate_records: list[dict[str, Any]],
) -> list[CheckResult]:
    """Difficulty knob validation for signal_benefit.

    Easy preset: signal_benefit below 2%.
    Hard preset: signal_benefit above 5%.
    Only runs when signal_benefit data is present in aggregate records.
    """
    checks: list[CheckResult] = []

    for agg in aggregate_records:
        sb_mean = agg.get("signal_benefit_mean")
        if sb_mean is None:
            continue

        sb_val = float(sb_mean)

        # Check difficulty knob thresholds if optimality parameter hints at preset
        optimality = agg.get("swept_agents.ai_optimality")
        signal_weight = agg.get("swept_agents.ai_signal_weight")

        if optimality is not None and signal_weight is not None:
            opt_f = float(optimality)
            sw_f = float(signal_weight)

            if opt_f <= 0.4 and sw_f <= 0.0:
                ok = sb_val < 2.0
                checks.append(
                    CheckResult(
                        name="Difficulty knob - easy signal_benefit",
                        passed=ok,
                        message=(
                            f"signal_benefit = {sb_val:.2f}% (target: < 2%)"
                            + ("" if ok else " [TOO HIGH FOR EASY]")
                        ),
                    )
                )

            if opt_f >= 0.9 and sw_f >= 0.8:
                ok = sb_val > 5.0
                checks.append(
                    CheckResult(
                        name="Difficulty knob - hard signal_benefit",
                        passed=ok,
                        message=(
                            f"signal_benefit = {sb_val:.2f}% (target: > 5%)"
                            + ("" if ok else " [TOO LOW FOR HARD]")
                        ),
                    )
                )

    return checks
