"""Tests for quest simulator tag system module."""

import math
import random

from models import (
    Card,
    CardType,
    PoolEntry,
    Rarity,
    Resonance,
    TagProfile,
)


def _make_card(
    name: str = "Test Card",
    card_number: int = 1,
    tags: frozenset[str] = frozenset(),
    rarity: Rarity = Rarity.COMMON,
    resonances: frozenset[Resonance] = frozenset(),
) -> Card:
    return Card(
        name=name,
        card_number=card_number,
        energy_cost=3,
        card_type=CardType.CHARACTER,
        subtype=None,
        is_fast=False,
        spark=2,
        rarity=rarity,
        rules_text="Test rules",
        resonances=resonances,
        tags=tags,
    )


def _make_pool_entries(
    tag: str, count: int, start_number: int = 1
) -> list[PoolEntry]:
    """Create pool entries whose cards all have the given tag."""
    return [
        PoolEntry(
            _make_card(
                name=f"{tag} Card {i}",
                card_number=start_number + i,
                tags=frozenset({tag}),
            )
        )
        for i in range(count)
    ]


class TestFilterPoolByTag:
    def test_returns_only_matching_entries(self) -> None:
        from tags import filter_pool_by_tag

        matching = _make_pool_entries("tribal:warrior", 3, start_number=1)
        non_matching = [
            PoolEntry(
                _make_card(
                    name="Other",
                    card_number=100,
                    tags=frozenset({"tribal:mage"}),
                )
            )
        ]
        pool = matching + non_matching
        result = filter_pool_by_tag(pool, "tribal:warrior")
        assert len(result) == 3
        for entry in result:
            assert "tribal:warrior" in entry.card.tags

    def test_empty_pool_returns_empty(self) -> None:
        from tags import filter_pool_by_tag

        result = filter_pool_by_tag([], "tribal:warrior")
        assert result == []

    def test_no_matches_returns_empty(self) -> None:
        from tags import filter_pool_by_tag

        pool = _make_pool_entries("tribal:mage", 5)
        result = filter_pool_by_tag(pool, "tribal:warrior")
        assert result == []

    def test_card_with_multiple_tags_matches_each(self) -> None:
        from tags import filter_pool_by_tag

        card = _make_card(
            name="Multi",
            card_number=1,
            tags=frozenset({"tribal:warrior", "mechanic:reclaim"}),
        )
        pool = [PoolEntry(card)]
        assert len(filter_pool_by_tag(pool, "tribal:warrior")) == 1
        assert len(filter_pool_by_tag(pool, "mechanic:reclaim")) == 1
        assert len(filter_pool_by_tag(pool, "role:finisher")) == 0


