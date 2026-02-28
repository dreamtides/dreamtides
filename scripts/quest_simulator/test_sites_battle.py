"""Tests for sites_battle module."""

import random
from typing import Optional
from unittest.mock import patch

from models import (
    Boss,
    Card,
    CardType,
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


def _make_rare_cards() -> list[Card]:
    """Create rare and legendary cards for the pool."""
    return [
        _make_card("Rare A", 101, Rarity.RARE, frozenset({Resonance.TIDE})),
        _make_card("Rare B", 102, Rarity.RARE, frozenset({Resonance.EMBER})),
        _make_card("Rare C", 103, Rarity.RARE, frozenset({Resonance.STONE})),
        _make_card("Rare D", 104, Rarity.RARE, frozenset({Resonance.ZEPHYR})),
        _make_card("Legendary A", 105, Rarity.LEGENDARY, frozenset({Resonance.RUIN})),
    ]


def _make_mixed_cards() -> list[Card]:
    """Create a mix of common and rare cards."""
    return [
        _make_card("Common A", 1, Rarity.COMMON, frozenset({Resonance.TIDE})),
        _make_card("Common B", 2, Rarity.COMMON, frozenset({Resonance.EMBER})),
        _make_card("Uncommon A", 3, Rarity.UNCOMMON, frozenset({Resonance.STONE})),
        _make_card("Rare A", 101, Rarity.RARE, frozenset({Resonance.TIDE})),
        _make_card("Rare B", 102, Rarity.RARE, frozenset({Resonance.EMBER})),
        _make_card("Rare C", 103, Rarity.RARE, frozenset({Resonance.STONE})),
        _make_card("Rare D", 104, Rarity.RARE, frozenset({Resonance.ZEPHYR})),
        _make_card("Legendary A", 105, Rarity.LEGENDARY, frozenset({Resonance.RUIN})),
    ]


def _make_pool(cards: list[Card]) -> list[PoolEntry]:
    """Create a simple pool with 1 entry per card."""
    return [PoolEntry(card) for card in cards]


def _make_bosses() -> list[Boss]:
    """Create a set of test bosses."""
    return [
        Boss(
            name="Pyrra, Ember Dancer",
            archetype="Aggro/Burn",
            ability_text="Whenever you play a character that costs 2 or less, kindle 1.",
            deck_description="Cheap aggro deck.",
            is_final=False,
            resonances=frozenset({Resonance.EMBER}),
        ),
        Boss(
            name="Thornroot, Grove-Warden",
            archetype="Spirit Animal Tribal",
            ability_text="Spirit Animals increase energy by 1.",
            deck_description="Ramp deck.",
            is_final=False,
            resonances=frozenset({Resonance.STONE}),
        ),
        Boss(
            name="Nihil, the Silence Between",
            archetype="Draw-Go Control",
            ability_text="Whenever your opponent plays a card, draw a card.",
            deck_description="Control deck.",
            is_final=True,
            resonances=frozenset({Resonance.ZEPHYR, Resonance.RUIN}),
        ),
        Boss(
            name="Keth, the Binding Dark",
            archetype="Prison",
            ability_text="Cards cost 1 more.",
            deck_description="Prison deck.",
            is_final=True,
            resonances=frozenset({Resonance.RUIN}),
        ),
    ]


def _make_quest_state(
    cards: Optional[list[Card]] = None,
    pool: Optional[list[PoolEntry]] = None,
    seed: int = 42,
    essence: int = 250,
    completion_level: int = 0,
) -> QuestState:
    test_cards = cards or _make_mixed_cards()
    test_pool = pool if pool is not None else _make_pool(test_cards)
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


def _battle_config() -> dict[str, int]:
    return {
        "base_essence": 100,
        "per_level": 25,
        "rare_pick_count": 3,
    }


def _quest_config() -> dict[str, int]:
    return {
        "total_battles": 7,
        "miniboss_battle": 4,
    }


class TestDetermineOpponent:
    """Tests for opponent determination based on completion level."""

    def test_level_0_is_dream_guardian(self) -> None:
        from sites_battle import determine_opponent

        bosses = _make_bosses()
        rng = random.Random(42)
        quest_cfg = _quest_config()
        name, info = determine_opponent(0, bosses, rng, quest_cfg)
        assert name == "Dream Guardian"
        assert info is None

    def test_level_1_is_dream_guardian(self) -> None:
        from sites_battle import determine_opponent

        bosses = _make_bosses()
        rng = random.Random(42)
        quest_cfg = _quest_config()
        name, info = determine_opponent(1, bosses, rng, quest_cfg)
        assert name == "Dream Guardian"
        assert info is None

    def test_level_2_is_dream_guardian(self) -> None:
        from sites_battle import determine_opponent

        bosses = _make_bosses()
        rng = random.Random(42)
        quest_cfg = _quest_config()
        name, info = determine_opponent(2, bosses, rng, quest_cfg)
        assert name == "Dream Guardian"
        assert info is None

    def test_level_3_is_miniboss(self) -> None:
        """completion_level 3 == miniboss_battle - 1 => miniboss."""
        from sites_battle import determine_opponent

        bosses = _make_bosses()
        rng = random.Random(42)
        quest_cfg = _quest_config()
        name, info = determine_opponent(3, bosses, rng, quest_cfg)
        assert name != "Dream Guardian"
        assert info is not None
        assert info.is_final is False

    def test_level_4_is_dream_guardian(self) -> None:
        from sites_battle import determine_opponent

        bosses = _make_bosses()
        rng = random.Random(42)
        quest_cfg = _quest_config()
        name, info = determine_opponent(4, bosses, rng, quest_cfg)
        assert name == "Dream Guardian"
        assert info is None

    def test_level_5_is_dream_guardian(self) -> None:
        from sites_battle import determine_opponent

        bosses = _make_bosses()
        rng = random.Random(42)
        quest_cfg = _quest_config()
        name, info = determine_opponent(5, bosses, rng, quest_cfg)
        assert name == "Dream Guardian"
        assert info is None

    def test_level_6_is_final_boss(self) -> None:
        """completion_level 6 == total_battles - 1 => final boss."""
        from sites_battle import determine_opponent

        bosses = _make_bosses()
        rng = random.Random(42)
        quest_cfg = _quest_config()
        name, info = determine_opponent(6, bosses, rng, quest_cfg)
        assert name != "Dream Guardian"
        assert info is not None
        assert info.is_final is True

    def test_miniboss_is_from_boss_list(self) -> None:
        """The miniboss should be one of the non-final bosses."""
        from sites_battle import determine_opponent

        bosses = _make_bosses()
        rng = random.Random(42)
        quest_cfg = _quest_config()
        name, info = determine_opponent(3, bosses, rng, quest_cfg)
        non_final_names = [b.name for b in bosses if not b.is_final]
        assert name in non_final_names

    def test_final_boss_is_from_boss_list(self) -> None:
        """The final boss should be one of the is_final=True bosses."""
        from sites_battle import determine_opponent

        bosses = _make_bosses()
        rng = random.Random(42)
        quest_cfg = _quest_config()
        name, info = determine_opponent(6, bosses, rng, quest_cfg)
        final_names = [b.name for b in bosses if b.is_final]
        assert name in final_names


class TestComputeEssenceReward:
    """Tests for essence reward calculation."""

    def test_level_0_reward(self) -> None:
        from sites_battle import compute_essence_reward

        config = _battle_config()
        assert compute_essence_reward(0, config) == 100

    def test_level_3_reward(self) -> None:
        from sites_battle import compute_essence_reward

        config = _battle_config()
        assert compute_essence_reward(3, config) == 175

    def test_level_6_reward(self) -> None:
        from sites_battle import compute_essence_reward

        config = _battle_config()
        assert compute_essence_reward(6, config) == 250


class TestRunBattle:
    """Tests for the full run_battle interaction."""

    def test_battle_grants_essence(self) -> None:
        """After battle, essence should increase by the reward amount."""
        from sites_battle import run_battle

        from models import AlgorithmParams

        cards = _make_mixed_cards()
        state = _make_quest_state(cards, essence=250, completion_level=0)
        battle_cfg = _battle_config()
        quest_cfg = _quest_config()
        algo_params = AlgorithmParams(
            exponent=1.4,
            floor_weight=0.5,
            neutral_base=3.0,
            staleness_factor=0.3,
        )

        with patch("sites_battle.input_handler.wait_for_continue"), \
             patch("sites_battle.input_handler.single_select", return_value=0):
            run_battle(
                state=state,
                battle_config=battle_cfg,
                quest_config=quest_cfg,
                algorithm_params=algo_params,
                bosses=[],
                dreamscape_name="Test Dreamscape",
                dreamscape_number=1,
                logger=None,
            )

        # 250 + 100 (base) + 25 * 0 (level) = 350
        assert state.essence == 350

    def test_battle_increments_completion(self) -> None:
        """After battle, completion_level should be incremented by 1."""
        from sites_battle import run_battle

        from models import AlgorithmParams

        cards = _make_mixed_cards()
        state = _make_quest_state(cards, essence=250, completion_level=2)
        battle_cfg = _battle_config()
        quest_cfg = _quest_config()
        algo_params = AlgorithmParams(
            exponent=1.4,
            floor_weight=0.5,
            neutral_base=3.0,
            staleness_factor=0.3,
        )

        with patch("sites_battle.input_handler.wait_for_continue"), \
             patch("sites_battle.input_handler.single_select", return_value=0):
            run_battle(
                state=state,
                battle_config=battle_cfg,
                quest_config=quest_cfg,
                algorithm_params=algo_params,
                bosses=[],
                dreamscape_name="Test Dreamscape",
                dreamscape_number=1,
                logger=None,
            )

        assert state.completion_level == 3

    def test_battle_adds_rare_card_to_deck(self) -> None:
        """After battle, a rare card should be added to the deck."""
        from sites_battle import run_battle

        from models import AlgorithmParams

        rare_cards = _make_rare_cards()
        pool = _make_pool(rare_cards)
        cards = _make_mixed_cards()
        state = _make_quest_state(cards, pool=pool, essence=250, completion_level=0)
        battle_cfg = _battle_config()
        quest_cfg = _quest_config()
        algo_params = AlgorithmParams(
            exponent=1.4,
            floor_weight=0.5,
            neutral_base=3.0,
            staleness_factor=0.3,
        )

        with patch("sites_battle.input_handler.wait_for_continue"), \
             patch("sites_battle.input_handler.single_select", return_value=0):
            run_battle(
                state=state,
                battle_config=battle_cfg,
                quest_config=quest_cfg,
                algorithm_params=algo_params,
                bosses=[],
                dreamscape_name="Test Dreamscape",
                dreamscape_number=1,
                logger=None,
            )

        assert state.deck_count() == 1
        picked_card = state.deck[0].card
        assert picked_card.rarity in (Rarity.RARE, Rarity.LEGENDARY)

    def test_battle_removes_picked_from_pool(self) -> None:
        """After battle, the picked rare entry should be removed from pool."""
        from sites_battle import run_battle

        from models import AlgorithmParams

        rare_cards = _make_rare_cards()
        pool = _make_pool(rare_cards)
        initial_pool_size = len(pool)
        cards = _make_mixed_cards()
        state = _make_quest_state(cards, pool=pool, essence=250, completion_level=0)
        battle_cfg = _battle_config()
        quest_cfg = _quest_config()
        algo_params = AlgorithmParams(
            exponent=1.4,
            floor_weight=0.5,
            neutral_base=3.0,
            staleness_factor=0.3,
        )

        with patch("sites_battle.input_handler.wait_for_continue"), \
             patch("sites_battle.input_handler.single_select", return_value=0):
            run_battle(
                state=state,
                battle_config=battle_cfg,
                quest_config=quest_cfg,
                algorithm_params=algo_params,
                bosses=[],
                dreamscape_name="Test Dreamscape",
                dreamscape_number=1,
                logger=None,
            )

        assert len(state.pool) == initial_pool_size - 1

    def test_battle_no_rare_cards_skips_draft(self) -> None:
        """When pool has no rare+ cards, no card is added to deck."""
        from sites_battle import run_battle

        from models import AlgorithmParams

        common_cards = [
            _make_card("Common A", 1, Rarity.COMMON, frozenset({Resonance.TIDE})),
            _make_card("Common B", 2, Rarity.COMMON, frozenset({Resonance.EMBER})),
        ]
        pool = _make_pool(common_cards)
        state = _make_quest_state(common_cards, pool=pool, essence=250, completion_level=0)
        battle_cfg = _battle_config()
        quest_cfg = _quest_config()
        algo_params = AlgorithmParams(
            exponent=1.4,
            floor_weight=0.5,
            neutral_base=3.0,
            staleness_factor=0.3,
        )

        with patch("sites_battle.input_handler.wait_for_continue"), \
             patch("sites_battle.input_handler.single_select", return_value=0):
            run_battle(
                state=state,
                battle_config=battle_cfg,
                quest_config=quest_cfg,
                algorithm_params=algo_params,
                bosses=[],
                dreamscape_name="Test Dreamscape",
                dreamscape_number=1,
                logger=None,
            )

        assert state.deck_count() == 0
        # Should still increment completion and grant essence
        assert state.completion_level == 1
        assert state.essence == 350

    def test_battle_fewer_than_3_rares_offers_available(self) -> None:
        """When pool has fewer than 3 rare+ cards, offer what's available."""
        from sites_battle import run_battle

        from models import AlgorithmParams

        cards_list = [
            _make_card("Rare Only", 101, Rarity.RARE, frozenset({Resonance.TIDE})),
            _make_card("Common A", 1, Rarity.COMMON, frozenset({Resonance.EMBER})),
        ]
        pool = _make_pool(cards_list)
        state = _make_quest_state(cards_list, pool=pool, essence=250, completion_level=0)
        battle_cfg = _battle_config()
        quest_cfg = _quest_config()
        algo_params = AlgorithmParams(
            exponent=1.4,
            floor_weight=0.5,
            neutral_base=3.0,
            staleness_factor=0.3,
        )

        with patch("sites_battle.input_handler.wait_for_continue"), \
             patch("sites_battle.input_handler.single_select", return_value=0):
            run_battle(
                state=state,
                battle_config=battle_cfg,
                quest_config=quest_cfg,
                algorithm_params=algo_params,
                bosses=[],
                dreamscape_name="Test Dreamscape",
                dreamscape_number=1,
                logger=None,
            )

        # Should still pick the 1 available rare
        assert state.deck_count() == 1
        assert state.deck[0].card.name == "Rare Only"

    def test_battle_with_logger(self) -> None:
        """Battle should call log_battle_complete when logger provided."""
        from sites_battle import run_battle

        from models import AlgorithmParams

        rare_cards = _make_rare_cards()
        pool = _make_pool(rare_cards)
        cards = _make_mixed_cards()
        state = _make_quest_state(cards, pool=pool, essence=250, completion_level=0)
        battle_cfg = _battle_config()
        quest_cfg = _quest_config()
        algo_params = AlgorithmParams(
            exponent=1.4,
            floor_weight=0.5,
            neutral_base=3.0,
            staleness_factor=0.3,
        )

        log_calls: list[dict[str, object]] = []

        class FakeLogger:
            def log_battle_complete(
                self,
                opponent_name: str,
                essence_reward: int,
                rare_pick: Optional[Card],
            ) -> None:
                log_calls.append({
                    "opponent_name": opponent_name,
                    "essence_reward": essence_reward,
                    "rare_pick": rare_pick,
                })

        with patch("sites_battle.input_handler.wait_for_continue"), \
             patch("sites_battle.input_handler.single_select", return_value=0):
            run_battle(
                state=state,
                battle_config=battle_cfg,
                quest_config=quest_cfg,
                algorithm_params=algo_params,
                bosses=[],
                dreamscape_name="Test Dreamscape",
                dreamscape_number=1,
                logger=FakeLogger(),  # type: ignore[arg-type]
            )

        assert len(log_calls) == 1
        assert log_calls[0]["opponent_name"] == "Dream Guardian"
        assert log_calls[0]["essence_reward"] == 100
        assert log_calls[0]["rare_pick"] is not None

    def test_battle_with_miniboss(self) -> None:
        """At completion_level 3, should face a miniboss."""
        from sites_battle import run_battle

        from models import AlgorithmParams

        rare_cards = _make_rare_cards()
        pool = _make_pool(rare_cards)
        cards = _make_mixed_cards()
        state = _make_quest_state(cards, pool=pool, essence=250, completion_level=3)
        battle_cfg = _battle_config()
        quest_cfg = _quest_config()
        bosses = _make_bosses()
        algo_params = AlgorithmParams(
            exponent=1.4,
            floor_weight=0.5,
            neutral_base=3.0,
            staleness_factor=0.3,
        )

        log_calls: list[dict[str, object]] = []

        class FakeLogger:
            def log_battle_complete(
                self,
                opponent_name: str,
                essence_reward: int,
                rare_pick: Optional[Card],
            ) -> None:
                log_calls.append({
                    "opponent_name": opponent_name,
                    "essence_reward": essence_reward,
                    "rare_pick": rare_pick,
                })

        with patch("sites_battle.input_handler.wait_for_continue"), \
             patch("sites_battle.input_handler.single_select", return_value=0):
            run_battle(
                state=state,
                battle_config=battle_cfg,
                quest_config=quest_cfg,
                algorithm_params=algo_params,
                bosses=bosses,
                dreamscape_name="Test Dreamscape",
                dreamscape_number=1,
                logger=FakeLogger(),  # type: ignore[arg-type]
            )

        assert len(log_calls) == 1
        non_final_names = [b.name for b in bosses if not b.is_final]
        assert log_calls[0]["opponent_name"] in non_final_names
        # Essence at level 3: 100 + 25 * 3 = 175
        assert log_calls[0]["essence_reward"] == 175

    def test_battle_essence_scales_with_level(self) -> None:
        """Essence reward should scale: base + per_level * completion_level."""
        from sites_battle import run_battle

        from models import AlgorithmParams

        rare_cards = _make_rare_cards()
        pool = _make_pool(rare_cards)
        cards = _make_mixed_cards()
        state = _make_quest_state(cards, pool=pool, essence=0, completion_level=5)
        battle_cfg = _battle_config()
        quest_cfg = _quest_config()
        algo_params = AlgorithmParams(
            exponent=1.4,
            floor_weight=0.5,
            neutral_base=3.0,
            staleness_factor=0.3,
        )

        with patch("sites_battle.input_handler.wait_for_continue"), \
             patch("sites_battle.input_handler.single_select", return_value=0):
            run_battle(
                state=state,
                battle_config=battle_cfg,
                quest_config=quest_cfg,
                algorithm_params=algo_params,
                bosses=[],
                dreamscape_name="Test Dreamscape",
                dreamscape_number=1,
                logger=None,
            )

        # 0 + 100 + 25 * 5 = 225
        assert state.essence == 225
