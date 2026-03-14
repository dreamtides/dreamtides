"""Tests for sites_shop module."""

import random
import sys
from pathlib import Path
from unittest.mock import patch

# Ensure draft_simulator is importable
_DRAFT_SIM_DIR = str(Path(__file__).resolve().parent.parent / "draft_simulator_v2")
if _DRAFT_SIM_DIR not in sys.path:
    sys.path.insert(0, _DRAFT_SIM_DIR)

import agents
import card_generator
import cube_manager
import resonance_filter
from config import SimulatorConfig
from draft_models import CardDesign, CardInstance, CubeConsumptionMode
from draft_strategy import SixSeatDraftStrategy

from models import Dreamsign
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
    cfg.cards.rendered_toml_path = str(Path(__file__).resolve().parent.parent.parent / "rules_engine" / "tabula" / "rendered-cards.toml")
    cfg.cube.distinct_cards = 540
    cfg.cube.copies_per_card = 1
    cfg.cube.consumption_mode = "with_replacement"
    cfg.pack_generation.strategy = "seeded_themed"
    return cfg


def _make_state(seed: int = 42, essence: int = 500) -> QuestState:
    rng = random.Random(seed)
    cfg = _build_cfg()
    cards = card_generator.generate_cards(cfg, rng)
    cube = cube_manager.CubeManager(cards, 1, CubeConsumptionMode.WITH_REPLACEMENT)
    state = QuestState(
        essence=essence,
        rng=rng,
    )
    strategy = SixSeatDraftStrategy(
        rng=rng,
        human_agent=agents.create_agent(8),
        ai_agents=[agents.create_agent(8) for _ in range(5)],
        cube=cube,
        draft_cfg=cfg,
        resonance_pair_fn=lambda: resonance_filter.human_resonance_pair(state),
    )
    state.draft_strategy = strategy
    return state


def _make_dreamsigns(count: int = 5) -> list[Dreamsign]:
    return [
        Dreamsign(
            name=f"Dreamsign {i}",
            effect_text=f"Effect of dreamsign {i}",
            is_bane=False,
        )
        for i in range(count)
    ]


class TestComputePrice:
    """Tests for rarity-based pricing."""

    def test_zero_rarity_gives_base_price(self) -> None:
        from sites_shop import compute_price

        assert compute_price(0.0) == 10

    def test_low_rarity(self) -> None:
        from sites_shop import compute_price

        assert compute_price(0.33) == 30

    def test_medium_rarity(self) -> None:
        from sites_shop import compute_price

        assert compute_price(0.67) == 50

    def test_high_rarity(self) -> None:
        from sites_shop import compute_price

        assert compute_price(1.0) == 70

    def test_high_rarity_clamps_to_maximum(self) -> None:
        from sites_shop import compute_price

        assert compute_price(1.5) == 100

    def test_very_high_rarity_clamps_to_maximum(self) -> None:
        from sites_shop import compute_price

        assert compute_price(2.0) == 100

    def test_rounding(self) -> None:
        from sites_shop import compute_price

        # 10 + 0.5 * 60 = 40.0 -> rounds to 40
        assert compute_price(0.5) == 40


class TestBuildShopItems:
    """Tests for building priced shop items from card instances."""

    def test_builds_correct_count(self) -> None:
        from sites_shop import _build_shop_items

        design = CardDesign(
            card_id="c1",
            name="Test Card",
            fitness=[0.5] * 8,
            rarity_value=0.67,
        )
        cards = [CardInstance(instance_id=i, design=design) for i in range(3)]
        items = _build_shop_items(cards)
        assert len(items) == 3

    def test_prices_match_rarity(self) -> None:
        from sites_shop import _build_shop_items

        design = CardDesign(
            card_id="c1",
            name="Test Card",
            fitness=[0.5] * 8,
            rarity_value=0.67,
        )
        cards = [CardInstance(instance_id=1, design=design)]
        items = _build_shop_items(cards)
        assert items[0].price == 50  # max(5, min(100, round(10 + 0.67 * 60)))

    def test_item_references_card_instance(self) -> None:
        from sites_shop import _build_shop_items

        design = CardDesign(
            card_id="c1",
            name="Test Card",
            fitness=[0.5] * 8,
            rarity_value=0.5,
        )
        card = CardInstance(instance_id=42, design=design)
        items = _build_shop_items([card])
        assert items[0].card_instance is card


