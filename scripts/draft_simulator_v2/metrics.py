"""Metrics engine for the draft simulator v2.

v2 changes: splashability uses tag_count >= splash_min_tags instead of
power/flex thresholds. Scoring functions updated for v2 scoring formula.
Stdlib-only, no external dependencies.
"""

import math
from dataclasses import dataclass, field
from typing import Optional

import agents
import colors
import config
import draft_runner
from draft_models import CardInstance
from utils import argmax

# ---------------------------------------------------------------------------
# Phase definitions
# ---------------------------------------------------------------------------

EARLY_START = 0
EARLY_END = 5
MID_START = 6
MID_END = 19
LATE_START = 20
LATE_END = 29


def pick_phase(pick_index: int) -> str:
    """Return the phase name for a given 0-based pick index."""
    if pick_index <= EARLY_END:
        return "early"
    if pick_index <= MID_END:
        return "mid"
    return "late"


# ---------------------------------------------------------------------------
# Mathematical utilities
# ---------------------------------------------------------------------------


def softmax(scores: list[float], tau: float) -> list[float]:
    """Compute softmax-normalized probabilities with temperature tau."""
    if not scores:
        return []
    if tau <= 0.0:
        tau = 1e-12
    max_s = max(scores)
    exps = [math.exp((s - max_s) / tau) for s in scores]
    total = sum(exps)
    if total <= 0.0:
        return [1.0 / len(scores)] * len(scores)
    return [e / total for e in exps]


def shannon_entropy(probs: list[float]) -> float:
    """Compute Shannon entropy in bits from a probability distribution."""
    if not probs:
        return 0.0
    entropy = 0.0
    for p in probs:
        if p > 0.0:
            entropy -= p * math.log2(p)
    return entropy


def choice_entropy(scores: list[float], tau: float) -> float:
    """Compute choice entropy: Shannon entropy of softmax-normalized scores."""
    probs = softmax(scores, tau)
    return shannon_entropy(probs)


def near_optimal_count(scores: list[float], gap: float) -> int:
    """Count scores within gap of the best score."""
    if not scores:
        return 0
    best = max(scores)
    threshold = best - gap
    return sum(1 for s in scores if s >= threshold)


def score_gap(scores: list[float]) -> float:
    """Difference between the best and second-best score."""
    if len(scores) < 2:
        return 0.0
    sorted_scores = sorted(scores, reverse=True)
    return sorted_scores[0] - sorted_scores[1]


# ---------------------------------------------------------------------------
# Phase-bucketed statistics
# ---------------------------------------------------------------------------


@dataclass(frozen=True)
class PhaseStats:
    """Statistics broken out by draft phase."""

    early: float
    mid: float
    late: float
    overall: float


@dataclass(frozen=True)
class ScoreGapStats:
    """Score gap statistics broken out by phase."""

    early_mean: float
    early_median: float
    early_p90: float
    mid_mean: float
    mid_median: float
    mid_p90: float
    late_mean: float
    late_median: float
    late_p90: float
    overall_mean: float
    overall_median: float
    overall_p90: float


@dataclass(frozen=True)
class ChoiceRichnessMetrics:
    """Choice richness metric family results."""

    near_optimal: PhaseStats
    score_gap_mean: PhaseStats
    score_gap_median: PhaseStats
    score_gap_p90: PhaseStats
    choice_entropy: PhaseStats


@dataclass(frozen=True)
class ConvergenceMetrics:
    """Convergence metric family results."""

    on_plan_density_mid_mean: float
    on_plan_prob_gte_3_mid: float
    on_plan_density_late_mean: float
    on_plan_prob_gte_3_late: float


@dataclass(frozen=True)
class SplashabilityMetrics:
    """Splashability metric family results."""

    splash_fraction: float


@dataclass(frozen=True)
class EarlyOpennessMetrics:
    """Early openness metric family results."""

    archetypes_exposed: float
    preference_entropy: float


@dataclass(frozen=True)
class CommitmentTimingMetrics:
    """Commitment timing metric family results."""

    mean_commitment_pick: float
    median_commitment_pick: float
    uncommitted_rate: float
    by_pick_5_rate: float


@dataclass(frozen=True)
class DraftMetrics:
    """Complete metrics for a single draft run."""

    choice_richness_full: ChoiceRichnessMetrics
    choice_richness_shown: ChoiceRichnessMetrics
    convergence_full: ConvergenceMetrics
    convergence_shown: ConvergenceMetrics
    splashability_full: SplashabilityMetrics
    splashability_shown: SplashabilityMetrics
    early_openness_full: EarlyOpennessMetrics
    early_openness_shown: EarlyOpennessMetrics
    commitment_timing: CommitmentTimingMetrics = field(
        default_factory=lambda: CommitmentTimingMetrics(0.0, 0.0, 1.0, 0.0)
    )
    forceability: Optional[float] = None
    forceability_archetype: Optional[int] = None
    forceability_per_archetype: Optional[dict[int, float]] = None
    signal_benefit: Optional[float] = None


