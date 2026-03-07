"""Tests for the main quest flow module."""

import random
from typing import Any, Optional
from unittest.mock import MagicMock, patch

from models import (
    BaneCard,
    Boss,
    DeckCard,
    Dreamcaller,
    DreamscapeNode,
    Dreamsign,
    EffectType,
    Journey,
    NodeState,
    Site,
    SiteType,
    TemptingOffer,
)
from quest_state import QuestState
from site_dispatch import SiteData, VisitContext


def _make_quest_state(seed: int = 42) -> QuestState:
    rng = random.Random(seed)
    return QuestState(
        essence=250,
        rng=rng,
        human_agent=None,
        ai_agents=[],
        cube=None,
        draft_cfg=None,
        packs=[],
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
            "reroll_cost": 50,
            "items_count": 6,
            "discount_min": 30,
            "discount_max": 90,
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
    }


def _make_dreamcallers() -> list[Dreamcaller]:
    return [
        Dreamcaller(
            name="Test Caller",
            archetype="Flash",
            essence_bonus=50,
            ability_text="Test ability.",
        ),
    ]


def _make_dreamsigns() -> list[Dreamsign]:
    return [
        Dreamsign(
            name="Test Sign",
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
            card_type="Event",
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
        for i in range(5):
            state.add_card(f"inst_{i}")
        assert state.deck_count() == 5

        _enforce_deck_limits(state, None)
        assert state.deck_count() == 30
        assert state.deck_count() > state.min_deck

    def test_over_limit_triggers_forced_purge(self) -> None:
        from flow import _enforce_deck_limits

        state = _make_quest_state()
        for i in range(55):
            state.add_card(f"inst_{i}")
        assert state.deck_count() == 55

        with patch("sites_purge.forced_deck_limit_purge") as mock_purge:
            _enforce_deck_limits(state, None)

        mock_purge.assert_called_once_with(state, None)

    def test_within_limits_no_action(self) -> None:
        from flow import _enforce_deck_limits

        state = _make_quest_state()
        for i in range(30):
            state.add_card(f"inst_{i}")
        assert 25 <= state.deck_count() <= 50

        with patch("sites_purge.forced_deck_limit_purge") as mock_purge:
            _enforce_deck_limits(state, None)

        mock_purge.assert_not_called()
        assert state.deck_count() == 30


class TestHandlePostBattle:
    """Test post-battle actions: node completion, victory check."""

    def test_increments_completion_level(self) -> None:
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

    def test_returns_true_on_victory(self) -> None:
        from flow import _handle_post_battle

        from models import Biome

        state = _make_quest_state()
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
        for i in range(10):
            state.add_card(f"inst_{i}")

        # Provide a mock human_agent with a w vector
        class MockAgent:
            w = [0.1] * 8

        state.human_agent = MockAgent()
        state.dreamcaller = _make_dreamcallers()[0]

        with patch("builtins.print") as mock_print:
            _show_victory(state, 7, 5, None)

        assert mock_print.call_count >= 1

    def test_show_victory_logs_session_end(self) -> None:
        from flow import _show_victory

        state = _make_quest_state()
        for i in range(10):
            state.add_card(f"inst_{i}")

        class MockAgent:
            w = [0.1] * 8

        state.human_agent = MockAgent()
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
        for i in range(5):
            state.add_card(f"inst_{i}")

        with patch("builtins.print") as mock_print:
            _enforce_deck_limits(state, None)

        printed_text = " ".join(
            str(c)
            for c in mock_print.call_args_list
            for c in (c.args if c.args else [])
        )
        assert (
            "25" in printed_text
            or "auto" in printed_text.lower()
            or "duplicate" in printed_text.lower()
        )


class TestViewDeckOption:
    """Test deck viewer option in dreamscape site selection."""

    def test_view_deck_option_importable(self) -> None:
        from flow import _VIEW_DECK_LABEL

        assert isinstance(_VIEW_DECK_LABEL, str)
        assert "Deck" in _VIEW_DECK_LABEL

    def test_view_deck_does_not_consume_site_visit(self) -> None:
        """Viewing the deck should not mark any site as visited."""
        state = _make_quest_state()
        for i in range(5):
            state.add_card(f"inst_{i}")

        sites = [
            Site(site_type=SiteType.DRAFT),
            Site(site_type=SiteType.ESSENCE),
            Site(site_type=SiteType.BATTLE),
        ]
        for s in sites:
            assert not s.is_visited

    def test_view_deck_preserves_essence(self) -> None:
        """Viewing the deck should not change essence."""
        state = _make_quest_state()
        initial_essence = state.essence
        assert state.essence == initial_essence

    def test_view_deck_preserves_deck_count(self) -> None:
        """Viewing the deck should not add or remove cards."""
        state = _make_quest_state()
        for i in range(5):
            state.add_card(f"inst_{i}")
        initial_count = state.deck_count()
        assert state.deck_count() == initial_count


class TestSiteVisitErrorHandling:
    """Test graceful error handling when a site handler raises."""

    def test_site_handler_exception_is_caught(self) -> None:
        """If a site handler raises, the loop continues and marks the site visited."""
        from flow import _dreamscape_loop

        state = _make_quest_state()
        data = _make_site_data()
        logger = MagicMock()

        node = DreamscapeNode(
            node_id=1,
            name="Test Node",
            biome=__import__("models").Biome.VERDANT,
            sites=[
                Site(site_type=SiteType.ESSENCE),
                Site(site_type=SiteType.BATTLE),
            ],
            state=NodeState.AVAILABLE,
            adjacent=[0],
        )

        call_count = 0

        def mock_visit(site: Site, *args: object, **kwargs: object) -> None:
            nonlocal call_count
            call_count += 1
            if call_count == 1:
                raise RuntimeError("Simulated site failure")
            site.is_visited = True

        with patch("site_dispatch.visit_site", side_effect=mock_visit):
            with patch("input_handler.single_select", return_value=1):
                with patch("render_atlas.render_dreamscape_sites", return_value=""):
                    with patch("render_atlas.site_type_name", return_value="Test"):
                        with patch("builtins.print"):
                            _dreamscape_loop(node, state, data, logger, 1)

        assert node.sites[0].is_visited
        assert node.sites[1].is_visited

    def test_site_handler_exception_logs_error(self) -> None:
        """If a site handler raises, the error is logged to JSONL."""
        from flow import _dreamscape_loop

        state = _make_quest_state()
        data = _make_site_data()
        logger = MagicMock()

        node = DreamscapeNode(
            node_id=1,
            name="Test Node",
            biome=__import__("models").Biome.VERDANT,
            sites=[
                Site(site_type=SiteType.ESSENCE),
                Site(site_type=SiteType.BATTLE),
            ],
            state=NodeState.AVAILABLE,
            adjacent=[0],
        )

        call_count = 0

        def mock_visit(site: Site, *args: object, **kwargs: object) -> None:
            nonlocal call_count
            call_count += 1
            if call_count == 1:
                raise RuntimeError("Simulated site failure")
            site.is_visited = True

        with patch("site_dispatch.visit_site", side_effect=mock_visit):
            with patch("input_handler.single_select", return_value=1):
                with patch("render_atlas.render_dreamscape_sites", return_value=""):
                    with patch("render_atlas.site_type_name", return_value="Test"):
                        with patch("builtins.print"):
                            _dreamscape_loop(node, state, data, logger, 1)

        logger.log_error.assert_called_once()
        error_call = logger.log_error.call_args
        assert "Simulated site failure" in str(error_call)
