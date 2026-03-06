"""Tests for Discovery Draft and Specialty Shop site interactions."""

import random
from typing import Optional
from unittest.mock import patch

from draft_models import CardDesign, CardInstance
from quest_state import QuestState


def _make_design(
    name: str = "Test Card",
    card_id: str = "test_001",
    fitness: Optional[list[float]] = None,
    power: float = 0.5,
    commit: float = 0.3,
    flex: float = 0.2,
) -> CardDesign:
    """Create a CardDesign for testing."""
    if fitness is None:
        fitness = [0.1] * 8
    return CardDesign(
        card_id=card_id,
        name=name,
        fitness=fitness,
        power=power,
        commit=commit,
        flex=flex,
    )


def _make_instance(
    instance_id: int = 0,
    name: str = "Test Card",
    card_id: str = "test_001",
    fitness: Optional[list[float]] = None,
    power: float = 0.5,
) -> CardInstance:
    """Create a CardInstance for testing."""
    design = _make_design(name=name, card_id=card_id, fitness=fitness, power=power)
    return CardInstance(instance_id=instance_id, design=design)


def _make_high_fitness_instances(
    archetype_index: int,
    count: int,
    start_id: int = 0,
    fitness_value: float = 0.9,
    power: float = 0.5,
) -> list[CardInstance]:
    """Create instances with high fitness for a specific archetype."""
    instances = []
    for i in range(count):
        fitness = [0.1] * 8
        fitness[archetype_index] = fitness_value
        design = _make_design(
            name=f"High A{archetype_index} Card {i}",
            card_id=f"high_a{archetype_index}_{i}",
            fitness=fitness,
            power=power,
        )
        instances.append(CardInstance(instance_id=start_id + i, design=design))
    return instances


def _make_quest_state(
    essence: int = 500,
    seed: int = 42,
    high_fitness_archetype: int = 0,
    card_count: int = 100,
) -> QuestState:
    """Create a QuestState with a cube containing cards for testing.

    Creates a mix of high-fitness and low-fitness cards. The human
    agent preference vector is biased toward the given archetype.
    """
    import sys
    from pathlib import Path

    draft_dir = str(Path(__file__).resolve().parent.parent / "draft_simulator")
    if draft_dir not in sys.path:
        sys.path.insert(0, draft_dir)

    import agents
    import cube_manager
    from config import SimulatorConfig
    from draft_models import CubeConsumptionMode

    rng = random.Random(seed)

    # Build designs: half high-fitness, half low-fitness
    designs: list[CardDesign] = []
    half = card_count // 2
    for i in range(half):
        fitness = [0.1] * 8
        fitness[high_fitness_archetype] = 0.9
        designs.append(
            _make_design(
                name=f"HighFit Card {i}",
                card_id=f"highfit_{i}",
                fitness=fitness,
                power=0.3 + (i % 10) * 0.1,
            )
        )
    for i in range(half, card_count):
        fitness = [0.1] * 8  # All low fitness
        designs.append(
            _make_design(
                name=f"LowFit Card {i}",
                card_id=f"lowfit_{i}",
                fitness=fitness,
                power=0.3 + (i % 10) * 0.1,
            )
        )

    cube = cube_manager.CubeManager(
        designs=designs,
        copies_per_card=1,
        consumption_mode=CubeConsumptionMode.WITH_REPLACEMENT,
    )

    cfg = SimulatorConfig()
    cfg.draft.seat_count = 6
    cfg.agents.learning_rate = 3.0
    cfg.agents.openness_window = 3
    cfg.cards.archetype_count = 8

    human_agent = agents.create_agent(archetype_count=8)
    # Bias the preference vector toward the target archetype
    human_agent.w[high_fitness_archetype] = 5.0

    ai_agents = [agents.create_agent(archetype_count=8) for _ in range(5)]

    return QuestState(
        essence=essence,
        rng=rng,
        human_agent=human_agent,
        ai_agents=ai_agents,
        cube=cube,
        draft_cfg=cfg,
    )


