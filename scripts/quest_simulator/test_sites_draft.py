"""Tests for sites_draft module.

Tests the Draft site interaction using the round manager and draft
engine types (CardInstance, CardDesign, AgentState).
"""

import random
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
            consumption_mode="with_replacement",
        ),
        refill=RefillConfig(strategy="no_refill"),
        pack_generation=PackGenerationConfig(strategy="seeded_themed"),
        scoring=ScoringConfig(),
    )


def _make_quest_state(seed: int = 42, cfg: SimulatorConfig | None = None) -> QuestState:
    """Create a QuestState with a full draft engine for testing."""
    rng = random.Random(seed)
    if cfg is None:
        cfg = _make_draft_cfg()
    designs = card_generator.generate_cards(cfg, rng)
    cube = cube_manager.CubeManager(
        designs,
        copies_per_card=cfg.cube.copies_per_card,
        consumption_mode=CubeConsumptionMode(cfg.cube.consumption_mode),
    )
    human_agent = agents.create_agent(cfg.cards.archetype_count)
    ai_agents_list = [agents.create_agent(cfg.cards.archetype_count) for _ in range(5)]

    return QuestState(
        essence=250,
        rng=rng,
        human_agent=human_agent,
        ai_agents=ai_agents_list,
        cube=cube,
        draft_cfg=cfg,
    )


class TestRunDraft:
    """Tests for run_draft function using the round manager."""

    def test_draft_adds_5_cards_to_deck(self) -> None:
        """A full draft site (5 picks) should add 5 cards to the deck."""
        from sites_draft import run_draft

        state = _make_quest_state()

        with patch("sites_draft.input_handler.single_select", return_value=0):
            run_draft(
                state=state,
                dreamscape_name="Test Dreamscape",
                dreamscape_number=1,
                logger=None,
            )

        assert state.deck_count() == 5

    def test_draft_advances_global_pick_index(self) -> None:
        """After 5 picks, the global pick index should be 5."""
        from sites_draft import run_draft

        state = _make_quest_state()
        initial_pick = state.global_pick_index

        with patch("sites_draft.input_handler.single_select", return_value=0):
            run_draft(
                state=state,
                dreamscape_name="Test Dreamscape",
                dreamscape_number=1,
                logger=None,
            )

        assert state.global_pick_index == initial_pick + 5

    def test_draft_updates_human_agent(self) -> None:
        """After picks, the human agent's drafted list should grow."""
        from sites_draft import run_draft

        state = _make_quest_state()
        assert len(state.human_agent.drafted) == 0

        with patch("sites_draft.input_handler.single_select", return_value=0):
            run_draft(
                state=state,
                dreamscape_name="Test Dreamscape",
                dreamscape_number=1,
                logger=None,
            )

        assert len(state.human_agent.drafted) == 5

    def test_draft_with_logger(self) -> None:
        """Draft should call log_draft_pick on the logger when provided."""
        from sites_draft import run_draft

        log_calls: list[dict[str, object]] = []

        class FakeLogger:
            def log_draft_pick(self, offered_cards, weights, picked_card):
                log_calls.append(
                    {
                        "offered": list(offered_cards),
                        "weights": list(weights),
                        "picked": picked_card,
                    }
                )

            def log_site_visit(self, **kwargs):
                pass

        state = _make_quest_state()

        with patch("sites_draft.input_handler.single_select", return_value=0):
            run_draft(
                state=state,
                dreamscape_name="Test Dreamscape",
                dreamscape_number=1,
                logger=FakeLogger(),  # type: ignore[arg-type]
            )

        assert len(log_calls) == 5
        for call in log_calls:
            assert len(call["offered"]) > 0  # type: ignore[arg-type]
            assert call["picked"] is not None

    def test_draft_different_picks(self) -> None:
        """Picking different indices should yield different cards."""
        from sites_draft import run_draft

        state1 = _make_quest_state(seed=42)
        state2 = _make_quest_state(seed=42)

        with patch("sites_draft.input_handler.single_select", return_value=0):
            run_draft(
                state=state1,
                dreamscape_name="Test",
                dreamscape_number=1,
                logger=None,
            )

        with patch("sites_draft.input_handler.single_select", return_value=1):
            run_draft(
                state=state2,
                dreamscape_name="Test",
                dreamscape_number=1,
                logger=None,
            )

        # Decks should differ when different indices are picked
        names1 = {dc.instance.design.name for dc in state1.deck}
        names2 = {dc.instance.design.name for dc in state2.deck}
        assert names1 != names2

    def test_draft_cards_come_from_pack(self) -> None:
        """Each picked card should be a valid CardInstance with a design."""
        from sites_draft import run_draft

        state = _make_quest_state()

        with patch("sites_draft.input_handler.single_select", return_value=0):
            run_draft(
                state=state,
                dreamscape_name="Test",
                dreamscape_number=1,
                logger=None,
            )

        for dc in state.deck:
            assert hasattr(dc.instance, "design")
            assert hasattr(dc.instance.design, "name")
            assert hasattr(dc.instance.design, "power")
            assert hasattr(dc.instance.design, "commit")
            assert hasattr(dc.instance.design, "flex")

    def test_draft_respects_config_show_n(self) -> None:
        """Draft should offer show_n cards from config, not a hardcoded count."""
        from sites_draft import run_draft

        cfg = _make_draft_cfg()
        cfg.agents.show_n = 2
        cfg.agents.show_n_strategy = "uniform"
        state = _make_quest_state(cfg=cfg)

        select_calls: list[int] = []
        original_select = show_n.select_cards

        def _tracking_select(pack_cards, n, strategy, rng, **kwargs):
            select_calls.append(n)
            return original_select(pack_cards, n, strategy, rng, **kwargs)

        with patch("sites_draft.input_handler.single_select", return_value=0), patch(
            "sites_draft.show_n.select_cards", side_effect=_tracking_select
        ):
            run_draft(
                state=state,
                dreamscape_name="Test",
                dreamscape_number=1,
                logger=None,
            )

        # Every call should have used show_n=2, not the old hardcoded 4
        assert len(select_calls) == 5
        for n in select_calls:
            assert n == 2

    def test_draft_empty_offer_advances_pick(self) -> None:
        """When show_n returns empty, the pick should still advance."""
        from sites_draft import run_draft

        state = _make_quest_state()
        initial_pick = state.global_pick_index

        with patch("sites_draft.input_handler.single_select", return_value=0), patch(
            "sites_draft.show_n.select_cards", return_value=[]
        ):
            run_draft(
                state=state,
                dreamscape_name="Test",
                dreamscape_number=1,
                logger=None,
            )

        # All 5 picks should advance even though offers were empty
        assert state.global_pick_index == initial_pick + 5
        # No cards should be added to the deck
        assert state.deck_count() == 0
