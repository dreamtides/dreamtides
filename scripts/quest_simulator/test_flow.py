"""Tests for the main quest flow module."""

import random
from types import MappingProxyType
from typing import Any, Optional
from unittest.mock import MagicMock, call, patch

from models import (
    AlgorithmParams,
    BaneCard,
    Boss,
    Card,
    CardType,
    DeckCard,
    DraftParams,
    Dreamcaller,
    DreamscapeNode,
    Dreamsign,
    EffectType,
    Journey,
    NodeState,
    PoolEntry,
    PoolParams,
    Rarity,
    Resonance,
    Site,
    SiteType,
    TemptingOffer,
)
from quest_state import QuestState
from site_dispatch import SiteData, VisitContext


def _make_card(
    name: str = "Test Card",
    card_number: int = 1,
    rarity: Rarity = Rarity.COMMON,
    resonances: Optional[frozenset[Resonance]] = None,
) -> Card:
    return Card(
        name=name,
        card_number=card_number,
        energy_cost=2,
        card_type=CardType.CHARACTER,
        subtype=None,
        is_fast=False,
        spark=1,
        rarity=rarity,
        rules_text=f"Rules for {name}.",
        resonances=resonances or frozenset(),
        tags=frozenset(),
    )


def _make_test_cards() -> list[Card]:
    return [
        _make_card("Card A", 1, Rarity.COMMON, frozenset({Resonance.TIDE})),
        _make_card("Card B", 2, Rarity.COMMON, frozenset({Resonance.TIDE})),
        _make_card("Card C", 3, Rarity.UNCOMMON, frozenset({Resonance.EMBER})),
        _make_card("Card D", 4, Rarity.UNCOMMON, frozenset({Resonance.EMBER})),
        _make_card("Card E", 5, Rarity.RARE, frozenset({Resonance.STONE})),
        _make_card("Card F", 6, Rarity.COMMON, frozenset({Resonance.ZEPHYR})),
        _make_card("Card G", 7, Rarity.COMMON, frozenset({Resonance.RUIN})),
        _make_card("Card H", 8, Rarity.COMMON, frozenset()),
        _make_card(
            "Card I",
            9,
            Rarity.LEGENDARY,
            frozenset({Resonance.TIDE, Resonance.RUIN}),
        ),
        _make_card("Card J", 10, Rarity.UNCOMMON, frozenset({Resonance.STONE})),
    ]


def _make_pool(cards: list[Card]) -> list[PoolEntry]:
    return [PoolEntry(card) for card in cards]


def _make_algorithm_params() -> AlgorithmParams:
    return AlgorithmParams(
        exponent=1.4,
        floor_weight=0.5,
        neutral_base=3.0,
        staleness_factor=0.3,
    )


def _make_draft_params() -> DraftParams:
    return DraftParams(cards_per_pick=4, picks_per_site=5)


def _make_pool_params() -> PoolParams:
    return PoolParams(
        copies_common=4,
        copies_uncommon=3,
        copies_rare=2,
        copies_legendary=1,
        variance_min=0.75,
        variance_max=1.25,
    )


def _make_quest_state(seed: int = 42) -> QuestState:
    cards = _make_test_cards()
    pool = _make_pool(cards)
    rng = random.Random(seed)
    variance = {r: 1.0 for r in Resonance}
    return QuestState(
        essence=250,
        pool=pool,
        rng=rng,
        all_cards=cards,
        pool_variance=variance,
    )


def _make_config() -> dict[str, dict[str, Any]]:
    return {
        "quest": {
            "starting_essence": 250,
            "max_deck": 50,
            "min_deck": 25,
            "max_dreamsigns": 12,
            "total_battles": 7,
            "miniboss_battle": 4,
        },
        "shop": {
            "price_common": 50,
            "price_uncommon": 80,
            "price_rare": 120,
            "price_legendary": 200,
            "reroll_cost": 50,
            "discount_min": 30,
            "discount_max": 90,
            "items_count": 6,
        },
        "battle_rewards": {
            "base_essence": 100,
            "per_level": 25,
            "rare_pick_count": 3,
        },
        "essence_sites": {
            "amount_level_0": 200,
            "amount_level_2": 250,
            "amount_level_4": 300,
        },
        "tags": {
            "scale": 1.5,
            "min_theme_cards": 6,
            "relevance_boost": 2.0,
            "depth_factor": 0.1,
        },
    }


