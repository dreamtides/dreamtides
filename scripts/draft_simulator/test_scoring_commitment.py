"""Tests for deck scoring and commitment detection modules.

Covers edge conditions, normalization invariants, scoring calibration,
and commitment detection boundary cases. Stdlib-only, no external
dependencies.
"""

import math
import unittest

import commitment
import deck_scorer
from config import CommitmentConfig, ScoringConfig
from draft_models import CardDesign, CardInstance


class TestDeckScorer(unittest.TestCase):
    """Tests for deck_scorer module."""

    def _make_card(self, fitness: list[float], power: float = 0.5) -> CardDesign:
        return CardDesign(
            card_id="test",
            name="Test Card",
            fitness=fitness,
            power=power,
            commit=0.5,
            flex=0.5,
        )

    def test_empty_pool_returns_zero(self) -> None:
        scoring = ScoringConfig()
        self.assertEqual(deck_scorer.deck_value([], [1.0, 0.0], scoring), 0.0)

    def test_empty_w_returns_zero(self) -> None:
        scoring = ScoringConfig()
        card = self._make_card([0.5, 0.5])
        self.assertEqual(deck_scorer.deck_value([card], [], scoring), 0.0)

    def test_output_in_unit_range(self) -> None:
        scoring = ScoringConfig()
        card = self._make_card([1.0, 1.0, 1.0], power=1.0)
        w = [10.0, 0.0, 0.0]
        result = deck_scorer.deck_value([card], w, scoring)
        self.assertGreaterEqual(result, 0.0)
        self.assertLessEqual(result, 1.0)

    def test_deterministic(self) -> None:
        scoring = ScoringConfig()
        cards = [self._make_card([0.8, 0.2], power=0.6) for _ in range(10)]
        w = [2.0, 1.0]
        r1 = deck_scorer.deck_value(cards, w, scoring)
        r2 = deck_scorer.deck_value(cards, w, scoring)
        self.assertEqual(r1, r2)

    def test_archetype_coherence_clamped(self) -> None:
        """Coherence must not exceed 1.0 even with high secondary."""
        cards = [self._make_card([1.0, 1.0], power=0.5) for _ in range(5)]
        w = [2.0, 1.5]
        result = deck_scorer.archetype_coherence(cards, w, secondary_weight=0.3)
        self.assertLessEqual(result, 1.0)
        self.assertGreaterEqual(result, 0.0)

    def test_raw_power_mean(self) -> None:
        cards = [
            self._make_card([0.0], power=0.2),
            self._make_card([0.0], power=0.8),
        ]
        self.assertAlmostEqual(deck_scorer.raw_power(cards), 0.5)

    def test_focus_bonus_saturation(self) -> None:
        """All cards above threshold should give bonus 1.0 when >= saturation."""
        cards = [self._make_card([0.9], power=0.5) for _ in range(10)]
        w = [1.0]
        result = deck_scorer.focus_bonus(cards, w, threshold=0.5, saturation=0.7)
        self.assertAlmostEqual(result, 1.0)

    def test_accepts_card_instances(self) -> None:
        """deck_value should work with CardInstance objects."""
        scoring = ScoringConfig()
        design = self._make_card([0.7, 0.3], power=0.6)
        instances = [CardInstance(instance_id=i, design=design) for i in range(5)]
        result = deck_scorer.deck_value(instances, [1.0, 0.5], scoring)
        self.assertGreaterEqual(result, 0.0)
        self.assertLessEqual(result, 1.0)

    def test_card_instance_matches_design(self) -> None:
        """CardInstance pool should produce same score as CardDesign pool."""
        scoring = ScoringConfig()
        design = self._make_card([0.7, 0.3], power=0.6)
        designs = [design] * 5
        instances: list[CardInstance] = [
            CardInstance(instance_id=i, design=design) for i in range(5)
        ]
        w = [1.0, 0.5]
        score_designs = deck_scorer.deck_value(designs, w, scoring)
        score_instances = deck_scorer.deck_value(instances, w, scoring)
        self.assertAlmostEqual(score_designs, score_instances)

    def test_pool_from_instances(self) -> None:
        design = self._make_card([0.5])
        instances = [CardInstance(instance_id=0, design=design)]
        result = deck_scorer.pool_from_instances(instances)
        self.assertEqual(result, [design])