class TestDrawAndFilter:
    """Test the archetype-based card drawing and filtering."""

    def test_draw_and_filter_returns_high_fitness_cards(self) -> None:
        """Cards returned should have high fitness for the player's top archetype."""
        from sites_discovery import draw_and_filter

        state = _make_quest_state(high_fitness_archetype=2)
        cards = draw_and_filter(state, count=4)

        assert len(cards) > 0
        assert len(cards) <= 4
        # All returned cards should have fitness >= 0.7 for archetype 2
        for card in cards:
            assert card.design.fitness[2] >= 0.7

    def test_draw_and_filter_with_no_matching_cards_relaxes_threshold(self) -> None:
        """When no cards meet the fitness threshold, should still return cards."""
        import sys
        from pathlib import Path

        draft_dir = str(Path(__file__).resolve().parent.parent / "draft_simulator")
        if draft_dir not in sys.path:
            sys.path.insert(0, draft_dir)

        import agents
        import cube_manager
        from config import SimulatorConfig
        from draft_models import CubeConsumptionMode

        from sites_discovery import draw_and_filter

        # All cards have low fitness everywhere
        designs = [
            _make_design(
                name=f"LowFit {i}",
                card_id=f"low_{i}",
                fitness=[0.2] * 8,
            )
            for i in range(50)
        ]
        cube = cube_manager.CubeManager(
            designs=designs,
            copies_per_card=1,
            consumption_mode=CubeConsumptionMode.WITH_REPLACEMENT,
        )
        cfg = SimulatorConfig()
        cfg.draft.seat_count = 6
        cfg.agents.learning_rate = 3.0
        cfg.agents.openness_window = 3

        human = agents.create_agent(archetype_count=8)
        human.w[0] = 5.0

        state = QuestState(
            essence=500,
            rng=random.Random(42),
            human_agent=human,
            ai_agents=[],
            cube=cube,
            draft_cfg=cfg,
        )

        cards = draw_and_filter(state, count=4)
        # Should still return some cards even without high-fitness matches
        assert len(cards) > 0


class TestPowerBasedPricing:
    """Test the power-based pricing formula for specialty shop."""

    def test_price_formula(self) -> None:
        """Price should be round(power * 25) clamped to [5, 100]."""
        from sites_discovery import compute_power_price

        assert compute_power_price(0.0) == 5  # Clamped to min
        assert compute_power_price(0.2) == 5  # round(5) = 5
        assert compute_power_price(0.5) == 12  # round(12.5) = 12 (banker's rounding)
        assert compute_power_price(1.0) == 25
        assert compute_power_price(2.0) == 50
        assert compute_power_price(4.0) == 100  # round(100) = 100
        assert compute_power_price(5.0) == 100  # Clamped to max

    def test_price_never_below_minimum(self) -> None:
        """Price should never be below 5."""
        from sites_discovery import compute_power_price

        for p in [0.0, 0.01, 0.05, 0.1]:
            assert compute_power_price(p) >= 5

    def test_price_never_above_maximum(self) -> None:
        """Price should never exceed 100."""
        from sites_discovery import compute_power_price

        for p in [10.0, 20.0, 50.0]:
            assert compute_power_price(p) <= 100


class TestShopItemCreation:
    """Test shop item creation with CardInstance and power-based pricing."""

    def test_shop_item_has_card_instance(self) -> None:
        """ShopItem should reference a CardInstance, not PoolEntry."""
        from sites_discovery import ShopItem

        inst = _make_instance(power=1.0)
        item = ShopItem(instance=inst, base_price=25, discounted_price=None)
        assert item.instance is inst
        assert item.base_price == 25

    def test_effective_price_without_discount(self) -> None:
        """Effective price should be base_price when no discount."""
        from sites_discovery import ShopItem

        inst = _make_instance(power=1.0)
        item = ShopItem(instance=inst, base_price=50, discounted_price=None)
        assert item.effective_price == 50

    def test_effective_price_with_discount(self) -> None:
        """Effective price should be discounted_price when set."""
        from sites_discovery import ShopItem

        inst = _make_instance(power=1.0)
        item = ShopItem(instance=inst, base_price=50, discounted_price=30)
        assert item.effective_price == 30


class TestDiscoveryDraftDoesNotAdvanceDraft:
    """Test that Discovery Draft does not consume draft picks."""

    def test_global_pick_index_unchanged(self) -> None:
        """Discovery Draft should not increment global_pick_index."""
        from sites_discovery import run_discovery_draft

        state = _make_quest_state()
        initial_pick_index = state.global_pick_index

        with patch("sites_discovery.single_select", return_value=0):
            run_discovery_draft(
                state=state,
                logger=None,
                dreamscape_name="Test",
                dreamscape_number=1,
                is_enhanced=False,
            )

        assert state.global_pick_index == initial_pick_index

    def test_round_pick_count_unchanged(self) -> None:
        """Discovery Draft should not increment round_pick_count."""
        from sites_discovery import run_discovery_draft

        state = _make_quest_state()
        initial_round_pick = state.round_pick_count

        with patch("sites_discovery.single_select", return_value=0):
            run_discovery_draft(
                state=state,
                logger=None,
                dreamscape_name="Test",
                dreamscape_number=1,
                is_enhanced=False,
            )

        assert state.round_pick_count == initial_round_pick