class TestSelectTheme:
    def test_selects_from_eligible_tags(self) -> None:
        from tags import select_theme

        # Create pool with two tags, each with enough cards
        pool = (
            _make_pool_entries("tribal:warrior", 8, start_number=1)
            + _make_pool_entries("mechanic:reclaim", 8, start_number=100)
        )
        profile = TagProfile()
        profile.add("tribal:warrior", 5)
        profile.add("mechanic:reclaim", 3)
        rng = random.Random(42)

        result = select_theme(
            pool=pool,
            profile=profile,
            rng=rng,
            min_theme_cards=6,
            tag_scale=1.5,
            relevance_boost=2.0,
            depth_factor=0.1,
        )
        assert result in ("tribal:warrior", "mechanic:reclaim")

    def test_excludes_tags_below_min_threshold(self) -> None:
        from tags import select_theme

        # "tribal:warrior" has 8 cards (eligible), "mechanic:reclaim" has 3 (not eligible)
        pool = (
            _make_pool_entries("tribal:warrior", 8, start_number=1)
            + _make_pool_entries("mechanic:reclaim", 3, start_number=100)
        )
        profile = TagProfile()
        profile.add("tribal:warrior", 5)
        profile.add("mechanic:reclaim", 10)  # High affinity but not enough cards
        rng = random.Random(42)

        result = select_theme(
            pool=pool,
            profile=profile,
            rng=rng,
            min_theme_cards=6,
            tag_scale=1.5,
            relevance_boost=2.0,
            depth_factor=0.1,
        )
        assert result == "tribal:warrior"

    def test_cold_start_uniform_random(self) -> None:
        """When all tag counts are 0, falls back to uniform random."""
        from tags import select_theme

        pool = (
            _make_pool_entries("tribal:warrior", 8, start_number=1)
            + _make_pool_entries("mechanic:reclaim", 8, start_number=100)
        )
        profile = TagProfile()  # Empty -- no counts
        rng = random.Random(42)

        result = select_theme(
            pool=pool,
            profile=profile,
            rng=rng,
            min_theme_cards=6,
            tag_scale=1.5,
            relevance_boost=2.0,
            depth_factor=0.1,
        )
        assert result in ("tribal:warrior", "mechanic:reclaim")

    def test_cold_start_selects_all_eligible_uniformly(self) -> None:
        """Over many runs, cold start should select each eligible tag."""
        from tags import select_theme

        pool = (
            _make_pool_entries("tag_a", 8, start_number=1)
            + _make_pool_entries("tag_b", 8, start_number=100)
            + _make_pool_entries("tag_c", 8, start_number=200)
        )
        profile = TagProfile()  # Empty

        selections: set[str] = set()
        for seed in range(100):
            rng = random.Random(seed)
            result = select_theme(
                pool=pool,
                profile=profile,
                rng=rng,
                min_theme_cards=6,
                tag_scale=1.5,
                relevance_boost=2.0,
                depth_factor=0.1,
            )
            selections.add(result)
        # All three tags should be selected at least once over 100 seeds
        assert selections == {"tag_a", "tag_b", "tag_c"}

    def test_returns_none_when_no_eligible_tags(self) -> None:
        from tags import select_theme

        # All tags have fewer than 6 cards
        pool = (
            _make_pool_entries("tribal:warrior", 3, start_number=1)
            + _make_pool_entries("mechanic:reclaim", 2, start_number=100)
        )
        profile = TagProfile()
        profile.add("tribal:warrior", 5)
        rng = random.Random(42)

        result = select_theme(
            pool=pool,
            profile=profile,
            rng=rng,
            min_theme_cards=6,
            tag_scale=1.5,
            relevance_boost=2.0,
            depth_factor=0.1,
        )
        assert result is None

    def test_returns_none_for_empty_pool(self) -> None:
        from tags import select_theme

        profile = TagProfile()
        rng = random.Random(42)

        result = select_theme(
            pool=[],
            profile=profile,
            rng=rng,
            min_theme_cards=6,
            tag_scale=1.5,
            relevance_boost=2.0,
            depth_factor=0.1,
        )
        assert result is None

    def test_weighted_selection_favors_higher_affinity(self) -> None:
        """Tags with higher affinity should be selected more often."""
        from tags import select_theme

        pool = (
            _make_pool_entries("high_affinity", 10, start_number=1)
            + _make_pool_entries("low_affinity", 10, start_number=100)
        )
        profile = TagProfile()
        profile.add("high_affinity", 20)
        profile.add("low_affinity", 1)

        counts: dict[str, int] = {"high_affinity": 0, "low_affinity": 0}
        for seed in range(200):
            rng = random.Random(seed)
            result = select_theme(
                pool=pool,
                profile=profile,
                rng=rng,
                min_theme_cards=6,
                tag_scale=1.5,
                relevance_boost=2.0,
                depth_factor=0.1,
            )
            assert result is not None
            counts[result] += 1
        # High affinity should be selected significantly more often
        assert counts["high_affinity"] > counts["low_affinity"]

    def test_theme_score_computation(self) -> None:
        """Verify the score formula produces expected values."""
        from tags import compute_theme_score

        # tag_affinity = ln(1 + 5) * 1.5 = ln(6) * 1.5
        # pool_depth = 10
        # theme_score = (ln(6) * 1.5) * 2.0 + 10 * 0.1 = ln(6)*3.0 + 1.0
        expected_affinity = math.log(1 + 5) * 1.5
        expected_score = expected_affinity * 2.0 + 10 * 0.1

        result = compute_theme_score(
            tag_count=5,
            pool_depth=10,
            tag_scale=1.5,
            relevance_boost=2.0,
            depth_factor=0.1,
        )
        assert abs(result - expected_score) < 1e-9

    def test_theme_score_zero_count(self) -> None:
        """Zero tag count gives zero affinity, score is purely depth-based."""
        from tags import compute_theme_score

        # tag_affinity = ln(1 + 0) * 1.5 = 0
        # theme_score = 0 * 2.0 + 8 * 0.1 = 0.8
        result = compute_theme_score(
            tag_count=0,
            pool_depth=8,
            tag_scale=1.5,
            relevance_boost=2.0,
            depth_factor=0.1,
        )
        assert abs(result - 0.8) < 1e-9

    def test_single_eligible_tag_always_selected(self) -> None:
        from tags import select_theme

        pool = _make_pool_entries("only_tag", 10, start_number=1)
        profile = TagProfile()
        profile.add("only_tag", 3)
        rng = random.Random(42)

        result = select_theme(
            pool=pool,
            profile=profile,
            rng=rng,
            min_theme_cards=6,
            tag_scale=1.5,
            relevance_boost=2.0,
            depth_factor=0.1,
        )
        assert result == "only_tag"

    def test_pool_depth_counts_entries_not_unique_cards(self) -> None:
        """Pool depth should count pool entries, not unique card numbers."""
        from tags import compute_theme_score, select_theme

        # Create 3 unique cards but 9 pool entries (3 copies each)
        card = _make_card(
            name="Warrior 1", card_number=1, tags=frozenset({"tribal:warrior"})
        )
        pool = [PoolEntry(card) for _ in range(9)]
        profile = TagProfile()
        profile.add("tribal:warrior", 2)
        rng = random.Random(42)

        result = select_theme(
            pool=pool,
            profile=profile,
            rng=rng,
            min_theme_cards=6,
            tag_scale=1.5,
            relevance_boost=2.0,
            depth_factor=0.1,
        )
        assert result == "tribal:warrior"

    def test_multi_tag_cards_produce_deterministic_results(self) -> None:
        """Cards with multiple tags should produce deterministic selection."""
        from tags import select_theme

        # Cards with multiple tags -- frozenset iteration order is
        # hash-seed dependent, so this test verifies that select_theme
        # produces deterministic results regardless of set iteration order.
        multi_tag_cards = [
            PoolEntry(
                _make_card(
                    name=f"Multi {i}",
                    card_number=i,
                    tags=frozenset({"tribal:warrior", "mechanic:reclaim", "role:finisher"}),
                )
            )
            for i in range(10)
        ]
        pool = multi_tag_cards
        profile = TagProfile()
        profile.add("tribal:warrior", 5)
        profile.add("mechanic:reclaim", 3)
        profile.add("role:finisher", 1)

        # Run multiple times with the same seed and verify identical results
        results = []
        for _ in range(10):
            rng = random.Random(42)
            result = select_theme(
                pool=pool,
                profile=profile,
                rng=rng,
                min_theme_cards=6,
                tag_scale=1.5,
                relevance_boost=2.0,
                depth_factor=0.1,
            )
            results.append(result)
        assert len(set(results)) == 1, f"Non-deterministic results: {results}"

    def test_zero_weight_fallback_to_uniform(self) -> None:
        """When all scores are zero, falls back to uniform random."""
        from tags import select_theme

        pool = (
            _make_pool_entries("tag_a", 8, start_number=1)
            + _make_pool_entries("tag_b", 8, start_number=100)
        )
        profile = TagProfile()
        profile.add("tag_a", 1)  # Non-zero so has_affinity is True

        rng = random.Random(42)
        # With tag_scale=0, relevance_boost=0, depth_factor=0 all scores are 0
        result = select_theme(
            pool=pool,
            profile=profile,
            rng=rng,
            min_theme_cards=6,
            tag_scale=0.0,
            relevance_boost=0.0,
            depth_factor=0.0,
        )
        assert result in ("tag_a", "tag_b")