def _make_dreamcallers() -> list[Dreamcaller]:
    return [
        Dreamcaller(
            name="Test Caller",
            resonances=frozenset({Resonance.TIDE}),
            resonance_bonus=MappingProxyType({"Tide": 3}),
            tags=frozenset({"mechanic:draw"}),
            tag_bonus=MappingProxyType({"mechanic:draw": 1}),
            essence_bonus=50,
            ability_text="Test ability.",
        ),
    ]


def _make_dreamsigns() -> list[Dreamsign]:
    return [
        Dreamsign(
            name="Test Sign",
            resonance=Resonance.TIDE,
            tags=frozenset(),
            effect_text="Test effect.",
            is_bane=False,
        ),
    ]


def _make_journeys() -> list[Journey]:
    return [
        Journey(
            name="Test Journey",
            description="A test journey.",
            effect_type=EffectType.ADD_ESSENCE,
            effect_value=100,
        ),
    ]


def _make_offers() -> list[TemptingOffer]:
    return [
        TemptingOffer(
            reward_name="Test Reward",
            reward_description="A reward.",
            reward_effect_type=EffectType.ADD_ESSENCE,
            reward_value=50,
            cost_name="Test Cost",
            cost_description="A cost.",
            cost_effect_type=EffectType.LOSE_ESSENCE,
            cost_value=25,
        ),
    ]


def _make_banes() -> list[BaneCard]:
    return [
        BaneCard(
            name="Test Bane",
            rules_text="Bad things happen.",
            card_type=CardType.EVENT,
            energy_cost=0,
        ),
    ]


def _make_bosses() -> list[Boss]:
    return [
        Boss(
            name="Test Boss",
            archetype="Aggro",
            ability_text="Test boss ability.",
            deck_description="A test deck.",
            is_final=False,
            resonances=frozenset({Resonance.EMBER}),
        ),
    ]


def _make_site_data() -> SiteData:
    return SiteData(
        dreamcallers=_make_dreamcallers(),
        dreamsigns=_make_dreamsigns(),
        journeys=_make_journeys(),
        offers=_make_offers(),
        banes=_make_banes(),
        bosses=_make_bosses(),
        algorithm_params=_make_algorithm_params(),
        draft_params=_make_draft_params(),
        pool_params=_make_pool_params(),
        config=_make_config(),
    )


class TestFlowImport:
    """Test that flow module imports correctly."""

    def test_run_quest_importable(self) -> None:
        from flow import run_quest

        assert callable(run_quest)

    def test_atlas_loop_importable(self) -> None:
        from flow import _atlas_loop

        assert callable(_atlas_loop)

    def test_dreamscape_loop_importable(self) -> None:
        from flow import _dreamscape_loop

        assert callable(_dreamscape_loop)

    def test_enforce_deck_limits_importable(self) -> None:
        from flow import _enforce_deck_limits

        assert callable(_enforce_deck_limits)

    def test_handle_post_battle_importable(self) -> None:
        from flow import _handle_post_battle

        assert callable(_handle_post_battle)

    def test_show_victory_importable(self) -> None:
        from flow import _show_victory

        assert callable(_show_victory)


