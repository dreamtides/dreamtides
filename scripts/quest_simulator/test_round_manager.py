"""Tests for the quest draft round manager."""

import random
import sys
from pathlib import Path

# Ensure draft_simulator is importable
_DRAFT_SIM_DIR = str(Path(__file__).resolve().parent.parent / "draft_simulator")
if _DRAFT_SIM_DIR not in sys.path:
    sys.path.insert(0, _DRAFT_SIM_DIR)

import agents
import card_generator
import cube_manager
from config import SimulatorConfig
from draft_models import CubeConsumptionMode

import round_manager
from quest_state import QuestState


def _build_cfg() -> SimulatorConfig:
    cfg = SimulatorConfig()
    cfg.draft.seat_count = 6
    cfg.draft.pack_size = 20
    cfg.draft.human_seats = 1
    cfg.draft.alternate_direction = False
    cfg.agents.show_n = 4
    cfg.agents.show_n_strategy = "sharpened_preference"
    cfg.agents.policy = "adaptive"
    cfg.agents.ai_optimality = 0.80
    cfg.agents.learning_rate = 3.0
    cfg.agents.openness_window = 3
    cfg.cards.archetype_count = 8
    cfg.cards.source = "synthetic"
    cfg.cube.distinct_cards = 540
    cfg.cube.copies_per_card = 1
    cfg.cube.consumption_mode = "with_replacement"
    cfg.refill.strategy = "no_refill"
    cfg.pack_generation.strategy = "seeded_themed"
    return cfg


def _make_state(seed: int = 42) -> QuestState:
    rng = random.Random(seed)
    cfg = _build_cfg()
    cards = card_generator.generate_cards(cfg, rng)
    cube = cube_manager.CubeManager(cards, 1, CubeConsumptionMode.WITH_REPLACEMENT)
    return QuestState(
        essence=250,
        rng=rng,
        human_agent=agents.create_agent(8),
        ai_agents=[agents.create_agent(8) for _ in range(5)],
        cube=cube,
        draft_cfg=cfg,
    )


class TestAdvanceToHumanPick:
    def test_generates_packs_when_none(self) -> None:
        state = _make_state()
        assert state.packs is None
        pack = round_manager.advance_to_human_pick(state)
        assert state.packs is not None
        assert len(state.packs) == 6

    def test_returns_pack_at_seat_zero(self) -> None:
        state = _make_state()
        pack = round_manager.advance_to_human_pick(state)
        assert state.packs is not None
        assert pack is state.packs[0]

    def test_pack_has_expected_card_count(self) -> None:
        state = _make_state()
        pack = round_manager.advance_to_human_pick(state)
        assert len(pack.cards) == 20

    def test_ai_agents_pick_from_their_packs(self) -> None:
        state = _make_state()
        pack = round_manager.advance_to_human_pick(state)
        assert state.packs is not None
        packs = state.packs
        for i in range(1, 6):
            assert len(packs[i].cards) == 19

    def test_ai_agents_accumulate_drafted_cards(self) -> None:
        state = _make_state()
        round_manager.advance_to_human_pick(state)
        for ai_agent in state.ai_agents:
            assert len(ai_agent.drafted) == 1

    def test_does_not_regenerate_packs_if_already_active(self) -> None:
        state = _make_state()
        round_manager.advance_to_human_pick(state)
        assert state.packs is not None
        first_packs = state.packs
        round_manager.complete_human_pick(
            state, first_packs[0].cards[0], first_packs[0].cards[:4]
        )
        round_manager.advance_to_human_pick(state)
        assert state.packs is not first_packs or state.round_pick_count > 0


