"""Tests for refill strategies.

Covers all three refill strategies (no_refill, uniform_refill,
constrained_refill), cosine similarity, round environment profile
computation, commit bias, and empty-signal edge cases. Stdlib-only,
no external dependencies.
"""

import math
import random
import unittest

import card_generator
import config
import cube_manager
import refill
from draft_models import CardDesign, CardInstance, CubeConsumptionMode, Pack


def _make_test_designs(archetype_count: int = 8) -> list[CardDesign]:
    """Create a small set of card designs for testing."""
    cfg = config.SimulatorConfig()
    cfg.cards.archetype_count = archetype_count
    cfg.cube.distinct_cards = 30
    rng = random.Random(999)
    return card_generator.generate_cards(cfg, rng)


def _make_cube(
    designs: list[CardDesign],
    mode: CubeConsumptionMode = CubeConsumptionMode.WITH_REPLACEMENT,
) -> cube_manager.CubeManager:
    return cube_manager.CubeManager(
        designs=designs,
        copies_per_card=1,
        consumption_mode=mode,
    )


def _make_pack(
    cube: cube_manager.CubeManager, rng: random.Random, pack_size: int = 10
) -> Pack:
    cards = cube.draw(pack_size, rng)
    archetype_count = len(cards[0].design.fitness)
    profile = [0.0] * archetype_count
    for c in cards:
        for i in range(archetype_count):
            profile[i] += c.design.fitness[i]
    n = len(cards)
    profile = [v / n for v in profile]
    return Pack(pack_id="test_pack", cards=list(cards), archetype_profile=profile)


