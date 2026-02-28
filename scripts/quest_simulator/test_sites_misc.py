"""Tests for sites_misc module (Duplication, Reward, Cleanse)."""

import random
from typing import Optional
from unittest.mock import patch

from models import (
    Card,
    CardType,
    DeckCard,
    Dreamsign,
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
    return [PoolEntry(card) for card in cards]


def _make_quest_state(
    cards: Optional[list[Card]] = None,
    pool: Optional[list[PoolEntry]] = None,
    seed: int = 42,
    essence: int = 250,
    completion_level: int = 0,
) -> QuestState:
    test_cards = cards or _make_test_cards()
    test_pool = pool or _make_pool(test_cards)
    rng = random.Random(seed)
    variance = {r: 1.0 for r in Resonance}
    state = QuestState(
        essence=essence,
        pool=test_pool,
        rng=rng,
        all_cards=test_cards,
        pool_variance=variance,
    )
    state.completion_level = completion_level
    return state


def _populate_deck(state: QuestState, count: int) -> None:
    cards = _make_test_cards()
    for i in range(count):
        card = cards[i % len(cards)]
        state.add_card(card)


def _make_bane_dreamsign() -> Dreamsign:
    return Dreamsign(
        name="Bane Sign",
        resonance=Resonance.RUIN,
        tags=frozenset(),
        effect_text="A bane dreamsign.",
        is_bane=True,
    )


def _make_normal_dreamsign(name: str = "Normal Sign") -> Dreamsign:
    return Dreamsign(
        name=name,
        resonance=Resonance.TIDE,
        tags=frozenset(),
        effect_text="A normal dreamsign.",
        is_bane=False,
    )


# ─── Duplication Tests ───────────────────────────────────────────────


class TestDuplicationSelectCandidates:
    """Tests for selecting duplication candidates from the deck."""

    def test_select_candidates_returns_three(self) -> None:
        """Normal duplication should select 3 random cards from deck."""
        from sites_misc import select_duplication_candidates

        state = _make_quest_state()
        _populate_deck(state, 10)

        candidates, copy_counts = select_duplication_candidates(
            state.deck, state.rng, enhanced=False
        )
        assert len(candidates) == 3
        assert len(copy_counts) == 3

    def test_select_candidates_copy_counts_in_range(self) -> None:
        """Copy counts should be between 1 and 4."""
        from sites_misc import select_duplication_candidates

        state = _make_quest_state()
        _populate_deck(state, 10)

        _candidates, copy_counts = select_duplication_candidates(
            state.deck, state.rng, enhanced=False
        )
        for count in copy_counts:
            assert 1 <= count <= 4

    def test_select_candidates_enhanced_returns_full_deck(self) -> None:
        """Enhanced duplication should return all deck cards."""
        from sites_misc import select_duplication_candidates

        state = _make_quest_state()
        _populate_deck(state, 5)

        candidates, copy_counts = select_duplication_candidates(
            state.deck, state.rng, enhanced=True
        )
        assert len(candidates) == 5
        # Enhanced: copy counts are generated after selection, should be length 1
        # Actually copy counts are generated per candidate in enhanced mode too
        assert len(copy_counts) == 5

    def test_select_candidates_small_deck(self) -> None:
        """With fewer than 3 cards, should return all available."""
        from sites_misc import select_duplication_candidates

        state = _make_quest_state()
        _populate_deck(state, 2)

        candidates, copy_counts = select_duplication_candidates(
            state.deck, state.rng, enhanced=False
        )
        assert len(candidates) == 2
        assert len(copy_counts) == 2


class TestRunDuplication:
    """Tests for the full duplication site interaction."""

    def test_duplication_adds_copies_to_deck(self) -> None:
        """Selecting a card should add the copy count number of copies."""
        from sites_misc import run_duplication

        state = _make_quest_state()
        _populate_deck(state, 10)
        initial_deck_size = state.deck_count()

        # Mock: select first candidate (index 0)
        with patch("sites_misc.input_handler.single_select", return_value=0), patch(
            "sites_misc.select_duplication_candidates",
            return_value=([state.deck[0]], [3]),
        ):
            run_duplication(
                state=state,
                dreamscape_name="Test Dreamscape",
                dreamscape_number=1,
                logger=None,
            )

        assert state.deck_count() == initial_deck_size + 3

    def test_duplication_skip_adds_nothing(self) -> None:
        """Skipping duplication should not change the deck."""
        from sites_misc import run_duplication

        state = _make_quest_state()
        _populate_deck(state, 10)
        initial_deck_size = state.deck_count()

        # Mock: select Skip option (last index = 3, since 3 candidates + 1 skip)
        with patch("sites_misc.input_handler.single_select", return_value=3):
            run_duplication(
                state=state,
                dreamscape_name="Test Dreamscape",
                dreamscape_number=1,
                logger=None,
            )

        assert state.deck_count() == initial_deck_size

    def test_duplication_updates_resonance_profile(self) -> None:
        """Adding copies should update the resonance profile."""
        from sites_misc import run_duplication

        cards = [
            _make_card("Tide Card", 1, Rarity.COMMON, frozenset({Resonance.TIDE})),
        ]
        state = _make_quest_state(cards)
        state.add_card(cards[0])
        assert state.resonance_profile.counts[Resonance.TIDE] == 1

        with patch("sites_misc.input_handler.single_select", return_value=0), patch(
            "sites_misc.select_duplication_candidates",
            return_value=([state.deck[0]], [2]),
        ):
            run_duplication(
                state=state,
                dreamscape_name="Test Dreamscape",
                dreamscape_number=1,
                logger=None,
            )

        assert state.resonance_profile.counts[Resonance.TIDE] == 3

    def test_duplication_with_logger(self) -> None:
        """Duplication should log the interaction."""
        from sites_misc import run_duplication

        state = _make_quest_state()
        _populate_deck(state, 5)

        log_calls: list[dict[str, object]] = []

        class FakeLogger:
            def log_site_visit(self, **kwargs: object) -> None:
                log_calls.append(dict(kwargs))

        with patch("sites_misc.input_handler.single_select", return_value=0):
            run_duplication(
                state=state,
                dreamscape_name="Test Dreamscape",
                dreamscape_number=1,
                logger=FakeLogger(),  # type: ignore[arg-type]
            )

        assert len(log_calls) == 1
        assert log_calls[0]["site_type"] == "Duplication"

    def test_duplication_empty_deck(self) -> None:
        """Duplication with an empty deck should handle gracefully."""
        from sites_misc import run_duplication

        state = _make_quest_state()
        assert state.deck_count() == 0

        run_duplication(
            state=state,
            dreamscape_name="Test Dreamscape",
            dreamscape_number=1,
            logger=None,
        )

        assert state.deck_count() == 0


# ─── Reward Site Tests ───────────────────────────────────────────────


class TestGenerateReward:
    """Tests for reward generation based on completion level."""

    def test_low_level_gives_essence(self) -> None:
        """Low completion levels should generate essence rewards."""
        from sites_misc import generate_reward

        rng = random.Random(42)
        reward = generate_reward(completion_level=0, rng=rng, all_cards=[])
        assert reward["type"] == "essence"
        assert 150 <= reward["value"] <= 250

    def test_mid_level_gives_card(self) -> None:
        """Mid completion levels should generate card rewards."""
        from sites_misc import generate_reward

        rng = random.Random(42)
        cards = _make_test_cards()
        reward = generate_reward(completion_level=3, rng=rng, all_cards=cards)
        assert reward["type"] == "card"
        assert reward["card"] is not None

    def test_high_level_gives_dreamsign(self) -> None:
        """High completion levels should generate dreamsign rewards."""
        from sites_misc import generate_reward

        rng = random.Random(42)
        all_dreamsigns = [_make_normal_dreamsign("Sign A")]
        reward = generate_reward(
            completion_level=5, rng=rng, all_cards=[], all_dreamsigns=all_dreamsigns
        )
        assert reward["type"] == "dreamsign"


class TestRunReward:
    """Tests for the full Reward site interaction."""

    def test_reward_accept_essence(self) -> None:
        """Accepting an essence reward should increase player essence."""
        from sites_misc import run_reward

        state = _make_quest_state(essence=100, completion_level=0)

        with patch(
            "sites_misc.input_handler.confirm_decline", return_value=True
        ), patch(
            "sites_misc.generate_reward",
            return_value={"type": "essence", "value": 200},
        ):
            run_reward(
                state=state,
                dreamscape_name="Test Dreamscape",
                dreamscape_number=1,
                logger=None,
                all_dreamsigns=[],
            )

        assert state.essence == 300

    def test_reward_decline_changes_nothing(self) -> None:
        """Declining a reward should not change state."""
        from sites_misc import run_reward

        state = _make_quest_state(essence=100, completion_level=0)

        with patch(
            "sites_misc.input_handler.confirm_decline", return_value=False
        ), patch(
            "sites_misc.generate_reward",
            return_value={"type": "essence", "value": 200},
        ):
            run_reward(
                state=state,
                dreamscape_name="Test Dreamscape",
                dreamscape_number=1,
                logger=None,
                all_dreamsigns=[],
            )

        assert state.essence == 100

    def test_reward_accept_card(self) -> None:
        """Accepting a card reward should add it to the deck."""
        from sites_misc import run_reward

        cards = _make_test_cards()
        state = _make_quest_state(cards, completion_level=3)
        initial_deck_size = state.deck_count()

        with patch(
            "sites_misc.input_handler.confirm_decline", return_value=True
        ), patch(
            "sites_misc.generate_reward",
            return_value={"type": "card", "value": 0, "card": cards[0]},
        ):
            run_reward(
                state=state,
                dreamscape_name="Test Dreamscape",
                dreamscape_number=1,
                logger=None,
                all_dreamsigns=[],
            )

        assert state.deck_count() == initial_deck_size + 1

    def test_reward_with_logger(self) -> None:
        """Reward site should log the interaction."""
        from sites_misc import run_reward

        state = _make_quest_state(essence=100, completion_level=0)

        log_calls: list[dict[str, object]] = []

        class FakeLogger:
            def log_site_visit(self, **kwargs: object) -> None:
                log_calls.append(dict(kwargs))

        with patch(
            "sites_misc.input_handler.confirm_decline", return_value=True
        ), patch(
            "sites_misc.generate_reward",
            return_value={"type": "essence", "value": 200},
        ):
            run_reward(
                state=state,
                dreamscape_name="Test Dreamscape",
                dreamscape_number=1,
                logger=FakeLogger(),  # type: ignore[arg-type]
                all_dreamsigns=[],
            )

        assert len(log_calls) == 1
        assert log_calls[0]["site_type"] == "RewardSite"


# ─── Cleanse Tests ───────────────────────────────────────────────────


class TestFindBanes:
    """Tests for identifying bane items in player's collection."""

    def test_finds_bane_cards(self) -> None:
        """Should identify bane cards in the deck."""
        from sites_misc import find_bane_items

        state = _make_quest_state()
        bane_card = _make_card("Bane Card", 99, Rarity.COMMON)
        state.deck.append(DeckCard(card=bane_card, is_bane=True))
        state.add_card(_make_card("Normal Card", 1, Rarity.COMMON))

        bane_deck_cards, bane_dreamsigns = find_bane_items(state)
        assert len(bane_deck_cards) == 1
        assert bane_deck_cards[0].card.name == "Bane Card"
        assert len(bane_dreamsigns) == 0

    def test_finds_bane_dreamsigns(self) -> None:
        """Should identify bane dreamsigns."""
        from sites_misc import find_bane_items

        state = _make_quest_state()
        bane_ds = _make_bane_dreamsign()
        state.dreamsigns.append(bane_ds)

        bane_deck_cards, bane_dreamsigns = find_bane_items(state)
        assert len(bane_deck_cards) == 0
        assert len(bane_dreamsigns) == 1
        assert bane_dreamsigns[0].name == "Bane Sign"

    def test_no_banes_returns_empty(self) -> None:
        """When no banes exist, both lists should be empty."""
        from sites_misc import find_bane_items

        state = _make_quest_state()
        _populate_deck(state, 5)
        state.dreamsigns.append(_make_normal_dreamsign())

        bane_deck_cards, bane_dreamsigns = find_bane_items(state)
        assert len(bane_deck_cards) == 0
        assert len(bane_dreamsigns) == 0


class TestRunCleanse:
    """Tests for the full Cleanse site interaction."""

    def test_cleanse_no_banes_auto_completes(self) -> None:
        """When no banes exist, cleanse should auto-complete."""
        from sites_misc import run_cleanse

        state = _make_quest_state()
        _populate_deck(state, 5)

        with patch("sites_misc.input_handler.wait_for_continue"):
            run_cleanse(
                state=state,
                dreamscape_name="Test Dreamscape",
                dreamscape_number=1,
                logger=None,
            )

        # Should complete without errors and deck unchanged
        assert state.deck_count() == 5

    def test_cleanse_removes_bane_cards(self) -> None:
        """Confirming cleanse should remove bane cards from deck."""
        from sites_misc import run_cleanse

        state = _make_quest_state()
        _populate_deck(state, 5)
        bane_card = _make_card("Bane Card", 99, Rarity.COMMON)
        state.deck.append(DeckCard(card=bane_card, is_bane=True))
        initial_deck_size = state.deck_count()

        with patch("sites_misc.input_handler.confirm_decline", return_value=True):
            run_cleanse(
                state=state,
                dreamscape_name="Test Dreamscape",
                dreamscape_number=1,
                logger=None,
            )

        assert state.deck_count() == initial_deck_size - 1
        assert all(not dc.is_bane for dc in state.deck)

    def test_cleanse_removes_bane_dreamsigns(self) -> None:
        """Confirming cleanse should remove bane dreamsigns."""
        from sites_misc import run_cleanse

        state = _make_quest_state()
        _populate_deck(state, 5)
        bane_ds = _make_bane_dreamsign()
        state.dreamsigns.append(bane_ds)

        with patch("sites_misc.input_handler.confirm_decline", return_value=True):
            run_cleanse(
                state=state,
                dreamscape_name="Test Dreamscape",
                dreamscape_number=1,
                logger=None,
            )

        assert len(state.dreamsigns) == 0

    def test_cleanse_decline_keeps_banes(self) -> None:
        """Declining cleanse should leave banes untouched."""
        from sites_misc import run_cleanse

        state = _make_quest_state()
        bane_card = _make_card("Bane Card", 99, Rarity.COMMON)
        state.deck.append(DeckCard(card=bane_card, is_bane=True))
        bane_ds = _make_bane_dreamsign()
        state.dreamsigns.append(bane_ds)

        with patch("sites_misc.input_handler.confirm_decline", return_value=False):
            run_cleanse(
                state=state,
                dreamscape_name="Test Dreamscape",
                dreamscape_number=1,
                logger=None,
            )

        # Banes should still be present
        assert any(dc.is_bane for dc in state.deck)
        assert any(ds.is_bane for ds in state.dreamsigns)

    def test_cleanse_limits_to_three_banes(self) -> None:
        """Cleanse should show at most 3 bane items."""
        from sites_misc import run_cleanse

        state = _make_quest_state()
        for i in range(5):
            bane_card = _make_card(f"Bane Card {i}", 90 + i, Rarity.COMMON)
            state.deck.append(DeckCard(card=bane_card, is_bane=True))

        with patch("sites_misc.input_handler.confirm_decline", return_value=True):
            run_cleanse(
                state=state,
                dreamscape_name="Test Dreamscape",
                dreamscape_number=1,
                logger=None,
            )

        # Should have removed at most 3 bane cards
        remaining_banes = [dc for dc in state.deck if dc.is_bane]
        assert len(remaining_banes) == 2  # 5 - 3 = 2

    def test_cleanse_with_logger(self) -> None:
        """Cleanse should log the interaction."""
        from sites_misc import run_cleanse

        state = _make_quest_state()
        bane_card = _make_card("Bane Card", 99, Rarity.COMMON)
        state.deck.append(DeckCard(card=bane_card, is_bane=True))

        log_calls: list[dict[str, object]] = []

        class FakeLogger:
            def log_site_visit(self, **kwargs: object) -> None:
                log_calls.append(dict(kwargs))

        with patch("sites_misc.input_handler.confirm_decline", return_value=True):
            run_cleanse(
                state=state,
                dreamscape_name="Test Dreamscape",
                dreamscape_number=1,
                logger=FakeLogger(),  # type: ignore[arg-type]
            )

        assert len(log_calls) == 1
        assert log_calls[0]["site_type"] == "Cleanse"