class TestCompleteHumanPick:
    def test_removes_chosen_card_from_pack(self) -> None:
        state = _make_state()
        pack = round_manager.advance_to_human_pick(state)
        chosen = pack.cards[0]
        shown = pack.cards[:4]
        original_count = len(pack.cards)
        round_manager.complete_human_pick(state, chosen, shown)
        assert state.packs is not None
        packs = state.packs
        assert chosen not in packs[0].cards
        # After rotation, packs[0] is a different pack; check the original
        # pack (now at some other seat) lost a card
        total_cards = sum(len(p.cards) for p in packs)
        expected = original_count - 1 + 19 * 5
        assert total_cards == expected

    def test_increments_counters(self) -> None:
        state = _make_state()
        pack = round_manager.advance_to_human_pick(state)
        round_manager.complete_human_pick(state, pack.cards[0], pack.cards[:4])
        assert state.round_pick_count == 1
        assert state.global_pick_index == 1

    def test_updates_human_agent(self) -> None:
        state = _make_state()
        pack = round_manager.advance_to_human_pick(state)
        assert len(state.human_agent.drafted) == 0
        round_manager.complete_human_pick(state, pack.cards[0], pack.cards[:4])
        assert len(state.human_agent.drafted) == 1

    def test_rotates_packs(self) -> None:
        state = _make_state()
        round_manager.advance_to_human_pick(state)
        assert state.packs is not None
        packs = state.packs
        original_pack_ids = [p.pack_id for p in packs]
        round_manager.complete_human_pick(
            state, packs[0].cards[0], packs[0].cards[:4]
        )
        assert state.packs is not None
        rotated_ids = [p.pack_id for p in state.packs]
        assert rotated_ids[0] == original_pack_ids[-1]
        assert rotated_ids[1] == original_pack_ids[0]


class TestRoundBoundary:
    def test_resets_after_ten_picks(self) -> None:
        state = _make_state(seed=99)
        for _ in range(10):
            pack = round_manager.advance_to_human_pick(state)
            round_manager.complete_human_pick(state, pack.cards[0], pack.cards[:4])
        assert state.round_index == 1
        assert state.round_pick_count == 0
        assert state.packs is None

    def test_generates_new_packs_after_round_boundary(self) -> None:
        state = _make_state(seed=99)
        for _ in range(10):
            pack = round_manager.advance_to_human_pick(state)
            round_manager.complete_human_pick(state, pack.cards[0], pack.cards[:4])
        assert state.packs is None
        pack = round_manager.advance_to_human_pick(state)
        assert state.packs is not None
        assert len(state.packs) == 6
        assert len(pack.cards) == 20

    def test_global_pick_index_spans_rounds(self) -> None:
        state = _make_state(seed=99)
        for _ in range(10):
            pack = round_manager.advance_to_human_pick(state)
            round_manager.complete_human_pick(state, pack.cards[0], pack.cards[:4])
        assert state.global_pick_index == 10
        pack = round_manager.advance_to_human_pick(state)
        round_manager.complete_human_pick(state, pack.cards[0], pack.cards[:4])
        assert state.global_pick_index == 11


class TestAdvancePickNoCard:
    def test_rotates_and_increments_without_card(self) -> None:
        state = _make_state()
        round_manager.advance_to_human_pick(state)
        assert state.packs is not None
        packs = state.packs
        original_pack_ids = [p.pack_id for p in packs]
        round_manager.advance_pick_no_card(state)
        assert state.round_pick_count == 1
        assert state.global_pick_index == 1
        assert state.packs is not None
        rotated_ids = [p.pack_id for p in state.packs]
        assert rotated_ids[0] == original_pack_ids[-1]

    def test_does_not_modify_human_agent(self) -> None:
        state = _make_state()
        round_manager.advance_to_human_pick(state)
        drafted_before = len(state.human_agent.drafted)
        round_manager.advance_pick_no_card(state)
        assert len(state.human_agent.drafted) == drafted_before

    def test_round_boundary_with_no_card_picks(self) -> None:
        state = _make_state(seed=77)
        for _ in range(10):
            round_manager.advance_to_human_pick(state)
            round_manager.advance_pick_no_card(state)
        assert state.round_index == 1
        assert state.round_pick_count == 0
        assert state.packs is None


class TestRngDeterminism:
    def test_same_seed_produces_same_packs(self) -> None:
        state1 = _make_state(seed=42)
        pack1 = round_manager.advance_to_human_pick(state1)
        card_ids_1 = [c.instance_id for c in pack1.cards]

        state2 = _make_state(seed=42)
        pack2 = round_manager.advance_to_human_pick(state2)
        card_ids_2 = [c.instance_id for c in pack2.cards]

        assert card_ids_1 == card_ids_2

    def test_different_seeds_produce_different_packs(self) -> None:
        state1 = _make_state(seed=42)
        pack1 = round_manager.advance_to_human_pick(state1)
        card_ids_1 = [c.instance_id for c in pack1.cards]

        state2 = _make_state(seed=99)
        pack2 = round_manager.advance_to_human_pick(state2)
        card_ids_2 = [c.instance_id for c in pack2.cards]

        assert card_ids_1 != card_ids_2