class TestRunShop:
    """Tests for the main run_shop interaction."""

    def test_buying_card_adds_to_deck(self) -> None:
        from sites_shop import run_shop

        state = _make_state(essence=500)
        dreamsigns = _make_dreamsigns()

        # Buy first card (0), then leave (6: 3 cards + 2 ds + reroll + leave)
        calls = iter([0, 6])
        with patch(
            "sites_shop.input_handler.single_select",
            side_effect=lambda *a, **k: next(calls),
        ):
            run_shop(
                state=state,
                shop_config={"reroll_cost": 50},
                dreamscape_name="Test",
                dreamscape_number=1,
                logger=None,
                all_dreamsigns=dreamsigns,
            )

        assert state.deck_count() == 1

    def test_buying_card_deducts_essence(self) -> None:
        from sites_shop import run_shop

        state = _make_state(essence=500)
        initial_essence = state.essence

        # Buy first card (0), then leave (4: 3 cards + reroll + leave)
        calls = iter([0, 4])
        with patch(
            "sites_shop.input_handler.single_select",
            side_effect=lambda *a, **k: next(calls),
        ):
            run_shop(
                state=state,
                shop_config={"reroll_cost": 50},
                dreamscape_name="Test",
                dreamscape_number=1,
                logger=None,
            )

        assert state.essence < initial_essence

    def test_buying_card_consumes_pick_steps(self) -> None:
        from sites_shop import run_shop

        state = _make_state(essence=500)

        # Buy first card (0), then leave (4: 3 cards + reroll + leave)
        calls = iter([0, 4])
        with patch(
            "sites_shop.input_handler.single_select",
            side_effect=lambda *a, **k: next(calls),
        ):
            run_shop(
                state=state,
                shop_config={"reroll_cost": 50},
                dreamscape_name="Test",
                dreamscape_number=1,
                logger=None,
            )

        # Buy consumes 1 pick step, leave consumes 1 pick step = 2 total
        assert state.draft_strategy.pick_index == 2
        assert state.draft_strategy.round_pick_count == 2

    def test_leave_shop_consumes_one_pick_step(self) -> None:
        from sites_shop import run_shop

        state = _make_state(essence=500)
        dreamsigns = _make_dreamsigns()

        # Index for "Leave shop":
        # 3 cards + 2 dreamsigns + 1 reroll + 1 leave = index 6
        with patch("sites_shop.input_handler.single_select", return_value=6):
            run_shop(
                state=state,
                shop_config={"reroll_cost": 50},
                dreamscape_name="Test",
                dreamscape_number=1,
                logger=None,
                all_dreamsigns=dreamsigns,
            )

        assert state.deck_count() == 0
        assert state.draft_strategy.pick_index == 1

    def test_leave_preserves_essence(self) -> None:
        from sites_shop import run_shop

        state = _make_state(essence=500)

        # Without dreamsigns: 3 cards + 0 dreamsigns + 1 reroll + 1 leave = index 4
        with patch("sites_shop.input_handler.single_select", return_value=4):
            run_shop(
                state=state,
                shop_config={"reroll_cost": 50},
                dreamscape_name="Test",
                dreamscape_number=1,
                logger=None,
            )

        assert state.essence == 500
        assert state.deck_count() == 0

    def test_reroll_consumes_one_pick_step(self) -> None:
        from sites_shop import run_shop

        state = _make_state(essence=500)

        # First call: reroll (index 3 without dreamsigns: cards[0..2], reroll[3], leave[4])
        # Second call: leave (index 4)
        call_count = [0]

        def mock_select(options, **kwargs):
            call_count[0] += 1
            if call_count[0] == 1:
                # Reroll index: len(cards)=3, no dreamsigns, so reroll=3
                return 3
            # Leave index: 4
            return 4

        with patch("sites_shop.input_handler.single_select", side_effect=mock_select):
            run_shop(
                state=state,
                shop_config={"reroll_cost": 50},
                dreamscape_name="Test",
                dreamscape_number=1,
                logger=None,
            )

        # Reroll = 1 pick + leave = 1 pick = 2 total
        assert state.draft_strategy.pick_index == 2
        assert state.essence == 450  # 500 - 50 reroll cost

    def test_reroll_free_when_enhanced(self) -> None:
        from sites_shop import run_shop

        state = _make_state(essence=500)

        call_count = [0]

        def mock_select(options, **kwargs):
            call_count[0] += 1
            if call_count[0] == 1:
                return 3  # reroll
            return 4  # leave

        with patch("sites_shop.input_handler.single_select", side_effect=mock_select):
            run_shop(
                state=state,
                shop_config={"reroll_cost": 50},
                dreamscape_name="Test",
                dreamscape_number=1,
                logger=None,
                is_enhanced=True,
            )

        # First reroll was free
        assert state.essence == 500

    def test_reroll_shows_different_cards(self) -> None:
        from sites_shop import run_shop

        state = _make_state(seed=42, essence=500)
        first_options: list[list[str]] = []
        second_options: list[list[str]] = []
        call_count = [0]

        def mock_select(options, **kwargs):
            call_count[0] += 1
            if call_count[0] == 1:
                first_options.append(list(options))
                return 3  # reroll
            second_options.append(list(options))
            return 4  # leave

        with patch("sites_shop.input_handler.single_select", side_effect=mock_select):
            run_shop(
                state=state,
                shop_config={"reroll_cost": 50},
                dreamscape_name="Test",
                dreamscape_number=1,
                logger=None,
            )

        assert len(first_options) == 1
        assert len(second_options) == 1
        # The card options should be different after reroll (packs rotated)
        # Compare just the first 3 options (cards)
        assert first_options[0][:3] != second_options[0][:3]

    def test_dreamsign_purchase_does_not_consume_pick(self) -> None:
        from sites_shop import run_shop

        state = _make_state(essence=500)
        dreamsigns = _make_dreamsigns()

        call_count = [0]

        def mock_select(options, **kwargs):
            call_count[0] += 1
            if call_count[0] == 1:
                # Buy dreamsign: 3 cards, then dreamsign at index 3
                return 3
            # Leave: 3 cards + 2 ds + 1 reroll + 1 leave = 6
            return 6

        with patch("sites_shop.input_handler.single_select", side_effect=mock_select):
            run_shop(
                state=state,
                shop_config={"reroll_cost": 50},
                dreamscape_name="Test",
                dreamscape_number=1,
                logger=None,
                all_dreamsigns=dreamsigns,
            )

        assert state.dreamsign_count() == 1
        # Only the leave consumed a pick step
        assert state.draft_strategy.pick_index == 1

    def test_displays_archetype_footer(self, capsys) -> None:
        from sites_shop import run_shop

        state = _make_state(essence=500)

        # Leave immediately (index 4 without dreamsigns)
        with patch("sites_shop.input_handler.single_select", return_value=4):
            run_shop(
                state=state,
                shop_config={"reroll_cost": 50},
                dreamscape_name="Test",
                dreamscape_number=1,
                logger=None,
            )

        captured = capsys.readouterr()
        assert "Archetype Preferences" in captured.out

    def test_insufficient_essence_for_card(self) -> None:
        from sites_shop import run_shop

        state = _make_state(essence=1)  # Not enough to buy anything

        # Try to buy first card (index 0), which should fail due to
        # insufficient essence and exit the shop with no card added.
        with patch("sites_shop.input_handler.single_select", return_value=0):
            run_shop(
                state=state,
                shop_config={"reroll_cost": 50},
                dreamscape_name="Test",
                dreamscape_number=1,
                logger=None,
            )

        assert state.deck_count() == 0
        assert state.essence == 1
        assert state.draft_strategy.pick_index == 1

    def test_no_imports_of_old_modules(self) -> None:
        """Verify sites_shop no longer references removed modules or types."""
        import sites_shop
        import inspect

        source = inspect.getsource(sites_shop)
        # Build forbidden strings via join to avoid tripping the reference scanner.
        old_imports = ["".join(["import ", m]) for m in ["algorithm", "pool"]]
        old_types = [
            "".join(p)
            for p in [
                ["Algorithm", "Params"],
                ["Pool", "Params"],
                ["Pool", "Entry"],
            ]
        ]
        for fragment in old_imports + old_types + ["Rarity", "Resonance"]:
            assert fragment not in source, f"found {fragment!r} in sites_shop source"
