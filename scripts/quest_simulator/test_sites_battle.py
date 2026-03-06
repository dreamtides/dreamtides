"""Tests for sites_battle module.

Tests the Battle site interaction using the round manager and draft
engine types (CardInstance, CardDesign, AgentState).
"""

import io
import random
from typing import Optional
from unittest.mock import patch

import agents
import card_generator
import cube_manager
import show_n
from config import (
    AgentsConfig,
    CardsConfig,
    CubeConfig,
    DraftConfig,
    PackGenerationConfig,
    RefillConfig,
    ScoringConfig,
    SimulatorConfig,
)
from draft_models import CubeConsumptionMode
from models import Boss
from quest_state import QuestState


def _make_draft_cfg() -> SimulatorConfig:
    """Create a minimal SimulatorConfig for testing."""
    return SimulatorConfig(
        draft=DraftConfig(
            seat_count=6,
            pack_size=20,
            human_seats=1,
            picks_per_round=[10],
            round_count=1,
            alternate_direction=False,
        ),
        agents=AgentsConfig(
            show_n=4,
            show_n_strategy="sharpened_preference",
            policy="adaptive",
            ai_optimality=0.80,
            learning_rate=3.0,
            openness_window=3,
        ),
        cards=CardsConfig(
            archetype_count=8,
            source="synthetic",
        ),
        cube=CubeConfig(
            distinct_cards=540,
            copies_per_card=1,
            consumption_mode=CubeConsumptionMode.WITH_REPLACEMENT,
        ),
        refill=RefillConfig(strategy="no_refill"),
        pack_generation=PackGenerationConfig(strategy="seeded_themed"),
        scoring=ScoringConfig(),
    )


def _make_quest_state(
    seed: int = 42,
    essence: int = 250,
    completion_level: int = 0,
    cfg: SimulatorConfig | None = None,
) -> QuestState:
    """Create a QuestState with a full draft engine for testing."""
    rng = random.Random(seed)
    if cfg is None:
        cfg = _make_draft_cfg()
    designs = card_generator.generate_cards(cfg, rng)
    cube = cube_manager.CubeManager(
        designs,
        copies_per_card=cfg.cube.copies_per_card,
        consumption_mode=cfg.cube.consumption_mode,
    )
    human_agent = agents.create_agent(cfg.cards.archetype_count)
    ai_agents_list = [agents.create_agent(cfg.cards.archetype_count) for _ in range(5)]

    state = QuestState(
        essence=essence,
        rng=rng,
        human_agent=human_agent,
        ai_agents=ai_agents_list,
        cube=cube,
        draft_cfg=cfg,
    )
    state.completion_level = completion_level
    return state


