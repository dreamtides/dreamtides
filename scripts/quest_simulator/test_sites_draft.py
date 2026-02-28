"""Tests for sites_draft module."""

import random
from typing import Optional
from unittest.mock import patch

from models import (
    AlgorithmParams,
    Card,
    CardType,
    DraftParams,
    PoolEntry,
    PoolParams,
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
        tags=frozenset(),
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


def _make_algorithm_params() -> AlgorithmParams:
    return AlgorithmParams(
        exponent=1.4,
        floor_weight=0.5,
        neutral_base=3.0,
        staleness_factor=0.3,
    )


def _make_draft_params(picks: int = 5, cards: int = 4) -> DraftParams:
    return DraftParams(
        cards_per_pick=cards,
        picks_per_site=picks,
    )


def _make_pool_params() -> PoolParams:
    return PoolParams(
        copies_common=4,
        copies_uncommon=3,
        copies_rare=2,
        copies_legendary=1,
        variance_min=0.75,
        variance_max=1.25,
    )


def _make_quest_state(
    cards: Optional[list[Card]] = None,
    pool: Optional[list[PoolEntry]] = None,
    seed: int = 42,
) -> QuestState:
    test_cards = cards or _make_test_cards()
    test_pool = pool or _make_pool(test_cards)
    rng = random.Random(seed)
    variance = {r: 1.0 for r in Resonance}
    return QuestState(
        essence=250,
        pool=test_pool,
        rng=rng,
        all_cards=test_cards,
        pool_variance=variance,
    )


class TestRunDraft:
    """Tests for run_draft function."""

    def test_draft_adds_cards_to_deck(self) -> None:
        """After a 2-pick draft, 2 cards should be in the deck."""
        from sites_draft import run_draft

        cards = _make_test_cards()
        state = _make_quest_state(cards)
        initial_pool_size = len(state.pool)
        params = _make_algorithm_params()
        draft_params = _make_draft_params(picks=2, cards=4)

        # Mock single_select to always pick index 0
        with patch("sites_draft.input_handler.single_select", return_value=0):
            run_draft(
                state=state,
                algorithm_params=params,
                draft_params=draft_params,
                pool_params=_make_pool_params(),
                dreamscape_name="Test Dreamscape",
                dreamscape_number=1,
                logger=None,
            )

        assert state.deck_count() == 2
        # Pool should have shrunk by 2 (one per pick)
        assert len(state.pool) == initial_pool_size - 2

    def test_draft_removes_picked_entry_from_pool(self) -> None:
        """The picked entry should be removed from the pool."""
        from sites_draft import run_draft

        cards = _make_test_cards()
        state = _make_quest_state(cards)
        params = _make_algorithm_params()
        draft_params = _make_draft_params(picks=1, cards=4)
        pool_before = list(state.pool)

        with patch("sites_draft.input_handler.single_select", return_value=0):
            run_draft(
                state=state,
                algorithm_params=params,
                draft_params=draft_params,
                pool_params=_make_pool_params(),
                dreamscape_name="Test Dreamscape",
                dreamscape_number=1,
                logger=None,
            )

        assert len(state.pool) == len(pool_before) - 1

    def test_draft_increments_staleness_on_unpicked(self) -> None:
        """Unpicked cards should have their pool entry staleness incremented."""
        from sites_draft import run_draft

        cards = _make_test_cards()
        pool = _make_pool(cards)
        state = _make_quest_state(cards, pool)
        params = _make_algorithm_params()
        draft_params = _make_draft_params(picks=1, cards=4)

        with patch("sites_draft.input_handler.single_select", return_value=0):
            run_draft(
                state=state,
                algorithm_params=params,
                draft_params=draft_params,
                pool_params=_make_pool_params(),
                dreamscape_name="Test Dreamscape",
                dreamscape_number=1,
                logger=None,
            )

        # Some entries that were offered but not picked should have staleness > 0
        stale_entries = [e for e in state.pool if e.staleness > 0]
        assert len(stale_entries) > 0

    def test_draft_respects_picks_per_site(self) -> None:
        """The number of picks should match draft_params.picks_per_site."""
        from sites_draft import run_draft

        cards = _make_test_cards()
        state = _make_quest_state(cards)
        params = _make_algorithm_params()
        draft_params = _make_draft_params(picks=3, cards=4)

        with patch("sites_draft.input_handler.single_select", return_value=0):
            run_draft(
                state=state,
                algorithm_params=params,
                draft_params=draft_params,
                pool_params=_make_pool_params(),
                dreamscape_name="Test Dreamscape",
                dreamscape_number=1,
                logger=None,
            )

        assert state.deck_count() == 3

    def test_draft_handles_small_pool(self) -> None:
        """When pool has fewer than cards_per_pick, offer what's available."""
        from sites_draft import run_draft

        cards = _make_test_cards()[:2]
        pool = _make_pool(cards)  # Only 2 entries
        state = _make_quest_state(cards, pool)
        params = _make_algorithm_params()
        draft_params = _make_draft_params(picks=1, cards=4)

        with patch("sites_draft.input_handler.single_select", return_value=0):
            run_draft(
                state=state,
                algorithm_params=params,
                draft_params=draft_params,
                pool_params=_make_pool_params(),
                dreamscape_name="Test Dreamscape",
                dreamscape_number=1,
                logger=None,
            )

        assert state.deck_count() == 1

    def test_draft_updates_resonance_profile(self) -> None:
        """Picking a card should update the resonance profile."""
        from sites_draft import run_draft

        cards = [
            _make_card("Tide Card", 1, Rarity.COMMON, frozenset({Resonance.TIDE})),
            _make_card("Ember Card", 2, Rarity.UNCOMMON, frozenset({Resonance.EMBER})),
            _make_card("Stone Card", 3, Rarity.RARE, frozenset({Resonance.STONE})),
            _make_card("Ruin Card", 4, Rarity.COMMON, frozenset({Resonance.RUIN})),
        ]
        pool = _make_pool(cards)
        state = _make_quest_state(cards, pool)
        params = _make_algorithm_params()
        draft_params = _make_draft_params(picks=1, cards=4)

        assert state.resonance_profile.total() == 0

        with patch("sites_draft.input_handler.single_select", return_value=0):
            run_draft(
                state=state,
                algorithm_params=params,
                draft_params=draft_params,
                pool_params=_make_pool_params(),
                dreamscape_name="Test Dreamscape",
                dreamscape_number=1,
                logger=None,
            )

        assert state.resonance_profile.total() > 0

    def test_draft_with_logger(self) -> None:
        """Draft should call log_draft_pick on the logger when provided."""
        from sites_draft import run_draft

        cards = _make_test_cards()
        state = _make_quest_state(cards)
        params = _make_algorithm_params()
        draft_params = _make_draft_params(picks=2, cards=4)

        log_calls: list[dict[str, object]] = []

        class FakeLogger:
            def log_draft_pick(
                self,
                offered_cards: list[Card],
                weights: list[float],
                picked_card: Card,
                profile_snapshot: dict[Resonance, int],
            ) -> None:
                log_calls.append(
                    {
                        "offered": list(offered_cards),
                        "weights": list(weights),
                        "picked": picked_card,
                        "profile": dict(profile_snapshot),
                    }
                )

            def log_site_visit(self, **kwargs: object) -> None:
                pass

        with patch("sites_draft.input_handler.single_select", return_value=0):
            run_draft(
                state=state,
                algorithm_params=params,
                draft_params=draft_params,
                pool_params=_make_pool_params(),
                dreamscape_name="Test Dreamscape",
                dreamscape_number=1,
                logger=FakeLogger(),  # type: ignore[arg-type]
            )

        assert len(log_calls) == 2
        for call in log_calls:
            assert len(call["offered"]) > 0  # type: ignore[arg-type]
            assert len(call["weights"]) > 0  # type: ignore[arg-type]
            assert call["picked"] is not None

    def test_draft_empty_pool_skips_gracefully(self) -> None:
        """When pool is completely empty, draft should handle gracefully."""
        from sites_draft import run_draft

        cards = _make_test_cards()
        state = _make_quest_state(cards, pool=[])
        params = _make_algorithm_params()
        draft_params = _make_draft_params(picks=1, cards=4)

        # Should not raise
        with patch("sites_draft.input_handler.single_select", return_value=0):
            run_draft(
                state=state,
                algorithm_params=params,
                draft_params=draft_params,
                pool_params=_make_pool_params(),
                dreamscape_name="Test Dreamscape",
                dreamscape_number=1,
                logger=None,
            )
