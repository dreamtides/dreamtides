"""Tests for refill strategies."""

import math
import random
import sys
from typing import Callable

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
    # Compute a simple archetype profile
    archetype_count = len(cards[0].design.fitness)
    profile = [0.0] * archetype_count
    for c in cards:
        for i in range(archetype_count):
            profile[i] += c.design.fitness[i]
    n = len(cards)
    profile = [v / n for v in profile]
    return Pack(pack_id="test_pack", cards=list(cards), archetype_profile=profile)


def test_no_refill_returns_none() -> None:
    """NoRefill should return None and not modify the pack."""
    designs = _make_test_designs()
    cube = _make_cube(designs)
    rng = random.Random(42)
    pack = _make_pack(cube, rng)
    original_size = len(pack.cards)

    result = refill.no_refill()
    assert result is None, f"Expected None, got {result}"
    # Pack should be unchanged
    assert len(pack.cards) == original_size
    print("  PASS: test_no_refill_returns_none")


def test_uniform_refill_returns_one_card() -> None:
    """UniformRefill should return exactly one CardInstance."""
    designs = _make_test_designs()
    cube = _make_cube(designs)
    rng = random.Random(42)

    result = refill.uniform_refill(cube, rng)
    assert isinstance(
        result, CardInstance
    ), f"Expected CardInstance, got {type(result)}"
    print("  PASS: test_uniform_refill_returns_one_card")


def test_constrained_refill_returns_one_card() -> None:
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
    assert isinstance(
        result, CardInstance
    ), f"Expected CardInstance, got {type(result)}"
    print("  PASS: test_constrained_refill_returns_one_card")


def test_constrained_refill_fidelity_zero_is_uniform() -> None:
    """At fidelity=0.0, all weights should be equal (no cosine influence)."""
    designs = _make_test_designs()
    cube = _make_cube(designs)

    signal = [1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]

    weights = refill.compute_constrained_weights(
        cube.supply, signal, fidelity=0.0, commit_bias=0.0
    )
    # All weights should be 1.0 (since (1 - 0) + 0 * sim = 1.0, then * (1 - 0) + 0 * c = 1.0)
    for i, w in enumerate(weights):
        assert (
            abs(w - 1.0) < 1e-6
        ), f"Weight at index {i} should be 1.0 at fidelity=0, got {w}"
    print("  PASS: test_constrained_refill_fidelity_zero_is_uniform")


def test_constrained_refill_fidelity_one_favors_similar() -> None:
    """At fidelity=1.0, weights should be dominated by cosine similarity."""
    # Create cards with known fitness: one very similar, one dissimilar
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
    # The similar card should have much higher weight
    assert (
        weights[0] > weights[1]
    ), f"Similar card weight ({weights[0]}) should be > dissimilar ({weights[1]})"
    print("  PASS: test_constrained_refill_fidelity_one_favors_similar")


def test_cosine_similarity_function() -> None:
    """Test the cosine similarity calculation directly."""
    a = [1.0, 0.0, 0.0]
    b = [1.0, 0.0, 0.0]
    sim = refill.cosine_similarity(a, b)
    assert abs(sim - 1.0) < 1e-6, f"Identical vectors should give sim=1.0, got {sim}"

    a = [1.0, 0.0, 0.0]
    b = [0.0, 1.0, 0.0]
    sim = refill.cosine_similarity(a, b)
    assert abs(sim - 0.0) < 1e-6, f"Orthogonal vectors should give sim=0.0, got {sim}"

    a = [1.0, 1.0, 0.0]
    b = [1.0, 0.0, 0.0]
    sim = refill.cosine_similarity(a, b)
    expected = 1.0 / math.sqrt(2.0)
    assert abs(sim - expected) < 1e-6, f"Expected {expected}, got {sim}"
    print("  PASS: test_cosine_similarity_function")


def test_compute_round_environment_profile() -> None:
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
    assert abs(profile[0] - 0.5) < 1e-6, f"Expected 0.5, got {profile[0]}"
    assert abs(profile[1] - 0.5) < 1e-6, f"Expected 0.5, got {profile[1]}"
    for i in range(2, 8):
        assert abs(profile[i] - 0.0) < 1e-6, f"Expected 0.0, got {profile[i]}"
    print("  PASS: test_compute_round_environment_profile")


def test_commit_bias_increases_weight_for_high_commit() -> None:
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
    assert (
        weights[0] > weights[1]
    ), f"High commit weight ({weights[0]}) should be > low commit ({weights[1]})"
    print("  PASS: test_commit_bias_increases_weight_for_high_commit")


def run_all_tests() -> None:
    """Run all tests and report results."""
    tests: list[tuple[str, Callable[[], None]]] = [
        ("test_no_refill_returns_none", test_no_refill_returns_none),
        ("test_uniform_refill_returns_one_card", test_uniform_refill_returns_one_card),
        (
            "test_constrained_refill_returns_one_card",
            test_constrained_refill_returns_one_card,
        ),
        (
            "test_constrained_refill_fidelity_zero_is_uniform",
            test_constrained_refill_fidelity_zero_is_uniform,
        ),
        (
            "test_constrained_refill_fidelity_one_favors_similar",
            test_constrained_refill_fidelity_one_favors_similar,
        ),
        ("test_cosine_similarity_function", test_cosine_similarity_function),
        (
            "test_compute_round_environment_profile",
            test_compute_round_environment_profile,
        ),
        (
            "test_commit_bias_increases_weight_for_high_commit",
            test_commit_bias_increases_weight_for_high_commit,
        ),
    ]
    passed = 0
    failed = 0
    for name, test_fn in tests:
        try:
            test_fn()
            passed += 1
        except Exception as e:
            print(f"  FAIL: {name}: {e}")
            failed += 1

    print(f"\n{passed}/{passed + failed} tests passed")
    if failed > 0:
        sys.exit(1)


if __name__ == "__main__":
    run_all_tests()