def _make_bosses() -> list[Boss]:
    """Create a set of test bosses."""
    return [
        Boss(
            name="Pyrra, Ember Dancer",
            archetype="Aggro/Burn",
            ability_text="Whenever you play a character that costs 2 or less, kindle 1.",
            deck_description="Cheap aggro deck.",
            is_final=False,
        ),
        Boss(
            name="Thornroot, Grove-Warden",
            archetype="Spirit Animal Tribal",
            ability_text="Spirit Animals increase energy by 1.",
            deck_description="Ramp deck.",
            is_final=False,
        ),
        Boss(
            name="Nihil, the Silence Between",
            archetype="Draw-Go Control",
            ability_text="Whenever your opponent plays a card, draw a card.",
            deck_description="Control deck.",
            is_final=True,
        ),
        Boss(
            name="Keth, the Binding Dark",
            archetype="Prison",
            ability_text="Cards cost 1 more.",
            deck_description="Prison deck.",
            is_final=True,
        ),
    ]


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
    """Tests for the full run_battle interaction using the round manager."""

    def test_battle_grants_essence(self) -> None:
        """After battle, essence should increase by the reward amount."""
        from sites_battle import run_battle

        state = _make_quest_state(essence=250, completion_level=0)

        with patch("sites_battle.input_handler.wait_for_continue"), patch(
            "sites_battle.input_handler.single_select", return_value=0
        ):
            run_battle(
                state=state,
                battle_config=_battle_config(),
                quest_config=_quest_config(),
                bosses=[],
                dreamscape_name="Test Dreamscape",
                dreamscape_number=1,
                logger=None,
            )

        # 250 + 100 (base) + 25 * 0 (level) = 350
        assert state.essence == 350

    def test_battle_does_not_increment_completion(self) -> None:
        """run_battle does not increment completion (flow handles that)."""
        from sites_battle import run_battle

        state = _make_quest_state(essence=250, completion_level=2)

        with patch("sites_battle.input_handler.wait_for_continue"), patch(
            "sites_battle.input_handler.single_select", return_value=0
        ):
            run_battle(
                state=state,
                battle_config=_battle_config(),
                quest_config=_quest_config(),
                bosses=[],
                dreamscape_name="Test Dreamscape",
                dreamscape_number=1,
                logger=None,
            )

        assert state.completion_level == 2

    def test_battle_adds_card_to_deck(self) -> None:
        """After battle, a card should be added to the deck from the pack."""
        from sites_battle import run_battle

        state = _make_quest_state(essence=250, completion_level=0)

        with patch("sites_battle.input_handler.wait_for_continue"), patch(
            "sites_battle.input_handler.single_select", return_value=0
        ):
            run_battle(
                state=state,
                battle_config=_battle_config(),
                quest_config=_quest_config(),
                bosses=[],
                dreamscape_name="Test Dreamscape",
                dreamscape_number=1,
                logger=None,
            )

        assert state.deck_count() == 1
        # The card should be a valid CardInstance
        dc = state.deck[0]
        assert hasattr(dc.instance, "design")
        assert hasattr(dc.instance.design, "name")

    def test_battle_advances_global_pick_index(self) -> None:
        """After battle, the global pick index should advance by 1."""
        from sites_battle import run_battle

        state = _make_quest_state(essence=250, completion_level=0)
        initial_pick = state.global_pick_index

        with patch("sites_battle.input_handler.wait_for_continue"), patch(
            "sites_battle.input_handler.single_select", return_value=0
        ):
            run_battle(
                state=state,
                battle_config=_battle_config(),
                quest_config=_quest_config(),
                bosses=[],
                dreamscape_name="Test Dreamscape",
                dreamscape_number=1,
                logger=None,
            )

        assert state.global_pick_index == initial_pick + 1

    def test_battle_updates_human_agent(self) -> None:
        """After battle pick, human agent's drafted list should grow by 1."""
        from sites_battle import run_battle

        state = _make_quest_state(essence=250, completion_level=0)
        assert len(state.human_agent.drafted) == 0

        with patch("sites_battle.input_handler.wait_for_continue"), patch(
            "sites_battle.input_handler.single_select", return_value=0
        ):
            run_battle(
                state=state,
                battle_config=_battle_config(),
                quest_config=_quest_config(),
                bosses=[],
                dreamscape_name="Test Dreamscape",
                dreamscape_number=1,
                logger=None,
            )

        assert len(state.human_agent.drafted) == 1

    def test_battle_with_logger(self) -> None:
        """Battle should call log_battle_complete when logger provided."""
        from sites_battle import run_battle

        log_calls: list[dict[str, object]] = []

        class FakeLogger:
            def log_battle_complete(self, opponent_name, essence_reward, rare_pick):
                log_calls.append(
                    {
                        "opponent_name": opponent_name,
                        "essence_reward": essence_reward,
                        "rare_pick": rare_pick,
                    }
                )

            def log_site_visit(self, **kwargs):
                pass

        state = _make_quest_state(essence=250, completion_level=0)

        with patch("sites_battle.input_handler.wait_for_continue"), patch(
            "sites_battle.input_handler.single_select", return_value=0
        ):
            run_battle(
                state=state,
                battle_config=_battle_config(),
                quest_config=_quest_config(),
                bosses=[],
                dreamscape_name="Test Dreamscape",
                dreamscape_number=1,
                logger=FakeLogger(),  # type: ignore[arg-type]
            )

        assert len(log_calls) == 1
        assert log_calls[0]["opponent_name"] == "Dream Guardian"
        assert log_calls[0]["essence_reward"] == 100
        assert log_calls[0]["rare_pick"] is not None

    def test_battle_essence_scales_with_level(self) -> None:
        """Essence reward should scale: base + per_level * completion_level."""
        from sites_battle import run_battle

        state = _make_quest_state(essence=0, completion_level=5)

        with patch("sites_battle.input_handler.wait_for_continue"), patch(
            "sites_battle.input_handler.single_select", return_value=0
        ):
            run_battle(
                state=state,
                battle_config=_battle_config(),
                quest_config=_quest_config(),
                bosses=[],
                dreamscape_name="Test Dreamscape",
                dreamscape_number=1,
                logger=None,
            )

        # 0 + 100 + 25 * 5 = 225
        assert state.essence == 225