class TestGetSelectableSites:
    """Test that _get_selectable_sites filters correctly."""

    def test_all_non_battle_unvisited_are_selectable(self) -> None:
        from flow import _get_selectable_sites

        sites = [
            Site(site_type=SiteType.DRAFT),
            Site(site_type=SiteType.ESSENCE),
            Site(site_type=SiteType.BATTLE),
        ]
        selectable = _get_selectable_sites(sites)
        assert len(selectable) == 2
        assert all(s.site_type != SiteType.BATTLE for s in selectable)

    def test_battle_unlocked_when_all_others_visited(self) -> None:
        from flow import _get_selectable_sites

        sites = [
            Site(site_type=SiteType.DRAFT, is_visited=True),
            Site(site_type=SiteType.ESSENCE, is_visited=True),
            Site(site_type=SiteType.BATTLE),
        ]
        selectable = _get_selectable_sites(sites)
        assert len(selectable) == 1
        assert selectable[0].site_type == SiteType.BATTLE

    def test_visited_sites_excluded(self) -> None:
        from flow import _get_selectable_sites

        sites = [
            Site(site_type=SiteType.DRAFT, is_visited=True),
            Site(site_type=SiteType.ESSENCE),
            Site(site_type=SiteType.BATTLE),
        ]
        selectable = _get_selectable_sites(sites)
        assert len(selectable) == 1
        assert selectable[0].site_type == SiteType.ESSENCE

    def test_empty_sites_returns_empty(self) -> None:
        from flow import _get_selectable_sites

        selectable = _get_selectable_sites([])
        assert selectable == []

    def test_only_battle_and_all_others_visited(self) -> None:
        from flow import _get_selectable_sites

        sites = [
            Site(site_type=SiteType.BATTLE),
        ]
        selectable = _get_selectable_sites(sites)
        assert len(selectable) == 1
        assert selectable[0].site_type == SiteType.BATTLE


class TestEnforceDeckLimits:
    """Test deck limit enforcement before battle."""

    def test_under_limit_triggers_auto_fill(self) -> None:
        from flow import _enforce_deck_limits

        state = _make_quest_state()
        cards = _make_test_cards()
        # Add only 5 cards, well under min_deck of 25
        for card in cards[:5]:
            state.add_card(card)
        assert state.deck_count() == 5

        _enforce_deck_limits(state, None)
        assert state.deck_count() == state.min_deck

    def test_over_limit_triggers_forced_purge(self) -> None:
        from flow import _enforce_deck_limits

        state = _make_quest_state()
        cards = _make_test_cards()
        # Add 55 cards, over max_deck of 50
        for i in range(55):
            state.add_card(cards[i % len(cards)])
        assert state.deck_count() == 55

        with patch("flow.sites_purge.forced_deck_limit_purge") as mock_purge:
            _enforce_deck_limits(state, None)

        mock_purge.assert_called_once_with(state, None)

    def test_within_limits_no_action(self) -> None:
        from flow import _enforce_deck_limits

        state = _make_quest_state()
        cards = _make_test_cards()
        for i in range(30):
            state.add_card(cards[i % len(cards)])
        assert 25 <= state.deck_count() <= 50

        with patch("flow.sites_purge.forced_deck_limit_purge") as mock_purge:
            _enforce_deck_limits(state, None)

        mock_purge.assert_not_called()
        assert state.deck_count() == 30