class TestDiscoveryDraftUpdatesAgent:
    """Test that update_agent_after_pick is called for each card taken."""

    def test_agent_preference_updated_after_pick(self) -> None:
        """Human agent w vector should change after a discovery draft pick."""
        from sites_discovery import run_discovery_draft

        state = _make_quest_state(high_fitness_archetype=0)
        initial_w = list(state.human_agent.w)

        with patch("sites_discovery.single_select", return_value=0):
            run_discovery_draft(
                state=state,
                logger=None,
                dreamscape_name="Test",
                dreamscape_number=1,
                is_enhanced=False,
            )

        # The agent's preference vector should have been updated
        assert state.human_agent.w != initial_w

    def test_card_added_to_deck_after_pick(self) -> None:
        """A card should be added to the deck after each pick."""
        from sites_discovery import run_discovery_draft

        state = _make_quest_state()
        initial_deck = state.deck_count()

        with patch("sites_discovery.single_select", return_value=0):
            run_discovery_draft(
                state=state,
                logger=None,
                dreamscape_name="Test",
                dreamscape_number=1,
                is_enhanced=False,
            )

        assert state.deck_count() > initial_deck


class TestSpecialtyShopDoesNotAdvanceDraft:
    """Test that Specialty Shop does not consume draft picks."""

    def test_global_pick_index_unchanged(self) -> None:
        """Specialty Shop should not increment global_pick_index."""
        from sites_discovery import run_specialty_shop

        state = _make_quest_state(essence=500)
        initial_pick_index = state.global_pick_index

        # Select first item (index 0) to buy
        with patch("sites_discovery.multi_select", return_value=[0]):
            run_specialty_shop(
                state=state,
                logger=None,
                dreamscape_name="Test",
                dreamscape_number=1,
                is_enhanced=False,
                shop_config={"reroll_cost": 50, "items_count": 4,
                             "discount_min": 30, "discount_max": 90},
            )

        assert state.global_pick_index == initial_pick_index


class TestSpecialtyShopPricing:
    """Test that Specialty Shop uses power-based pricing."""

    def test_shop_uses_power_pricing(self) -> None:
        """Specialty Shop items should be priced based on power, not rarity."""
        from sites_discovery import compute_power_price, prepare_shop_items

        instances = [
            _make_instance(instance_id=i, name=f"Card {i}", power=1.0 + i * 0.5)
            for i in range(4)
        ]
        rng = random.Random(42)
        shop_config = {
            "reroll_cost": 50,
            "items_count": 4,
            "discount_min": 30,
            "discount_max": 90,
        }

        items = prepare_shop_items(instances, rng, shop_config)

        for i, item in enumerate(items):
            expected_base = compute_power_price(instances[i].design.power)
            assert item.base_price == expected_base


class TestSpecialtyShopReroll:
    """Test that reroll re-draws from cube without advancing draft."""

    def test_reroll_does_not_advance_draft(self) -> None:
        """Rerolling in Specialty Shop should not advance the draft."""
        from sites_discovery import run_specialty_shop

        state = _make_quest_state(essence=500)
        initial_pick_index = state.global_pick_index

        # First call: select reroll (last index = items_count = 4)
        # Second call: select nothing (empty list to exit)
        call_count = [0]
        def mock_multi_select(options, render_fn=None):
            call_count[0] += 1
            if call_count[0] == 1:
                return [len(options) - 1]  # Select reroll
            return []  # Buy nothing

        with patch("sites_discovery.multi_select", side_effect=mock_multi_select):
            run_specialty_shop(
                state=state,
                logger=None,
                dreamscape_name="Test",
                dreamscape_number=1,
                is_enhanced=False,
                shop_config={"reroll_cost": 50, "items_count": 4,
                             "discount_min": 30, "discount_max": 90},
            )

        assert state.global_pick_index == initial_pick_index


class TestNoOldImports:
    """Test that old imports are removed."""

    def test_no_algorithm_import(self) -> None:
        """sites_discovery should not import algorithm."""
        import importlib
        import sites_discovery
        importlib.reload(sites_discovery)
        assert not hasattr(sites_discovery, "select_cards")

    def test_no_pool_import(self) -> None:
        """sites_discovery should not import pool."""
        import importlib
        import sites_discovery
        importlib.reload(sites_discovery)
        assert not hasattr(sites_discovery, "increment_staleness")
        assert not hasattr(sites_discovery, "remove_entry")

    def test_no_tags_import(self) -> None:
        """sites_discovery should not import tags."""
        import importlib
        import sites_discovery
        importlib.reload(sites_discovery)
        assert not hasattr(sites_discovery, "select_theme")
        assert not hasattr(sites_discovery, "filter_pool_by_tag")
