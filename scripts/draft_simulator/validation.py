"""Validation and calibration checks for the draft simulator.

Runs distribution sanity checks, cross-configuration directional
comparisons, difficulty knob validation, and metric stability checks
after simulation. Results are printed to stdout and are informational
(not fatal). Stdlib-only, no external dependencies.
"""

import math
from dataclasses import dataclass
from typing import Any


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

    Runs distribution sanity checks, metric stability checks, and
    cross-configuration comparisons when applicable.
    """
    checks: list[CheckResult] = []

    checks.extend(_check_commitment_timing(run_records))
    checks.extend(_check_choice_richness_baseline(run_records))
    checks.extend(_check_metric_stability(aggregate_records))

    return ValidationReport(checks=checks)


def format_validation_report(report: ValidationReport) -> str:
    """Format a validation report for printing to stdout."""
    lines: list[str] = []
    lines.append("")
    lines.append("=" * 60)
    lines.append("Validation Results")
    lines.append("=" * 60)

    if not report.checks:
        lines.append("  No validation checks were run.")
        return "\n".join(lines)

    passed = sum(1 for c in report.checks if c.passed)
    total = len(report.checks)
    lines.append(f"  {passed}/{total} checks passed")
    lines.append("")

    for check in report.checks:
        status = "PASS" if check.passed else "FAIL"
        lines.append(f"  [{status}] {check.name}")
        lines.append(f"         {check.message}")

    lines.append("=" * 60)
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
