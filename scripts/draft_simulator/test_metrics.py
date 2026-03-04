"""Tests for the metrics engine."""

import math
import sys

sys.path.insert(0, "scripts/draft_simulator")

import metrics


def test_softmax_basic() -> None:
    """Softmax of equal scores should produce uniform distribution."""
    scores = [1.0, 1.0, 1.0]
    result = metrics.softmax(scores, tau=1.0)
    assert len(result) == 3
    for p in result:
        assert abs(p - 1.0 / 3.0) < 1e-6


def test_softmax_temperature() -> None:
    """Higher temperature should produce more uniform distribution."""
    scores = [2.0, 1.0, 0.0]
    sharp = metrics.softmax(scores, tau=0.1)
    flat = metrics.softmax(scores, tau=10.0)
    # Sharp should have most weight on first element
    assert sharp[0] > flat[0]
    # Flat should be more uniform
    assert abs(flat[0] - flat[2]) < abs(sharp[0] - sharp[2])


def test_softmax_empty() -> None:
    """Softmax of empty list should return empty list."""
    assert metrics.softmax([], tau=1.0) == []


def test_softmax_single() -> None:
    """Softmax of single element should return [1.0]."""
    result = metrics.softmax([5.0], tau=1.0)
    assert len(result) == 1
    assert abs(result[0] - 1.0) < 1e-6


def test_shannon_entropy_uniform() -> None:
    """Shannon entropy of uniform distribution should be log2(n)."""
    probs = [0.25, 0.25, 0.25, 0.25]
    entropy = metrics.shannon_entropy(probs)
    assert abs(entropy - 2.0) < 1e-6


def test_shannon_entropy_certain() -> None:
    """Shannon entropy of a certain distribution should be 0."""
    probs = [1.0, 0.0, 0.0]
    entropy = metrics.shannon_entropy(probs)
    assert abs(entropy - 0.0) < 1e-6


def test_shannon_entropy_empty() -> None:
    """Shannon entropy of empty list should be 0."""
    assert metrics.shannon_entropy([]) == 0.0


def test_choice_entropy_from_scores() -> None:
    """Choice entropy should use softmax then Shannon entropy."""
    # Equal scores -> uniform softmax -> max entropy
    scores = [1.0, 1.0, 1.0, 1.0]
    entropy = metrics.choice_entropy(scores, tau=1.0)
    assert abs(entropy - 2.0) < 1e-6


def test_near_optimal_count() -> None:
    """Near-optimal count should count cards within gap of best score."""
    scores = [0.9, 0.85, 0.82, 0.7, 0.5]
    # gap=0.1 means within [0.9-0.1, 0.9] = [0.8, 0.9]
    count = metrics.near_optimal_count(scores, gap=0.1)
    assert count == 3  # 0.9, 0.85, 0.82


def test_near_optimal_count_all_same() -> None:
    """All same scores should all be near-optimal."""
    scores = [0.5, 0.5, 0.5]
    count = metrics.near_optimal_count(scores, gap=0.1)
    assert count == 3


def test_near_optimal_count_empty() -> None:
    """Empty scores should return 0."""
    assert metrics.near_optimal_count([], gap=0.1) == 0


def test_score_gap() -> None:
    """Score gap is difference between best and second-best."""
    scores = [0.9, 0.7, 0.5]
    gap = metrics.score_gap(scores)
    assert abs(gap - 0.2) < 1e-6


def test_score_gap_single() -> None:
    """Single score should have gap 0."""
    assert metrics.score_gap([0.5]) == 0.0


def test_score_gap_empty() -> None:
    """Empty scores should have gap 0."""
    assert metrics.score_gap([]) == 0.0


def test_pick_phase() -> None:
    """Phase classification by pick index."""
    assert metrics.pick_phase(0) == "early"
    assert metrics.pick_phase(5) == "early"
    assert metrics.pick_phase(6) == "mid"
    assert metrics.pick_phase(19) == "mid"
    assert metrics.pick_phase(20) == "late"
    assert metrics.pick_phase(29) == "late"


