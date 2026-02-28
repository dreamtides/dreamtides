"""Tests for sites_purge module."""

import random
from typing import Optional
from unittest.mock import patch

from models import (
    Card,
    CardType,
    DeckCard,
    PoolEntry,
    Rarity,
    Resonance,
)
from quest_state import QuestState


def _make_card(
    name: str,
    card_number: int,
    rarity: Rarity = Rarity.COMMON,
    resonances: Optional[frozenset[Resonance]] = None,
    energy_cost: int = 2,
    spark: Optional[int] = 1,
    tags: Optional[frozenset[str]] = None,
) -> Card:
    return Card(
        name=name,
        card_number=card_number,
        energy_cost=energy_cost,
        card_type=CardType.CHARACTER,
        subtype=None,
        is_fast=False,
        spark=spark,
        rarity=rarity,
        rules_text=f"Rules for {name}.",
        resonances=resonances or frozenset(),
        tags=tags or frozenset(),
    )


def _make_test_cards() -> list[Card]:
    """Create a set of test cards spanning rarities and resonances."""
    return [
        _make_card("Tide Card A", 1, Rarity.COMMON, frozenset({Resonance.TIDE})),
        _make_card("Tide Card B", 2, Rarity.COMMON, frozenset({Resonance.TIDE})),
        _make_card("Ember Card A", 3, Rarity.UNCOMMON, frozenset({Resonance.EMBER})),
        _make_card("Ember Card B", 4, Rarity.UNCOMMON, frozenset({Resonance.EMBER})),
        _make_card("Stone Card A", 5, Rarity.RARE, frozenset({Resonance.STONE})),
        _make_card("Zephyr Card A", 6, Rarity.COMMON, frozenset({Resonance.ZEPHYR})),
        _make_card("Ruin Card A", 7, Rarity.COMMON, frozenset({Resonance.RUIN})),
        _make_card("Neutral Card A", 8, Rarity.COMMON, frozenset()),
        _make_card(
            "Dual Card A",
            9,
            Rarity.LEGENDARY,
            frozenset({Resonance.TIDE, Resonance.RUIN}),
        ),
        _make_card("Stone Card B", 10, Rarity.UNCOMMON, frozenset({Resonance.STONE})),
    ]


def _make_pool(cards: list[Card]) -> list[PoolEntry]:
    """Create a simple pool with 1 entry per card."""
    return [PoolEntry(card) for card in cards]


def _make_quest_state(
    cards: Optional[list[Card]] = None,
    pool: Optional[list[PoolEntry]] = None,
    seed: int = 42,
    essence: int = 250,
    max_deck: int = 50,
) -> QuestState:
    test_cards = cards or _make_test_cards()
    test_pool = pool or _make_pool(test_cards)
    rng = random.Random(seed)
    variance = {r: 1.0 for r in Resonance}
    return QuestState(
        essence=essence,
        pool=test_pool,
        rng=rng,
        all_cards=test_cards,
        pool_variance=variance,
        max_deck=max_deck,
    )


def _populate_deck(state: QuestState, count: int) -> None:
    """Add cards to the deck until it has `count` cards."""
    cards = _make_test_cards()
    for i in range(count):
        card = cards[i % len(cards)]
        state.add_card(card)