class TestCommitment(unittest.TestCase):
    """Tests for commitment detection module."""

    def test_empty_history_returns_none(self) -> None:
        cfg = CommitmentConfig()
        result = commitment.detect_commitment([], cfg)
        self.assertIsNone(result.commitment_pick)
        self.assertIsNone(result.entropy_commitment_pick)

    def test_uniform_returns_none(self) -> None:
        """Uniform preference vectors should never trigger commitment."""
        cfg = CommitmentConfig()
        history = [[0.125] * 8 for _ in range(30)]
        result = commitment.detect_commitment(history, cfg)
        self.assertIsNone(result.commitment_pick)

    def test_committed_pick_detected(self) -> None:
        """A strongly focused vector that stays stable should commit."""
        cfg = CommitmentConfig(
            commitment_threshold=0.35,
            stability_window=3,
        )
        # Build history: first 2 picks uniform, then heavily biased
        history: list[list[float]] = []
        for _ in range(2):
            history.append([0.125] * 8)
        for _ in range(10):
            w = [0.01] * 8
            w[0] = 5.0
            history.append(w)
        result = commitment.detect_commitment(history, cfg)
        self.assertIsNotNone(result.commitment_pick)
        self.assertEqual(result.committed_archetype, 0)

    def test_commitment_pick_is_earliest_stable(self) -> None:
        """Commitment should return the first pick that stays stable."""
        cfg = CommitmentConfig(
            commitment_threshold=0.35,
            stability_window=2,
        )
        # Pick 0: focused but changes at pick 1
        # Pick 2+: focused and stable
        history: list[list[float]] = []
        w_focused_0 = [0.01] * 4
        w_focused_0[0] = 5.0
        w_focused_1 = [0.01] * 4
        w_focused_1[1] = 5.0
        history.append(w_focused_0)  # pick 0: arch 0
        history.append(w_focused_1)  # pick 1: arch 1 (breaks stability)
        history.append(w_focused_1)  # pick 2: arch 1
        history.append(w_focused_1)  # pick 3: arch 1
        history.append(w_focused_1)  # pick 4: arch 1
        result = commitment.detect_commitment(history, cfg)
        self.assertEqual(result.commitment_pick, 1)
        self.assertEqual(result.committed_archetype, 1)

    def test_preference_vector_update(self) -> None:
        w = [0.125, 0.125]
        fitness = [0.5, 0.3]
        updated = commitment.update_preference_vector(w, fitness, 1.0)
        self.assertAlmostEqual(updated[0], 0.625)
        self.assertAlmostEqual(updated[1], 0.425)

    def test_preference_vector_not_renormalized(self) -> None:
        w = [0.5, 0.5]
        fitness = [1.0, 1.0]
        updated = commitment.update_preference_vector(w, fitness, 1.0)
        self.assertAlmostEqual(sum(updated), 3.0)

    def test_preference_vector_length_mismatch_raises(self) -> None:
        """Mismatched w and fitness lengths should raise ValueError."""
        w = [0.125, 0.125]
        fitness = [0.5, 0.3, 0.2]
        with self.assertRaises(ValueError):
            commitment.update_preference_vector(w, fitness, 1.0)

    def test_concentration_uniform(self) -> None:
        w = [0.125] * 8
        self.assertAlmostEqual(commitment.concentration(w), 0.125)

    def test_concentration_focused(self) -> None:
        w = [0.0] * 7 + [1.0]
        self.assertAlmostEqual(commitment.concentration(w), 1.0)

    def test_concentration_zero_sum(self) -> None:
        w = [0.0] * 4
        self.assertEqual(commitment.concentration(w), 0.0)

    def test_shannon_entropy_uniform(self) -> None:
        w = [1.0] * 8
        expected = math.log2(8)
        self.assertAlmostEqual(commitment.shannon_entropy(w), expected)

    def test_shannon_entropy_single(self) -> None:
        w = [0.0] * 7 + [1.0]
        self.assertAlmostEqual(commitment.shannon_entropy(w), 0.0)

    def test_entropy_commitment_detected(self) -> None:
        """Entropy-based commitment should be detected when entropy drops."""
        cfg = CommitmentConfig(
            entropy_threshold=2.0,
            stability_window=2,
        )
        history: list[list[float]] = []
        for _ in range(2):
            history.append([0.125] * 8)
        for _ in range(10):
            w = [0.001] * 8
            w[3] = 10.0
            history.append(w)
        result = commitment.detect_commitment(history, cfg)
        self.assertIsNotNone(result.entropy_commitment_pick)
        self.assertEqual(result.entropy_committed_archetype, 3)

    def test_initial_preference_vector(self) -> None:
        w = commitment.initial_preference_vector(4)
        self.assertEqual(len(w), 4)
        self.assertAlmostEqual(sum(w), 1.0)
        for v in w:
            self.assertAlmostEqual(v, 0.25)


if __name__ == "__main__":
    unittest.main()