def test_forceability_computation() -> None:
    """Forceability should be max(mean_force / mean_adaptive)."""
    force_dvs: dict[int, list[float]] = {
        0: [0.7, 0.8],
        1: [0.6, 0.5],
    }
    adaptive_dvs = [0.8, 0.9]
    # arch 0: mean=0.75, adaptive mean=0.85 => 0.75/0.85=0.882
    # arch 1: mean=0.55, adaptive mean=0.85 => 0.55/0.85=0.647
    import config
    import draft_runner

    # Create minimal DraftResult (we only need forceability which uses
    # external data, not traces)
    dr = draft_runner.DraftResult(
        seat_results=[],
        traces=[],
        seed=42,
        card_pool={},
    )
    cfg = config.SimulatorConfig()
    m = metrics.compute_metrics(
        dr,
        cfg,
        force_deck_values=force_dvs,
        adaptive_deck_values=adaptive_dvs,
    )
    assert m.forceability is not None
    assert abs(m.forceability - 0.75 / 0.85) < 1e-6
    assert m.forceability_archetype == 0


def test_signal_benefit_computation() -> None:
    """Signal benefit should be percentage improvement."""
    import config
    import draft_runner

    dr = draft_runner.DraftResult(
        seat_results=[],
        traces=[],
        seed=42,
        card_pool={},
    )
    cfg = config.SimulatorConfig()
    aware = [0.8, 0.9]
    ignorant = [0.7, 0.8]
    m = metrics.compute_metrics(
        dr, cfg, aware_deck_values=aware, ignorant_deck_values=ignorant
    )
    # mean_aware=0.85, mean_ignorant=0.75
    # benefit = (0.85 - 0.75) / 0.75 * 100 = 13.33%
    assert m.signal_benefit is not None
    assert abs(m.signal_benefit - 13.333333) < 0.01


def test_signal_benefit_none_without_data() -> None:
    """Signal benefit should be None when data not provided."""
    import config
    import draft_runner

    dr = draft_runner.DraftResult(
        seat_results=[],
        traces=[],
        seed=42,
        card_pool={},
    )
    cfg = config.SimulatorConfig()
    m = metrics.compute_metrics(dr, cfg)
    assert m.signal_benefit is None
    assert m.forceability is None


def test_full_draft_metrics() -> None:
    """Smoke test: compute metrics on an actual draft run."""
    import config
    import draft_runner

    cfg = config.SimulatorConfig()
    result = draft_runner.run_draft(cfg, seed=42, trace_enabled=True)
    m = metrics.compute_metrics(result, cfg)

    # Choice richness should produce non-negative values
    assert m.choice_richness_shown.near_optimal.overall >= 1.0
    assert m.choice_richness_shown.choice_entropy.overall >= 0.0
    assert m.choice_richness_full.near_optimal.overall >= 1.0

    # Signal benefit and forceability should be None (single run)
    assert m.signal_benefit is None
    assert m.forceability is None

    # Splashability should be in [0, 1]
    assert 0.0 <= m.splashability_shown.splash_fraction <= 1.0
    assert 0.0 <= m.splashability_full.splash_fraction <= 1.0

    # Early openness should report positive values
    assert m.early_openness_shown.archetypes_exposed >= 0.0
    assert m.early_openness_shown.preference_entropy >= 0.0

    # Formatting should produce non-empty text
    text = metrics.format_metrics(m)
    assert "Choice Richness" in text
    assert "Convergence" in text
    assert "Signal Benefit" in text
    assert "Forceability" in text
    assert "Splashability" in text
    assert "Early Openness" in text
    assert "N/A (requires sweep)" in text


def test_format_metrics_with_sweep_data() -> None:
    """Format should show values when sweep data is provided."""
    import config
    import draft_runner

    dr = draft_runner.DraftResult(
        seat_results=[],
        traces=[],
        seed=42,
        card_pool={},
    )
    cfg = config.SimulatorConfig()
    m = metrics.compute_metrics(
        dr,
        cfg,
        force_deck_values={0: [0.7], 1: [0.5]},
        adaptive_deck_values=[0.8],
        aware_deck_values=[0.85],
        ignorant_deck_values=[0.75],
    )
    text = metrics.format_metrics(m)
    assert "N/A" not in text
    assert "Signal Benefit:" in text
    assert "Forceability:" in text


if __name__ == "__main__":
    test_softmax_basic()
    test_softmax_temperature()
    test_softmax_empty()
    test_softmax_single()
    test_shannon_entropy_uniform()
    test_shannon_entropy_certain()
    test_shannon_entropy_empty()
    test_choice_entropy_from_scores()
    test_near_optimal_count()
    test_near_optimal_count_all_same()
    test_near_optimal_count_empty()
    test_score_gap()
    test_score_gap_single()
    test_score_gap_empty()
    test_pick_phase()
    test_forceability_computation()
    test_signal_benefit_computation()
    test_signal_benefit_none_without_data()
    test_full_draft_metrics()
    test_format_metrics_with_sweep_data()
    print("All metrics tests passed!")