class TestHandlePostBattle:
    """Test post-battle actions: node completion, staleness decay, victory check."""

    def test_increments_completion_level(self) -> None:
        from flow import _handle_post_battle

        state = _make_quest_state()
        nodes = [
            DreamscapeNode(
                node_id=0,
                name="Nexus",
                biome=random.choice(list(__import__("models").Biome)),
                sites=[],
                state=NodeState.COMPLETED,
                adjacent=[1],
            ),
            DreamscapeNode(
                node_id=1,
                name="Test Node",
                biome=random.choice(list(__import__("models").Biome)),
                sites=[],
                state=NodeState.AVAILABLE,
                adjacent=[0],
            ),
        ]
        assert state.completion_level == 0

        victory = _handle_post_battle(state, nodes, 1, 7, None)
        assert state.completion_level == 1
        assert not victory

    def test_marks_node_completed(self) -> None:
        from flow import _handle_post_battle

        from models import Biome

        state = _make_quest_state()
        nodes = [
            DreamscapeNode(
                node_id=0,
                name="Nexus",
                biome=Biome.VERDANT,
                sites=[],
                state=NodeState.COMPLETED,
                adjacent=[1],
            ),
            DreamscapeNode(
                node_id=1,
                name="Test Node",
                biome=Biome.VERDANT,
                sites=[],
                state=NodeState.AVAILABLE,
                adjacent=[0],
            ),
        ]

        _handle_post_battle(state, nodes, 1, 7, None)
        assert nodes[1].state == NodeState.COMPLETED

    def test_generates_new_nodes(self) -> None:
        from flow import _handle_post_battle

        from models import Biome

        state = _make_quest_state()
        nodes = [
            DreamscapeNode(
                node_id=0,
                name="Nexus",
                biome=Biome.VERDANT,
                sites=[],
                state=NodeState.COMPLETED,
                adjacent=[1],
            ),
            DreamscapeNode(
                node_id=1,
                name="Test Node",
                biome=Biome.VERDANT,
                sites=[],
                state=NodeState.AVAILABLE,
                adjacent=[0],
            ),
        ]
        initial_count = len(nodes)

        _handle_post_battle(state, nodes, 1, 7, None)
        assert len(nodes) > initial_count

    def test_decays_staleness(self) -> None:
        from flow import _handle_post_battle

        from models import Biome

        state = _make_quest_state()
        # Set staleness on some pool entries
        for entry in state.pool[:3]:
            entry.staleness = 2

        nodes = [
            DreamscapeNode(
                node_id=0,
                name="Nexus",
                biome=Biome.VERDANT,
                sites=[],
                state=NodeState.COMPLETED,
                adjacent=[1],
            ),
            DreamscapeNode(
                node_id=1,
                name="Test Node",
                biome=Biome.VERDANT,
                sites=[],
                state=NodeState.AVAILABLE,
                adjacent=[0],
            ),
        ]

        _handle_post_battle(state, nodes, 1, 7, None)
        for entry in state.pool[:3]:
            assert entry.staleness == 1

    def test_returns_true_on_victory(self) -> None:
        from flow import _handle_post_battle

        from models import Biome

        state = _make_quest_state()
        # Set completion to 6, so after increment it will be 7 (== total_battles)
        state.completion_level = 6

        nodes = [
            DreamscapeNode(
                node_id=0,
                name="Nexus",
                biome=Biome.VERDANT,
                sites=[],
                state=NodeState.COMPLETED,
                adjacent=[1],
            ),
            DreamscapeNode(
                node_id=1,
                name="Test Node",
                biome=Biome.VERDANT,
                sites=[],
                state=NodeState.AVAILABLE,
                adjacent=[0],
            ),
        ]

        victory = _handle_post_battle(state, nodes, 1, 7, None)
        assert victory is True

    def test_returns_false_when_not_victory(self) -> None:
        from flow import _handle_post_battle

        from models import Biome

        state = _make_quest_state()
        state.completion_level = 3

        nodes = [
            DreamscapeNode(
                node_id=0,
                name="Nexus",
                biome=Biome.VERDANT,
                sites=[],
                state=NodeState.COMPLETED,
                adjacent=[1],
            ),
            DreamscapeNode(
                node_id=1,
                name="Test Node",
                biome=Biome.VERDANT,
                sites=[],
                state=NodeState.AVAILABLE,
                adjacent=[0],
            ),
        ]

        victory = _handle_post_battle(state, nodes, 1, 7, None)
        assert victory is False


class TestShowVictory:
    """Test victory screen display."""

    def test_show_victory_prints_output(self) -> None:
        from flow import _show_victory

        state = _make_quest_state()
        # Add some cards for a realistic state
        cards = _make_test_cards()
        for card in cards:
            state.add_card(card)
        state.dreamcaller = _make_dreamcallers()[0]

        with patch("builtins.print") as mock_print:
            _show_victory(state, 7, 5, None)

        # Victory screen should have been printed
        assert mock_print.call_count >= 1

    def test_show_victory_logs_session_end(self) -> None:
        from flow import _show_victory

        state = _make_quest_state()
        cards = _make_test_cards()
        for card in cards:
            state.add_card(card)
        state.dreamcaller = _make_dreamcallers()[0]

        logger = MagicMock()

        with patch("builtins.print"):
            _show_victory(state, 7, 5, logger)

        logger.log_session_end.assert_called_once()


class TestAutoFillMessage:
    """Test that auto-fill prints an informative message."""

    def test_auto_fill_prints_message(self) -> None:
        from flow import _enforce_deck_limits

        state = _make_quest_state()
        cards = _make_test_cards()
        for card in cards[:5]:
            state.add_card(card)

        with patch("builtins.print") as mock_print:
            _enforce_deck_limits(state, None)

        # Should have printed something about auto-filling
        printed_text = " ".join(
            str(c) for c in mock_print.call_args_list
            for c in (c.args if c.args else [])
        )
        assert "25" in printed_text or "auto" in printed_text.lower() or "duplicate" in printed_text.lower()