class TestRefillStrategies(unittest.TestCase):
    """Tests for refill strategy functions."""

    def test_no_refill_returns_none(self) -> None:
        """NoRefill should return None and not modify the pack."""
        designs = _make_test_designs()
        cube = _make_cube(designs)
        rng = random.Random(42)
        pack = _make_pack(cube, rng)
        original_size = len(pack.cards)

        result = refill.no_refill()
        self.assertIsNone(result)
        self.assertEqual(len(pack.cards), original_size)

    def test_uniform_refill_returns_one_card(self) -> None:
        """UniformRefill should return exactly one CardInstance."""
        designs = _make_test_designs()
        cube = _make_cube(designs)
        rng = random.Random(42)

        result = refill.uniform_refill(cube, rng)
        self.assertIsInstance(result, CardInstance)

    def test_constrained_refill_returns_one_card(self) -> None:
        """ConstrainedRefill should return exactly one CardInstance."""
        designs = _make_test_designs()
        cube = _make_cube(designs)
        rng = random.Random(42)
        signal = [0.5] * 8

        result = refill.constrained_refill(
            cube=cube,
            signal=signal,
            fidelity=0.7,
            commit_bias=0.3,
            rng=rng,
        )
        self.assertIsInstance(result, CardInstance)

    def test_constrained_refill_fidelity_zero_is_uniform(self) -> None:
        """At fidelity=0.0, all weights should be equal regardless of commit_bias."""
        designs = _make_test_designs()
        cube = _make_cube(designs)

        signal = [1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]

        # With commit_bias=0.0
        weights = refill.compute_constrained_weights(
            cube.supply, signal, fidelity=0.0, commit_bias=0.0
        )
        for i, w in enumerate(weights):
            self.assertAlmostEqual(
                w, 1.0, places=6, msg=f"Weight at index {i} should be 1.0"
            )

        # With commit_bias=0.3 (default), fidelity=0 should still be uniform
        weights_with_bias = refill.compute_constrained_weights(
            cube.supply, signal, fidelity=0.0, commit_bias=0.3
        )
        for i, w in enumerate(weights_with_bias):
            self.assertAlmostEqual(
                w,
                1.0,
                places=6,
                msg=f"Weight at index {i} should be 1.0 even with commit_bias=0.3",
            )

    def test_constrained_refill_fidelity_one_favors_similar(self) -> None:
        """At fidelity=1.0, weights should be dominated by cosine similarity."""
        similar = CardDesign(
            card_id="sim",
            name="Similar",
            fitness=[0.9, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
            power=0.5,
            commit=0.5,
            flex=0.5,
        )
        dissimilar = CardDesign(
            card_id="dis",
            name="Dissimilar",
            fitness=[0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.9],
            power=0.5,
            commit=0.5,
            flex=0.5,
        )
        instances = [
            CardInstance(instance_id=0, design=similar),
            CardInstance(instance_id=1, design=dissimilar),
        ]
        signal = [1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]

        weights = refill.compute_constrained_weights(
            instances, signal, fidelity=1.0, commit_bias=0.0
        )
        self.assertGreater(weights[0], weights[1])

    def test_cosine_similarity_function(self) -> None:
        """Test the cosine similarity calculation directly."""
        a = [1.0, 0.0, 0.0]
        b = [1.0, 0.0, 0.0]
        sim = refill.cosine_similarity(a, b)
        self.assertAlmostEqual(sim, 1.0, places=6)

        a = [1.0, 0.0, 0.0]
        b = [0.0, 1.0, 0.0]
        sim = refill.cosine_similarity(a, b)
        self.assertAlmostEqual(sim, 0.0, places=6)

        a = [1.0, 1.0, 0.0]
        b = [1.0, 0.0, 0.0]
        sim = refill.cosine_similarity(a, b)
        expected = 1.0 / math.sqrt(2.0)
        self.assertAlmostEqual(sim, expected, places=6)

    def test_compute_round_environment_profile(self) -> None:
        """Test round environment profile is mean of all pack profiles."""
        packs = [
            Pack(
                pack_id="p1",
                cards=[],
                archetype_profile=[1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
            ),
            Pack(
                pack_id="p2",
                cards=[],
                archetype_profile=[0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
            ),
        ]
        profile = refill.compute_round_environment_profile(packs)
        self.assertAlmostEqual(profile[0], 0.5, places=6)
        self.assertAlmostEqual(profile[1], 0.5, places=6)
        for i in range(2, 8):
            self.assertAlmostEqual(profile[i], 0.0, places=6)

    def test_commit_bias_increases_weight_for_high_commit(self) -> None:
        """With commit_bias > 0, high-commit cards should get higher weights."""
        high_commit = CardDesign(
            card_id="hc",
            name="HighCommit",
            fitness=[0.5, 0.5, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
            power=0.5,
            commit=0.9,
            flex=0.5,
        )
        low_commit = CardDesign(
            card_id="lc",
            name="LowCommit",
            fitness=[0.5, 0.5, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
            power=0.5,
            commit=0.1,
            flex=0.5,
        )
        instances = [
            CardInstance(instance_id=0, design=high_commit),
            CardInstance(instance_id=1, design=low_commit),
        ]
        signal = [0.5, 0.5, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]

        weights = refill.compute_constrained_weights(
            instances, signal, fidelity=0.5, commit_bias=0.8
        )
        self.assertGreater(weights[0], weights[1])

    def test_empty_round_environment_profile(self) -> None:
        """Empty pack list should return empty profile."""
        profile = refill.compute_round_environment_profile([])
        self.assertEqual(profile, [])

    def test_constrained_refill_with_empty_signal_uses_floor_weights(self) -> None:
        """An empty signal vector should produce floor-weight sampling."""
        designs = _make_test_designs()
        cube = _make_cube(designs)

        weights = refill.compute_constrained_weights(
            cube.supply, signal=[], fidelity=0.7, commit_bias=0.0
        )
        for w in weights:
            self.assertGreater(w, 0.0, "All weights should be positive")

    def test_refill_adds_card_to_pack(self) -> None:
        """Uniform refill card should be appendable to a pack's card list."""
        designs = _make_test_designs()
        cube = _make_cube(designs)
        rng = random.Random(42)
        pack = _make_pack(cube, rng)
        original_size = len(pack.cards)

        card = refill.uniform_refill(cube, rng)
        pack.cards.append(card)
        self.assertEqual(len(pack.cards), original_size + 1)

    def test_constrained_refill_respects_fingerprint_source(self) -> None:
        """Constrained refill should accept either pack_origin or round_environment signal."""
        designs = _make_test_designs()
        cube = _make_cube(designs)
        rng = random.Random(42)
        pack = _make_pack(cube, rng)

        pack_signal = pack.archetype_profile
        card_pack = refill.constrained_refill(
            cube=cube,
            signal=pack_signal,
            fidelity=0.7,
            commit_bias=0.3,
            rng=rng,
        )
        self.assertIsInstance(card_pack, CardInstance)

        round_signal = refill.compute_round_environment_profile([pack])
        card_round = refill.constrained_refill(
            cube=cube,
            signal=round_signal,
            fidelity=0.7,
            commit_bias=0.3,
            rng=rng,
        )
        self.assertIsInstance(card_round, CardInstance)


if __name__ == "__main__":
    unittest.main()