class TestRunPurge:
    """Tests for voluntary purge site interaction."""

    def test_purge_removes_selected_cards(self) -> None:
        """Selected cards should be removed from the deck."""
        from sites_purge import run_purge

        state = _make_quest_state()
        _populate_deck(state, 10)
        initial_deck_size = state.deck_count()

        # Select first two cards for purging (indices 0 and 1)
        with patch("sites_purge.input_handler.multi_select", return_value=[0, 1]):
            run_purge(
                state=state,
                dreamscape_name="Test Dreamscape",
                dreamscape_number=1,
                logger=None,
            )

        assert state.deck_count() == initial_deck_size - 2

    def test_purge_updates_resonance_profile(self) -> None:
        """Removing a card should decrement resonance profile counts."""
        from sites_purge import run_purge

        cards = [
            _make_card("Tide Card", 1, Rarity.COMMON, frozenset({Resonance.TIDE})),
        ]
        state = _make_quest_state(cards)
        state.add_card(cards[0])
        assert state.resonance_profile.counts[Resonance.TIDE] == 1

        # Select index 0 to purge
        with patch("sites_purge.input_handler.multi_select", return_value=[0]):
            run_purge(
                state=state,
                dreamscape_name="Test Dreamscape",
                dreamscape_number=1,
                logger=None,
            )

        assert state.resonance_profile.counts[Resonance.TIDE] == 0

    def test_purge_updates_tag_profile(self) -> None:
        """Removing a card should decrement tag profile counts."""
        from sites_purge import run_purge

        cards = [
            _make_card(
                "Tagged Card",
                1,
                Rarity.COMMON,
                frozenset({Resonance.TIDE}),
                tags=frozenset({"warrior", "fast"}),
            ),
        ]
        state = _make_quest_state(cards)
        state.add_card(cards[0])
        assert state.tag_profile.counts.get("warrior", 0) == 1

        with patch("sites_purge.input_handler.multi_select", return_value=[0]):
            run_purge(
                state=state,
                dreamscape_name="Test Dreamscape",
                dreamscape_number=1,
                logger=None,
            )

        assert state.tag_profile.counts.get("warrior", 0) == 0

    def test_purge_default_max_is_three(self) -> None:
        """Without enhancement, max selections should be 3."""
        from sites_purge import run_purge

        state = _make_quest_state()
        _populate_deck(state, 10)
        initial_deck_size = state.deck_count()

        # Try to select 5 cards, but multi_select should be called with max=3
        # So at most 3 should be removed even if we return 5 indices
        captured_kwargs: list[dict[str, object]] = []

        original_multi_select = None

        def mock_multi_select(options: list[str], **kwargs: object) -> list[int]:
            captured_kwargs.append(kwargs)
            return [0, 1, 2]

        with patch(
            "sites_purge.input_handler.multi_select",
            side_effect=mock_multi_select,
        ):
            run_purge(
                state=state,
                dreamscape_name="Test Dreamscape",
                dreamscape_number=1,
                logger=None,
            )

        assert state.deck_count() == initial_deck_size - 3
        # Verify max_selections was passed as 3
        assert captured_kwargs[0].get("max_selections") == 3

    def test_purge_enhanced_max_is_six(self) -> None:
        """When enhanced (Ashen biome), max selections should be 6."""
        from sites_purge import run_purge

        state = _make_quest_state()
        _populate_deck(state, 10)
        initial_deck_size = state.deck_count()

        captured_kwargs: list[dict[str, object]] = []

        def mock_multi_select(options: list[str], **kwargs: object) -> list[int]:
            captured_kwargs.append(kwargs)
            return [0, 1, 2, 3, 4, 5]

        with patch(
            "sites_purge.input_handler.multi_select",
            side_effect=mock_multi_select,
        ):
            run_purge(
                state=state,
                dreamscape_name="Test Dreamscape",
                dreamscape_number=1,
                logger=None,
                is_enhanced=True,
            )

        assert state.deck_count() == initial_deck_size - 6
        assert captured_kwargs[0].get("max_selections") == 6

    def test_purge_skip_removes_nothing(self) -> None:
        """Selecting no cards should not change the deck."""
        from sites_purge import run_purge

        state = _make_quest_state()
        _populate_deck(state, 10)
        initial_deck_size = state.deck_count()

        # Select nothing (empty list)
        with patch("sites_purge.input_handler.multi_select", return_value=[]):
            run_purge(
                state=state,
                dreamscape_name="Test Dreamscape",
                dreamscape_number=1,
                logger=None,
            )

        assert state.deck_count() == initial_deck_size

    def test_purge_does_not_return_cards_to_pool(self) -> None:
        """Purged cards should not be returned to the draft pool."""
        from sites_purge import run_purge

        state = _make_quest_state()
        _populate_deck(state, 10)
        initial_pool_size = len(state.pool)

        with patch("sites_purge.input_handler.multi_select", return_value=[0, 1]):
            run_purge(
                state=state,
                dreamscape_name="Test Dreamscape",
                dreamscape_number=1,
                logger=None,
            )

        # Pool size should be unchanged -- purged cards are permanently removed
        assert len(state.pool) == initial_pool_size

    def test_purge_empty_deck(self) -> None:
        """Purge with an empty deck should handle gracefully."""
        from sites_purge import run_purge

        state = _make_quest_state()
        assert state.deck_count() == 0

        with patch("sites_purge.input_handler.multi_select", return_value=[]):
            run_purge(
                state=state,
                dreamscape_name="Test Dreamscape",
                dreamscape_number=1,
                logger=None,
            )

        assert state.deck_count() == 0

    def test_purge_with_logger(self) -> None:
        """Purge should log the interaction via log_site_visit."""
        from sites_purge import run_purge

        state = _make_quest_state()
        _populate_deck(state, 5)

        log_calls: list[dict[str, object]] = []

        class FakeLogger:
            def log_site_visit(self, **kwargs: object) -> None:
                log_calls.append(dict(kwargs))

        with patch("sites_purge.input_handler.multi_select", return_value=[0, 1]):
            run_purge(
                state=state,
                dreamscape_name="Test Dreamscape",
                dreamscape_number=1,
                logger=FakeLogger(),  # type: ignore[arg-type]
            )

        assert len(log_calls) == 1
        assert log_calls[0]["site_type"] == "Purge"