class TestBattleDisplayOutput:
    """Tests for the visual display output of run_battle."""

    def _run_and_capture(
        self,
        completion_level: int = 0,
        bosses: Optional[list[Boss]] = None,
    ) -> str:
        """Run a battle and return captured stdout."""
        from sites_battle import run_battle

        state = _make_quest_state(
            essence=250,
            completion_level=completion_level,
        )

        buf = io.StringIO()
        with patch("sites_battle.input_handler.wait_for_continue"), patch(
            "sites_battle.input_handler.single_select", return_value=0
        ), patch("sys.stdout", buf):
            run_battle(
                state=state,
                battle_config=_battle_config(),
                quest_config=_quest_config(),
                bosses=bosses or [],
                dreamscape_name="Test Dreamscape",
                dreamscape_number=1,
                logger=None,
            )
        return buf.getvalue()

    def test_boss_battle_shows_dramatic_header(self) -> None:
        """Boss battles should show dramatic intro with name and archetype."""
        bosses = _make_bosses()
        output = self._run_and_capture(completion_level=3, bosses=bosses)
        assert "BATTLE 4" in output
        assert "MINIBOSS" in output

    def test_boss_battle_shows_archetype(self) -> None:
        """Boss battles should show the archetype."""
        bosses = _make_bosses()
        output = self._run_and_capture(completion_level=3, bosses=bosses)
        assert "Archetype:" in output

    def test_guardian_battle_shows_guardian_name(self) -> None:
        """Dream Guardian battles should show the guardian name."""
        output = self._run_and_capture(completion_level=0)
        assert "Dream Guardian" in output

    def test_victory_message_visible(self) -> None:
        """Victory message should be visually distinct with VICTORY text."""
        output = self._run_and_capture(completion_level=0)
        assert "VICTORY" in output

    def test_essence_reward_shown(self) -> None:
        """Essence reward amount should appear in the output."""
        output = self._run_and_capture(completion_level=0)
        assert "Essence reward" in output
        assert "+100" in output

    def test_completion_progress_shown(self) -> None:
        """After battle, completion progress should be displayed."""
        output = self._run_and_capture(completion_level=2)
        assert "Completion:" in output
        assert "3/7" in output

    def test_double_line_separators_present(self) -> None:
        """Output should contain double-line separators for dramatic framing."""
        output = self._run_and_capture(completion_level=0)
        assert "\u2550" in output

    def test_archetype_preference_footer(self) -> None:
        """Output should contain the archetype preference footer."""
        output = self._run_and_capture(completion_level=0)
        assert "Archetype Preferences" in output

    def test_no_resonance_references(self) -> None:
        """Output should not reference the old resonance system."""
        output = self._run_and_capture(completion_level=0)
        # Check that no resonance types appear in the output
        for term in ["Tide", "Ember", "Zephyr", "Stone", "Ruin"]:
            assert term not in output, f"Found old resonance term '{term}' in output"


class TestBattleConfigDriven:
    """Tests that battle card pick honors configuration values."""

    def test_battle_respects_rare_pick_count(self) -> None:
        """Battle should offer rare_pick_count cards from config, not a hardcoded count."""
        from sites_battle import run_battle

        state = _make_quest_state(essence=250, completion_level=0)

        select_calls: list[int] = []
        original_select = show_n.select_cards

        def _tracking_select(pack_cards, n, strategy, rng, **kwargs):
            select_calls.append(n)
            return original_select(pack_cards, n, strategy, rng, **kwargs)

        battle_cfg = _battle_config()
        battle_cfg["rare_pick_count"] = 1

        with patch("sites_battle.input_handler.wait_for_continue"), patch(
            "sites_battle.input_handler.single_select", return_value=0
        ), patch("sites_battle.show_n.select_cards", side_effect=_tracking_select):
            run_battle(
                state=state,
                battle_config=battle_cfg,
                quest_config=_quest_config(),
                bosses=[],
                dreamscape_name="Test",
                dreamscape_number=1,
                logger=None,
            )

        assert len(select_calls) == 1
        assert select_calls[0] == 1

    def test_battle_empty_offer_advances_pick(self) -> None:
        """When show_n returns empty, the pick should still advance."""
        from sites_battle import run_battle

        state = _make_quest_state(essence=250, completion_level=0)
        initial_pick = state.global_pick_index

        with patch("sites_battle.input_handler.wait_for_continue"), patch(
            "sites_battle.show_n.select_cards", return_value=[]
        ):
            run_battle(
                state=state,
                battle_config=_battle_config(),
                quest_config=_quest_config(),
                bosses=[],
                dreamscape_name="Test",
                dreamscape_number=1,
                logger=None,
            )

        # Pick should still advance even with empty offer
        assert state.global_pick_index == initial_pick + 1
        # No card should be added
        assert state.deck_count() == 0