@dataclass(frozen=True)
class GoalMetricCIs:
    """95% confidence interval half-widths for goal metrics."""

    convergence_mid: Optional[float] = None
    convergence_late: Optional[float] = None
    choice_richness: Optional[float] = None
    forceability: Optional[float] = None
    splashability: Optional[float] = None
    early_openness: Optional[float] = None
    signal_benefit: Optional[float] = None


# ---------------------------------------------------------------------------
# Per-pick scoring helpers
# ---------------------------------------------------------------------------


def _score_cards_for_seat(
    cards: list[CardInstance],
    seat_result: draft_runner.SeatResult,
    pick_index: int,
    policy: str,
    cfg: config.SimulatorConfig,
    openness_snapshot: list[float],
) -> list[float]:
    """Score a set of cards using the agent's state at a given pick."""
    archetype_count = cfg.cards.archetype_count
    if pick_index == 0:
        w_before = [1.0 / archetype_count] * archetype_count
    elif pick_index - 1 < len(seat_result.w_history):
        w_before = list(seat_result.w_history[pick_index - 1])
    else:
        w_before = list(seat_result.final_w)

    agent_snapshot = agents.AgentState(
        w=w_before,
        drafted=list(seat_result.drafted[:pick_index]),
        openness=list(openness_snapshot),
    )

    scores: list[float] = []
    for card in cards:
        if policy == "greedy":
            scores.append(agents.score_card_greedy(card, agent_snapshot, cfg.scoring))
        elif policy == "adaptive":
            scores.append(agents.score_card_adaptive(card, agent_snapshot, cfg.agents))
        elif policy == "signal_ignorant":
            scores.append(
                agents.score_card_signal_ignorant(card, agent_snapshot, cfg.agents)
            )
        elif policy == "archetype_loyal":
            best_arch = argmax(agent_snapshot.w)
            scores.append(card.design.fitness[best_arch])
        elif policy == "force":
            arch = (
                cfg.agents.force_archetype
                if cfg.agents.force_archetype is not None
                else 0
            )
            scores.append(card.design.fitness[arch])
        else:
            scores.append(agents.score_card_adaptive(card, agent_snapshot, cfg.agents))

    return scores


# ---------------------------------------------------------------------------
# Aggregate helpers
# ---------------------------------------------------------------------------


def _mean(values: list[float]) -> float:
    """Mean of a list of floats, or 0.0 if empty."""
    if not values:
        return 0.0
    return sum(values) / len(values)