class TestForcedDeckLimitPurge:
    """Tests for forced deck-limit purge before battles."""

    def test_forced_purge_not_triggered_under_limit(self) -> None:
        """When deck <= max_deck, forced purge should do nothing."""
        from sites_purge import forced_deck_limit_purge

        state = _make_quest_state(max_deck=50)
        _populate_deck(state, 50)
        assert state.deck_count() == 50

        # Should not call multi_select at all
        with patch(
            "sites_purge.input_handler.multi_select",
            return_value=[],
        ) as mock_ms:
            forced_deck_limit_purge(state=state, logger=None)

        mock_ms.assert_not_called()
        assert state.deck_count() == 50

    def test_forced_purge_removes_until_at_limit(self) -> None:
        """Forced purge should remove cards until deck <= max_deck."""
        from sites_purge import forced_deck_limit_purge

        state = _make_quest_state(max_deck=10)
        _populate_deck(state, 13)
        assert state.deck_count() == 13

        # Need to remove at least 3 cards (13 - 10 = 3)
        with patch(
            "sites_purge.input_handler._is_interactive",
            return_value=True,
        ), patch(
            "sites_purge.input_handler.multi_select",
            return_value=[0, 1, 2],
        ):
            forced_deck_limit_purge(state=state, logger=None)

        assert state.deck_count() <= 10

    def test_forced_purge_loops_until_satisfied(self) -> None:
        """If player doesn't remove enough, forced purge should loop."""
        from sites_purge import forced_deck_limit_purge

        state = _make_quest_state(max_deck=10)
        _populate_deck(state, 13)
        # Need to remove 3, but player only removes 1 each time

        call_count = [0]

        def mock_multi_select(options: list[str], **kwargs: object) -> list[int]:
            call_count[0] += 1
            return [0]  # Remove 1 card each time

        with patch(
            "sites_purge.input_handler._is_interactive",
            return_value=True,
        ), patch(
            "sites_purge.input_handler.multi_select",
            side_effect=mock_multi_select,
        ):
            forced_deck_limit_purge(state=state, logger=None)

        assert state.deck_count() <= 10
        assert call_count[0] == 3  # 3 iterations to remove 3 cards

    def test_forced_purge_auto_removes_non_interactive(self) -> None:
        """In non-interactive mode, forced purge auto-removes excess cards."""
        from sites_purge import forced_deck_limit_purge

        state = _make_quest_state(max_deck=10)
        _populate_deck(state, 13)
        assert state.deck_count() == 13

        # Non-interactive mode (default in tests) should auto-remove
        forced_deck_limit_purge(state=state, logger=None)

        assert state.deck_count() == 10

    def test_forced_purge_updates_profiles(self) -> None:
        """Forced purge should update resonance and tag profiles."""
        from sites_purge import forced_deck_limit_purge

        cards = [
            _make_card(
                "Tide Card",
                1,
                Rarity.COMMON,
                frozenset({Resonance.TIDE}),
                tags=frozenset({"warrior"}),
            ),
        ]
        state = _make_quest_state(cards, max_deck=2)
        for _ in range(4):
            state.add_card(cards[0])
        assert state.resonance_profile.counts[Resonance.TIDE] == 4

        # Remove 2 to get to limit of 2
        with patch(
            "sites_purge.input_handler._is_interactive",
            return_value=True,
        ), patch(
            "sites_purge.input_handler.multi_select",
            return_value=[0, 1],
        ):
            forced_deck_limit_purge(state=state, logger=None)

        assert state.deck_count() <= 2
        assert state.resonance_profile.counts[Resonance.TIDE] == 2

    def test_forced_purge_does_not_return_to_pool(self) -> None:
        """Forced purge should not return cards to the draft pool."""
        from sites_purge import forced_deck_limit_purge

        state = _make_quest_state(max_deck=5)
        _populate_deck(state, 8)
        initial_pool_size = len(state.pool)

        with patch(
            "sites_purge.input_handler._is_interactive",
            return_value=True,
        ), patch(
            "sites_purge.input_handler.multi_select",
            return_value=[0, 1, 2],
        ):
            forced_deck_limit_purge(state=state, logger=None)

        assert len(state.pool) == initial_pool_size

    def test_forced_purge_with_logger(self) -> None:
        """Forced purge should log the interaction."""
        from sites_purge import forced_deck_limit_purge

        state = _make_quest_state(max_deck=5)
        _populate_deck(state, 7)

        log_calls: list[dict[str, object]] = []

        class FakeLogger:
            def log_site_visit(self, **kwargs: object) -> None:
                log_calls.append(dict(kwargs))

        with patch(
            "sites_purge.input_handler.multi_select",
            return_value=[0, 1],
        ):
            forced_deck_limit_purge(state=state, logger=None)

        assert state.deck_count() <= 5

    def test_forced_purge_log_includes_dreamscape(self) -> None:
        """Forced purge JSONL log should include the dreamscape name."""
        from sites_purge import forced_deck_limit_purge

        state = _make_quest_state(max_deck=5)
        _populate_deck(state, 7)

        log_calls: list[dict[str, object]] = []

        class FakeLogger:
            def log_site_visit(self, **kwargs: object) -> None:
                log_calls.append(dict(kwargs))

        forced_deck_limit_purge(
            state=state,
            logger=FakeLogger(),  # type: ignore[arg-type]
            dreamscape_name="Twilight Grove",
        )

        assert state.deck_count() <= 5
        assert len(log_calls) == 1
        assert log_calls[0]["dreamscape"] == "Twilight Grove"