def _median(values: list[float]) -> float:
    """Median of a list of floats, or 0.0 if empty."""
    if not values:
        return 0.0
    s = sorted(values)
    n = len(s)
    if n % 2 == 1:
        return s[n // 2]
    return (s[n // 2 - 1] + s[n // 2]) / 2.0


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


def _std(values: list[float]) -> float:
    """Population standard deviation."""
    if len(values) < 2:
        return 0.0
    m = _mean(values)
    variance = sum((v - m) ** 2 for v in values) / len(values)
    return math.sqrt(variance)


def _ci_95(values: list[float]) -> float:
    """95% confidence interval half-width for the mean."""
    n = len(values)
    if n < 2:
        return 0.0
    return 1.96 * _std(values) / math.sqrt(n)


def _bucket_by_phase(
    per_pick_values: list[tuple[int, float]],
) -> tuple[list[float], list[float], list[float], list[float]]:
    """Split (pick_index, value) pairs into early/mid/late/overall buckets."""
    early: list[float] = []
    mid: list[float] = []
    late: list[float] = []
    overall: list[float] = []

    for pick_idx, val in per_pick_values:
        overall.append(val)
        phase = pick_phase(pick_idx)
        if phase == "early":
            early.append(val)
        elif phase == "mid":
            mid.append(val)
        else:
            late.append(val)

    return early, mid, late, overall


# ---------------------------------------------------------------------------
# Choice Richness computation
# ---------------------------------------------------------------------------


def _compute_choice_richness(
    result: draft_runner.DraftResult,
    cfg: config.SimulatorConfig,
    use_shown: bool,
) -> ChoiceRichnessMetrics:
    """Compute choice richness metrics for a draft result."""
    richness_gap = cfg.metrics.richness_gap
    tau = cfg.metrics.tau

    near_opt_data: list[tuple[int, float]] = []
    gap_data: list[tuple[int, float]] = []
    entropy_data: list[tuple[int, float]] = []

    human_seat = 0

    for trace in result.traces:
        if trace.seat_index != human_seat:
            continue

        if use_shown:
            card_ids = trace.shown_card_ids
        else:
            card_ids = trace.pack_card_ids

        cards = [result.card_pool[cid] for cid in card_ids if cid in result.card_pool]
        if not cards:
            continue

        seat_result = result.seat_results[trace.seat_index]
        scores = _score_cards_for_seat(
            cards,
            seat_result,
            trace.pick_index,
            cfg.agents.policy,
            cfg,
            trace.agent_openness_snapshot,
        )

        near_opt = near_optimal_count(scores, richness_gap)
        gap = score_gap(scores)
        ent = choice_entropy(scores, tau)

        near_opt_data.append((trace.pick_index, float(near_opt)))
        gap_data.append((trace.pick_index, gap))
        entropy_data.append((trace.pick_index, ent))

    e_no, m_no, l_no, a_no = _bucket_by_phase(near_opt_data)
    e_g, m_g, l_g, a_g = _bucket_by_phase(gap_data)
    e_e, m_e, l_e, a_e = _bucket_by_phase(entropy_data)

    return ChoiceRichnessMetrics(
        near_optimal=PhaseStats(
            early=_mean(e_no),
            mid=_mean(m_no),
            late=_mean(l_no),
            overall=_mean(a_no),
        ),
        score_gap_mean=PhaseStats(
            early=_mean(e_g),
            mid=_mean(m_g),
            late=_mean(l_g),
            overall=_mean(a_g),
        ),
        score_gap_median=PhaseStats(
            early=_median(e_g),
            mid=_median(m_g),
            late=_median(l_g),
            overall=_median(a_g),
        ),
        score_gap_p90=PhaseStats(
            early=_percentile(e_g, 90),
            mid=_percentile(m_g, 90),
            late=_percentile(l_g, 90),
            overall=_percentile(a_g, 90),
        ),
        choice_entropy=PhaseStats(
            early=_mean(e_e),
            mid=_mean(m_e),
            late=_mean(l_e),
            overall=_mean(a_e),
        ),
    )


# ---------------------------------------------------------------------------
# Convergence computation
# ---------------------------------------------------------------------------


def _compute_convergence(
    result: draft_runner.DraftResult,
    cfg: config.SimulatorConfig,
    use_shown: bool,
) -> ConvergenceMetrics:
    """Compute convergence metrics for post-commitment picks."""
    on_plan_threshold = cfg.metrics.on_plan_threshold
    human_seat = 0

    empty = ConvergenceMetrics(
        on_plan_density_mid_mean=0.0,
        on_plan_prob_gte_3_mid=0.0,
        on_plan_density_late_mean=0.0,
        on_plan_prob_gte_3_late=0.0,
    )

    if not result.seat_results:
        return empty

    sr = result.seat_results[human_seat]

    if sr.commitment_pick is None or sr.committed_archetype is None:
        return empty

    commitment_pick = sr.commitment_pick
    committed_arch = sr.committed_archetype

    mid_on_plan_counts: list[float] = []
    late_on_plan_counts: list[float] = []

    for trace in result.traces:
        if trace.seat_index != human_seat:
            continue
        if trace.pick_index <= commitment_pick:
            continue

        if use_shown:
            card_ids = trace.shown_card_ids
        else:
            card_ids = trace.pack_card_ids

        cards = [result.card_pool[cid] for cid in card_ids if cid in result.card_pool]

        on_plan_count = sum(
            1 for c in cards if c.design.fitness[committed_arch] >= on_plan_threshold
        )

        phase = pick_phase(trace.pick_index)
        if phase == "mid":
            mid_on_plan_counts.append(float(on_plan_count))
        elif phase == "late":
            late_on_plan_counts.append(float(on_plan_count))

    if not mid_on_plan_counts and not late_on_plan_counts:
        return empty

    def _prob_gte_3(counts: list[float]) -> float:
        if not counts:
            return 0.0
        return sum(1 for c in counts if c >= 3.0) / len(counts)

    return ConvergenceMetrics(
        on_plan_density_mid_mean=_mean(mid_on_plan_counts),
        on_plan_prob_gte_3_mid=_prob_gte_3(mid_on_plan_counts),
        on_plan_density_late_mean=_mean(late_on_plan_counts),
        on_plan_prob_gte_3_late=_prob_gte_3(late_on_plan_counts),
    )


# ---------------------------------------------------------------------------
# Splashability computation (v2: tag_count based)
# ---------------------------------------------------------------------------


def _compute_splashability(
    result: draft_runner.DraftResult,
    cfg: config.SimulatorConfig,
    use_shown: bool,
) -> SplashabilityMetrics:
    """Compute splashability: fraction of post-commitment picks with a
    viable off-plan option.

    v2: Off-plan card is splashable if tag_count >= splash_min_tags
    (multi-archetype cards are inherently flexible).
    """
    human_seat = 0

    if not result.seat_results:
        return SplashabilityMetrics(splash_fraction=0.0)

    sr = result.seat_results[human_seat]

    if sr.commitment_pick is None or sr.committed_archetype is None:
        return SplashabilityMetrics(splash_fraction=0.0)

    commitment_pick = sr.commitment_pick
    committed_arch = sr.committed_archetype
    splash_min_tags = cfg.metrics.splash_min_tags

    post_commitment_picks = 0
    picks_with_splash = 0

    for trace in result.traces:
        if trace.seat_index != human_seat:
            continue
        if trace.pick_index <= commitment_pick:
            continue

        if use_shown:
            card_ids = trace.shown_card_ids
        else:
            card_ids = trace.pack_card_ids

        cards = [result.card_pool[cid] for cid in card_ids if cid in result.card_pool]

        post_commitment_picks += 1

        has_splash = any(
            c.design.fitness[committed_arch] < 0.3
            and sum(1 for f in c.design.fitness if f >= 1.0) >= splash_min_tags
            for c in cards
        )

        if has_splash:
            picks_with_splash += 1

    if post_commitment_picks == 0:
        return SplashabilityMetrics(splash_fraction=0.0)

    return SplashabilityMetrics(
        splash_fraction=picks_with_splash / post_commitment_picks
    )


# ---------------------------------------------------------------------------
# Early Openness computation
# ---------------------------------------------------------------------------


def _compute_early_openness(
    result: draft_runner.DraftResult,
    cfg: config.SimulatorConfig,
    use_shown: bool,
) -> EarlyOpennessMetrics:
    """Compute early openness metrics."""
    exposure_threshold = cfg.metrics.exposure_threshold
    human_seat = 0

    exposed_archetypes: set[int] = set()
    pref_entropies: list[float] = []

    for trace in result.traces:
        if trace.seat_index != human_seat:
            continue

        if trace.pick_index <= 4:
            if use_shown:
                card_ids = trace.shown_card_ids
            else:
                card_ids = trace.pack_card_ids

            cards = [
                result.card_pool[cid] for cid in card_ids if cid in result.card_pool
            ]

            for card in cards:
                for arch_idx, fit in enumerate(card.design.fitness):
                    if fit >= exposure_threshold:
                        exposed_archetypes.add(arch_idx)

        if trace.pick_index <= 5:
            w = trace.agent_w_snapshot
            total = sum(w)
            if total > 0:
                normalized = [v / total for v in w]
                pref_entropies.append(shannon_entropy(normalized))

    return EarlyOpennessMetrics(
        archetypes_exposed=float(len(exposed_archetypes)),
        preference_entropy=_mean(pref_entropies),
    )


# ---------------------------------------------------------------------------
# Commitment Timing computation
# ---------------------------------------------------------------------------


def _compute_commitment_timing(
    result: draft_runner.DraftResult,
) -> CommitmentTimingMetrics:
    """Compute commitment timing metrics across all seats."""
    committed_picks: list[float] = []
    total_seats = len(result.seat_results)

    for sr in result.seat_results:
        if sr.commitment_pick is not None:
            committed_picks.append(float(sr.commitment_pick))

    if not committed_picks:
        return CommitmentTimingMetrics(
            mean_commitment_pick=0.0,
            median_commitment_pick=0.0,
            uncommitted_rate=1.0 if total_seats > 0 else 0.0,
            by_pick_5_rate=0.0,
        )

    uncommitted = total_seats - len(committed_picks)
    by_pick_5 = sum(1 for p in committed_picks if p <= 5)

    return CommitmentTimingMetrics(
        mean_commitment_pick=_mean(committed_picks),
        median_commitment_pick=_median(committed_picks),
        uncommitted_rate=uncommitted / total_seats if total_seats > 0 else 0.0,
        by_pick_5_rate=by_pick_5 / total_seats if total_seats > 0 else 0.0,
    )


# ---------------------------------------------------------------------------
# Main metrics computation
# ---------------------------------------------------------------------------


def compute_metrics(
    result: draft_runner.DraftResult,
    cfg: config.SimulatorConfig,
    force_deck_values: Optional[dict[int, list[float]]] = None,
    adaptive_deck_values: Optional[list[float]] = None,
    aware_deck_values: Optional[list[float]] = None,
    ignorant_deck_values: Optional[list[float]] = None,
) -> DraftMetrics:
    """Compute all six metric families from a draft result."""
    choice_richness_full = _compute_choice_richness(result, cfg, use_shown=False)
    choice_richness_shown = _compute_choice_richness(result, cfg, use_shown=True)

    convergence_full = _compute_convergence(result, cfg, use_shown=False)
    convergence_shown = _compute_convergence(result, cfg, use_shown=True)

    splashability_full = _compute_splashability(result, cfg, use_shown=False)
    splashability_shown = _compute_splashability(result, cfg, use_shown=True)

    early_openness_full = _compute_early_openness(result, cfg, use_shown=False)
    early_openness_shown = _compute_early_openness(result, cfg, use_shown=True)

    commitment_timing = _compute_commitment_timing(result)

    forceability_val: Optional[float] = None
    forceability_arch: Optional[int] = None
    forceability_per_arch: Optional[dict[int, float]] = None
    if force_deck_values is not None and adaptive_deck_values is not None:
        adaptive_mean = _mean(adaptive_deck_values)
        if adaptive_mean > 0:
            per_arch: dict[int, float] = {}
            max_fi = 0.0
            max_arch = 0
            for arch, dvs in force_deck_values.items():
                fi = _mean(dvs) / adaptive_mean
                per_arch[arch] = fi
                if fi > max_fi:
                    max_fi = fi
                    max_arch = arch
            forceability_val = max_fi
            forceability_arch = max_arch
            forceability_per_arch = per_arch

    signal_benefit_val: Optional[float] = None
    if aware_deck_values is not None and ignorant_deck_values is not None:
        mean_aware = _mean(aware_deck_values)
        mean_ignorant = _mean(ignorant_deck_values)
        if mean_ignorant > 0:
            signal_benefit_val = ((mean_aware - mean_ignorant) / mean_ignorant) * 100.0

    return DraftMetrics(
        choice_richness_full=choice_richness_full,
        choice_richness_shown=choice_richness_shown,
        convergence_full=convergence_full,
        convergence_shown=convergence_shown,
        splashability_full=splashability_full,
        splashability_shown=splashability_shown,
        early_openness_full=early_openness_full,
        early_openness_shown=early_openness_shown,
        commitment_timing=commitment_timing,
        forceability=forceability_val,
        forceability_archetype=forceability_arch,
        forceability_per_archetype=forceability_per_arch,
        signal_benefit=signal_benefit_val,
    )


# ---------------------------------------------------------------------------
# Text formatting
# ---------------------------------------------------------------------------


def format_goal_metrics(m: DraftMetrics, cis: Optional[GoalMetricCIs] = None) -> str:
    """Format the six goal metrics with red/yellow/green status indicators."""
    lines: list[str] = []
    lines.append(colors.section("Goal Metrics:"))

    def _ci_fmt(ci_val: Optional[float], fmt: str = ".1f", suffix: str = "") -> str:
        if ci_val is None:
            return ""
        return f" {colors.dim(f'± {ci_val:{fmt}}{suffix}')}"

    def _status(
        value: float | None,
        green_thresh: float,
        yellow_thresh: float,
        direction: str = "gte",
    ) -> str:
        if value is None:
            return colors.dim("[N/A]")
        if direction == "gte":
            if value >= green_thresh:
                return colors.ok("[PASS]")
            if value >= yellow_thresh:
                return colors.warn("[WARN]")
            return colors.fail("[FAIL]")
        else:
            if value < green_thresh:
                return colors.ok("[PASS]")
            if value < yellow_thresh:
                return colors.warn("[WARN]")
            return colors.fail("[FAIL]")

    conv_mid = m.convergence_shown.on_plan_density_mid_mean
    ci_conv_mid = _ci_fmt(cis.convergence_mid if cis else None)
    lines.append(
        f"  {_status(conv_mid, 2.0, 1.5)}  "
        f"{colors.label('Convergence (mid):')}"
        f"  {colors.num(f'{conv_mid:.1f}')}{ci_conv_mid} "
        f"{colors.dim('(target: >= 2.0)')}"
    )

    conv_late = m.convergence_shown.on_plan_density_late_mean
    ci_conv_late = _ci_fmt(cis.convergence_late if cis else None)
    lines.append(
        f"  {_status(conv_late, 2.0, 1.5)}  "
        f"{colors.label('Convergence (late):')}"
        f" {colors.num(f'{conv_late:.1f}')}{ci_conv_late} "
        f"{colors.dim('(target: >= 2.0)')}"
    )

    near_opt = m.choice_richness_shown.near_optimal.overall
    ci_near_opt = _ci_fmt(cis.choice_richness if cis else None)
    lines.append(
        f"  {_status(near_opt, 1.5, 1.2)}  "
        f"{colors.label('Choice richness:')}"
        f"  {colors.num(f'{near_opt:.1f}')}{ci_near_opt} "
        f"{colors.dim('near-optimal (target: >= 1.5)')}"
    )

    force_val = m.forceability
    ci_force = _ci_fmt(cis.forceability if cis else None, ".2f")
    if force_val is not None:
        lines.append(
            f"  {_status(force_val, 0.95, 1.0, direction='lt')}  "
            f"{colors.label('Forceability:')}"
            f"     {colors.num(f'{force_val:.2f}')}{ci_force} "
            f"{colors.dim('(target: < 0.95)')}"
        )
    else:
        lines.append(
            f"  {colors.dim('[N/A]')}  "
            f"{colors.label('Forceability:')}"
            f"     {colors.dim('requires --runs N')}"
        )

    splash = m.splashability_shown.splash_fraction
    ci_splash = _ci_fmt(cis.splashability if cis else None, ".2f")
    lines.append(
        f"  {_status(splash, 0.40, 0.30)}  "
        f"{colors.label('Splashability:')}"
        f"    {colors.num(f'{splash:.2f}')}{ci_splash} "
        f"{colors.dim('(target: >= 0.40)')}"
    )

    openness = m.early_openness_shown.archetypes_exposed
    ci_openness = _ci_fmt(cis.early_openness if cis else None)
    lines.append(
        f"  {_status(openness, 5.0, 4.0)}  "
        f"{colors.label('Early openness:')}"
        f"   {colors.num(f'{openness:.1f}')}{ci_openness} "
        f"{colors.dim('archetypes (target: >= 5.0)')}"
    )

    sig_val = m.signal_benefit
    ci_sig = _ci_fmt(cis.signal_benefit if cis else None, ".1f", "%")
    if sig_val is not None:
        lines.append(
            f"  {_status(sig_val, 2.0, 0.0)}  "
            f"{colors.label('Signal benefit:')}"
            f"   {colors.num(f'{sig_val:+.1f}%')}{ci_sig} "
            f"{colors.dim('(target: >= 2%)')}"
        )
    else:
        lines.append(
            f"  {colors.dim('[N/A]')}  "
            f"{colors.label('Signal benefit:')}"
            f"   {colors.dim('requires --runs N')}"
        )

    return "\n".join(lines)


def format_metrics(m: DraftMetrics) -> str:
    """Format metrics as a human-readable text summary."""
    lines: list[str] = []

    def _phase_vals(label: str, ps: PhaseStats, fmt: str = ".1f") -> str:
        return (
            f"  {colors.label(label)}  "
            f"{colors.dim('early=')}{ colors.num(f'{ps.early:{fmt}}')}  "
            f"{colors.dim('mid=')}{ colors.num(f'{ps.mid:{fmt}}')}  "
            f"{colors.dim('late=')}{ colors.num(f'{ps.late:{fmt}}')}  "
            f"{colors.dim('overall=')}{ colors.num(f'{ps.overall:{fmt}}')}"
        )

    cr = m.choice_richness_shown
    lines.append(colors.section("Choice Richness (shown-N):"))
    lines.append(_phase_vals("Near-optimal count:", cr.near_optimal, ".1f"))
    lines.append(_phase_vals("Score gap:          ", cr.score_gap_mean, ".2f"))
    lines.append(_phase_vals("Choice entropy:     ", cr.choice_entropy, ".2f"))

    crf = m.choice_richness_full
    lines.append("")
    lines.append(colors.section("Choice Richness (full-pack):"))
    lines.append(_phase_vals("Near-optimal count:", crf.near_optimal, ".1f"))
    lines.append(_phase_vals("Score gap:          ", crf.score_gap_mean, ".2f"))
    lines.append(_phase_vals("Choice entropy:     ", crf.choice_entropy, ".2f"))

    lines.append("")
    lines.append(colors.section("Convergence (shown-N, post-commitment):"))
    lines.append(
        f"  {colors.label('On-plan density (mid):')}  "
        f"{colors.dim('mean=')}{colors.num(f'{m.convergence_shown.on_plan_density_mid_mean:.1f}')}, "
        f"{colors.dim('P(>=3)=')}{colors.num(f'{m.convergence_shown.on_plan_prob_gte_3_mid:.2f}')}"
    )
    lines.append(
        f"  {colors.label('On-plan density (late):')} "
        f"{colors.dim('mean=')}{colors.num(f'{m.convergence_shown.on_plan_density_late_mean:.1f}')}, "
        f"{colors.dim('P(>=3)=')}{colors.num(f'{m.convergence_shown.on_plan_prob_gte_3_late:.2f}')}"
    )

    lines.append("")
    lines.append(colors.section("Convergence (full-pack, post-commitment):"))
    lines.append(
        f"  {colors.label('On-plan density (mid):')}  "
        f"{colors.dim('mean=')}{colors.num(f'{m.convergence_full.on_plan_density_mid_mean:.1f}')}, "
        f"{colors.dim('P(>=3)=')}{colors.num(f'{m.convergence_full.on_plan_prob_gte_3_mid:.2f}')}"
    )
    lines.append(
        f"  {colors.label('On-plan density (late):')} "
        f"{colors.dim('mean=')}{colors.num(f'{m.convergence_full.on_plan_density_late_mean:.1f}')}, "
        f"{colors.dim('P(>=3)=')}{colors.num(f'{m.convergence_full.on_plan_prob_gte_3_late:.2f}')}"
    )

    lines.append("")
    if m.signal_benefit is not None:
        lines.append(
            f"{colors.label('Signal Benefit:')} "
            f"{colors.num(f'{m.signal_benefit:+.1f}%')} "
            f"{colors.dim('(adaptive vs signal-ignorant)')}"
        )
    else:
        lines.append(
            f"{colors.label('Signal Benefit:')} {colors.dim('N/A (requires sweep)')}"
        )

    if m.forceability is not None and m.forceability_archetype is not None:
        lines.append(
            f"{colors.label('Forceability:')} "
            f"{colors.dim('max=')}{colors.num(f'{m.forceability:.2f}')} "
            f"{colors.dim(f'(archetype {m.forceability_archetype})')}"
        )
    else:
        lines.append(
            f"{colors.label('Forceability:')} {colors.dim('N/A (requires sweep)')}"
        )

    lines.append(
        f"{colors.label('Splashability:')} "
        f"{colors.num(f'{m.splashability_shown.splash_fraction:.2f}')}"
    )

    eo = m.early_openness_shown
    lines.append(
        f"{colors.label('Early Openness:')} "
        f"{colors.num(f'{eo.archetypes_exposed:.1f}')} "
        f"{colors.dim('archetypes exposed,')} "
        f"{colors.dim('preference entropy=')}"
        f"{colors.num(f'{eo.preference_entropy:.2f}')}"
    )

    ct = m.commitment_timing
    lines.append("")
    lines.append(colors.section("Commitment Timing:"))
    lines.append(
        f"  {colors.label('Mean pick:')}"
        f"    {colors.num(f'{ct.mean_commitment_pick:.1f}')}, "
        f"{colors.dim('median=')}{colors.num(f'{ct.median_commitment_pick:.1f}')}"
    )
    lines.append(
        f"  {colors.label('Uncommitted:')}"
        f"  {colors.num(f'{ct.uncommitted_rate:.2f}')}, "
        f"{colors.dim('by pick 5=')}{colors.num(f'{ct.by_pick_5_rate:.2f}')}"
    )

    return "\n".join(lines)


# ---------------------------------------------------------------------------
# Multi-run averaging
# ---------------------------------------------------------------------------


def _average_phase_stats(items: list[PhaseStats]) -> PhaseStats:
    """Mean of each field across a list of PhaseStats."""
    n = len(items)
    return PhaseStats(
        early=sum(ps.early for ps in items) / n,
        mid=sum(ps.mid for ps in items) / n,
        late=sum(ps.late for ps in items) / n,
        overall=sum(ps.overall for ps in items) / n,
    )


def _average_choice_richness(
    items: list[ChoiceRichnessMetrics],
) -> ChoiceRichnessMetrics:
    """Field-wise mean of ChoiceRichnessMetrics."""
    return ChoiceRichnessMetrics(
        near_optimal=_average_phase_stats([m.near_optimal for m in items]),
        score_gap_mean=_average_phase_stats([m.score_gap_mean for m in items]),
        score_gap_median=_average_phase_stats([m.score_gap_median for m in items]),
        score_gap_p90=_average_phase_stats([m.score_gap_p90 for m in items]),
        choice_entropy=_average_phase_stats([m.choice_entropy for m in items]),
    )


def _average_convergence(items: list[ConvergenceMetrics]) -> ConvergenceMetrics:
    """Field-wise mean of ConvergenceMetrics."""
    n = len(items)
    return ConvergenceMetrics(
        on_plan_density_mid_mean=sum(m.on_plan_density_mid_mean for m in items) / n,
        on_plan_prob_gte_3_mid=sum(m.on_plan_prob_gte_3_mid for m in items) / n,
        on_plan_density_late_mean=sum(m.on_plan_density_late_mean for m in items) / n,
        on_plan_prob_gte_3_late=sum(m.on_plan_prob_gte_3_late for m in items) / n,
    )


def _average_splashability(items: list[SplashabilityMetrics]) -> SplashabilityMetrics:
    """Field-wise mean of SplashabilityMetrics."""
    n = len(items)
    return SplashabilityMetrics(
        splash_fraction=sum(m.splash_fraction for m in items) / n,
    )


def _average_early_openness(items: list[EarlyOpennessMetrics]) -> EarlyOpennessMetrics:
    """Field-wise mean of EarlyOpennessMetrics."""
    n = len(items)
    return EarlyOpennessMetrics(
        archetypes_exposed=sum(m.archetypes_exposed for m in items) / n,
        preference_entropy=sum(m.preference_entropy for m in items) / n,
    )


def average_metrics(metrics_list: list[DraftMetrics]) -> DraftMetrics:
    """Average a list of DraftMetrics into a single DraftMetrics."""
    n = len(metrics_list)
    if n == 0:
        raise ValueError("Cannot average an empty metrics list")
    if n == 1:
        return metrics_list[0]

    ct_items = [m.commitment_timing for m in metrics_list]
    avg_commitment_timing = CommitmentTimingMetrics(
        mean_commitment_pick=sum(c.mean_commitment_pick for c in ct_items) / n,
        median_commitment_pick=sum(c.median_commitment_pick for c in ct_items) / n,
        uncommitted_rate=sum(c.uncommitted_rate for c in ct_items) / n,
        by_pick_5_rate=sum(c.by_pick_5_rate for c in ct_items) / n,
    )

    force_vals = [m.forceability for m in metrics_list if m.forceability is not None]
    signal_vals = [
        m.signal_benefit for m in metrics_list if m.signal_benefit is not None
    ]

    return DraftMetrics(
        choice_richness_full=_average_choice_richness(
            [m.choice_richness_full for m in metrics_list]
        ),
        choice_richness_shown=_average_choice_richness(
            [m.choice_richness_shown for m in metrics_list]
        ),
        convergence_full=_average_convergence(
            [m.convergence_full for m in metrics_list]
        ),
        convergence_shown=_average_convergence(
            [m.convergence_shown for m in metrics_list]
        ),
        splashability_full=_average_splashability(
            [m.splashability_full for m in metrics_list]
        ),
        splashability_shown=_average_splashability(
            [m.splashability_shown for m in metrics_list]
        ),
        early_openness_full=_average_early_openness(
            [m.early_openness_full for m in metrics_list]
        ),
        early_openness_shown=_average_early_openness(
            [m.early_openness_shown for m in metrics_list]
        ),
        commitment_timing=avg_commitment_timing,
        forceability=_mean(force_vals) if force_vals else None,
        signal_benefit=_mean(signal_vals) if signal_vals else None,
    )


def compute_goal_cis(
    metrics_list: list[DraftMetrics],
    per_run_signal_benefits: Optional[list[float]] = None,
    per_run_forceabilities: Optional[list[float]] = None,
) -> GoalMetricCIs:
    """Compute 95% CI half-widths for each goal metric from per-run values."""
    if len(metrics_list) < 2:
        return GoalMetricCIs()

    conv_mid_vals = [m.convergence_shown.on_plan_density_mid_mean for m in metrics_list]
    conv_late_vals = [
        m.convergence_shown.on_plan_density_late_mean for m in metrics_list
    ]
    near_opt_vals = [m.choice_richness_shown.near_optimal.overall for m in metrics_list]
    splash_vals = [m.splashability_shown.splash_fraction for m in metrics_list]
    openness_vals = [m.early_openness_shown.archetypes_exposed for m in metrics_list]

    return GoalMetricCIs(
        convergence_mid=_ci_95(conv_mid_vals),
        convergence_late=_ci_95(conv_late_vals),
        choice_richness=_ci_95(near_opt_vals),
        forceability=(
            _ci_95(per_run_forceabilities)
            if per_run_forceabilities and len(per_run_forceabilities) >= 2
            else None
        ),
        splashability=_ci_95(splash_vals),
        early_openness=_ci_95(openness_vals),
        signal_benefit=(
            _ci_95(per_run_signal_benefits)
            if per_run_signal_benefits and len(per_run_signal_benefits) >= 2
            else None
        ),
    )
