"""End-to-end playthrough tests for the quest simulator.

Verifies that the full quest loop runs from start to victory for multiple
seeds, produces valid JSONL logs with all required event types, and
demonstrates deterministic behavior when the same seed is used.
"""

import json
import random
import sys
from pathlib import Path
from typing import Any
from unittest.mock import patch

# Ensure draft_simulator is importable before importing quest modules
# that depend on it (e.g. flow -> render -> colors).
_DRAFT_SIM_DIR = str(Path(__file__).resolve().parent.parent / "draft_simulator")
if _DRAFT_SIM_DIR not in sys.path:
    sys.path.insert(0, _DRAFT_SIM_DIR)

import agents
import atlas
import card_generator
import cube_manager
import data_loader
import flow
import render_status
from jsonl_log import SessionLogger
from quest_state import QuestState
from site_dispatch import SiteData
from draft_models import CubeConsumptionMode
from quest_sim import _build_draft_config, draft_config_summary


def _run_full_quest(seed: int) -> tuple[QuestState, Path]:
    """Run a complete quest with the given seed and return final state and log path.

    Uses real data files and the full flow loop in non-interactive mode.
    Returns (quest_state, jsonl_log_path).
    """
    rng = random.Random(seed)

    # Build draft engine
    cfg = _build_draft_config()
    cards = card_generator.generate_cards(cfg, rng)
    cube = cube_manager.CubeManager(
        designs=cards,
        copies_per_card=1,
        consumption_mode=CubeConsumptionMode.WITH_REPLACEMENT,
    )
    human_agent = agents.create_agent(archetype_count=cfg.cards.archetype_count)
    ai_agents = [
        agents.create_agent(archetype_count=cfg.cards.archetype_count)
        for _ in range(cfg.draft.seat_count - 1)
    ]

    # Load quest data
    config = data_loader.load_config()
    dreamcallers = data_loader.load_dreamcallers()
    dreamsigns = data_loader.load_dreamsigns()
    journeys = data_loader.load_journeys()
    offers = data_loader.load_offers()
    banes = data_loader.load_banes()
    bosses = data_loader.load_bosses()

    quest_config: dict[str, Any] = config.get("quest", {})
    starting_essence: int = int(quest_config.get("starting_essence", 250))
    max_deck: int = int(quest_config.get("max_deck", 50))
    min_deck: int = int(quest_config.get("min_deck", 25))
    max_dreamsigns: int = int(quest_config.get("max_dreamsigns", 12))
    total_battles: int = int(quest_config.get("total_battles", 7))

    state = QuestState(
        essence=starting_essence,
        rng=rng,
        human_agent=human_agent,
        ai_agents=ai_agents,
        cube=cube,
        draft_cfg=cfg,
        packs=None,
        max_deck=max_deck,
        min_deck=min_deck,
        max_dreamsigns=max_dreamsigns,
    )

    data = SiteData(
        dreamcallers=dreamcallers,
        dreamsigns=dreamsigns,
        journeys=journeys,
        offers=offers,
        banes=banes,
        bosses=bosses,
        config=config,
    )

    nodes = atlas.initialize_atlas(rng)

    logger = SessionLogger(seed)
    logger.log_session_start(
        seed,
        nodes,
        draft_config=draft_config_summary(cfg),
    )

    flow.run_quest(
        state=state,
        nodes=nodes,
        data=data,
        total_battles=total_battles,
        logger=logger,
    )

    logger.close()
    return state, logger.path


def _parse_log(log_path: Path) -> list[dict[str, Any]]:
    """Parse a JSONL log file into a list of event dicts."""
    events: list[dict[str, Any]] = []
    with open(log_path) as f:
        for line in f:
            if line.strip():
                events.append(json.loads(line))
    return events


class TestFullPlaythrough:
    """Test complete quest playthroughs with real data."""

    def test_seed_42_completes_to_victory(self) -> None:
        """Seed 42 reaches completion level 7 (victory)."""
        with patch("builtins.print"):
            state, log_path = _run_full_quest(42)
        assert state.completion_level == 7
        assert state.deck_count() > 0
        assert state.dreamcaller is not None
        if log_path.exists():
            log_path.unlink()

    def test_seed_1_completes_to_victory(self) -> None:
        """Seed 1 reaches completion level 7 (victory)."""
        with patch("builtins.print"):
            state, log_path = _run_full_quest(1)
        assert state.completion_level == 7
        assert state.deck_count() > 0
        assert state.dreamcaller is not None
        if log_path.exists():
            log_path.unlink()

    def test_seed_100_completes_to_victory(self) -> None:
        """Seed 100 reaches completion level 7 (victory)."""
        with patch("builtins.print"):
            state, log_path = _run_full_quest(100)
        assert state.completion_level == 7
        assert state.deck_count() > 0
        assert state.dreamcaller is not None
        if log_path.exists():
            log_path.unlink()

    def test_victory_state_has_reasonable_deck_size(self) -> None:
        """Final deck size should be between min_deck and some reasonable upper bound."""
        with patch("builtins.print"):
            state, log_path = _run_full_quest(42)
        assert state.deck_count() >= state.min_deck
        assert state.deck_count() <= 100
        if log_path.exists():
            log_path.unlink()

    def test_victory_state_has_essence(self) -> None:
        """Final essence should be non-negative."""
        with patch("builtins.print"):
            state, log_path = _run_full_quest(42)
        assert state.essence >= 0
        if log_path.exists():
            log_path.unlink()


class TestJsonlLogCompleteness:
    """Test that JSONL logs contain all required event types."""

    def test_log_contains_session_start(self) -> None:
        with patch("builtins.print"):
            _, log_path = _run_full_quest(42)
        events = _parse_log(log_path)
        types = [e["event"] for e in events]
        assert "session_start" in types
        log_path.unlink()

    def test_log_contains_session_end(self) -> None:
        with patch("builtins.print"):
            _, log_path = _run_full_quest(42)
        events = _parse_log(log_path)
        types = [e["event"] for e in events]
        assert "session_end" in types
        log_path.unlink()

    def test_log_contains_site_visits(self) -> None:
        with patch("builtins.print"):
            _, log_path = _run_full_quest(42)
        events = _parse_log(log_path)
        site_visits = [e for e in events if e["event"] == "site_visit"]
        assert len(site_visits) > 10
        log_path.unlink()

    def test_log_contains_draft_picks(self) -> None:
        with patch("builtins.print"):
            _, log_path = _run_full_quest(42)
        events = _parse_log(log_path)
        draft_picks = [e for e in events if e["event"] == "draft_pick"]
        assert len(draft_picks) > 5
        log_path.unlink()

    def test_log_contains_shop_site_visits(self) -> None:
        """Shop site visits should appear in the log even without purchases."""
        with patch("builtins.print"):
            _, log_path = _run_full_quest(42)
        events = _parse_log(log_path)
        shop_visits = [
            e
            for e in events
            if e["event"] == "site_visit"
            and e.get("site_type") in ("Shop", "SpecialtyShop")
        ]
        assert (
            len(shop_visits) > 0
        ), "No Shop or SpecialtyShop site_visit events found in the log"
        log_path.unlink()

    def test_shop_purchase_only_logged_when_items_bought(self) -> None:
        """shop_purchase events should only be emitted when items are bought."""
        with patch("builtins.print"):
            _, log_path = _run_full_quest(42)
        events = _parse_log(log_path)
        shop_events = [e for e in events if e["event"] == "shop_purchase"]
        for e in shop_events:
            assert (
                len(e.get("items_bought", [])) > 0
            ), "shop_purchase event logged with empty items_bought"
        log_path.unlink()

    def test_log_contains_battle_completes(self) -> None:
        with patch("builtins.print"):
            _, log_path = _run_full_quest(42)
        events = _parse_log(log_path)
        battles = [e for e in events if e["event"] == "battle_complete"]
        assert len(battles) == 7
        log_path.unlink()

    def test_log_contains_all_required_event_types(self) -> None:
        with patch("builtins.print"):
            _, log_path = _run_full_quest(42)
        events = _parse_log(log_path)
        types = set(e["event"] for e in events)
        required = {
            "session_start",
            "site_visit",
            "draft_pick",
            "battle_complete",
            "session_end",
        }
        missing = required - types
        assert not missing, f"Missing event types: {missing}"
        log_path.unlink()

    def test_log_has_dreamscape_enter_events(self) -> None:
        with patch("builtins.print"):
            _, log_path = _run_full_quest(42)
        events = _parse_log(log_path)
        enters = [e for e in events if e["event"] == "dreamscape_enter"]
        assert len(enters) == 7
        log_path.unlink()


class TestDeterminism:
    """Test that same seed produces identical quest progression."""

    def test_same_seed_same_events(self) -> None:
        """Two runs with seed 42 should produce identical event sequences."""
        with patch("builtins.print"):
            _, log1_path = _run_full_quest(42)
        with patch("builtins.print"):
            _, log2_path = _run_full_quest(42)

        events1 = _parse_log(log1_path)
        events2 = _parse_log(log2_path)

        types1 = [e["event"] for e in events1]
        types2 = [e["event"] for e in events2]
        assert types1 == types2

        # Dreamscape names should match
        ds1 = [
            e["dreamscape_name"] for e in events1 if e["event"] == "dreamscape_enter"
        ]
        ds2 = [
            e["dreamscape_name"] for e in events2 if e["event"] == "dreamscape_enter"
        ]
        assert ds1 == ds2

        log1_path.unlink()
        log2_path.unlink()

    def test_different_seeds_different_paths(self) -> None:
        """Seeds 42 and 1 should produce different dreamscape sequences."""
        with patch("builtins.print"):
            _, log1_path = _run_full_quest(42)
        with patch("builtins.print"):
            _, log2_path = _run_full_quest(1)

        events1 = _parse_log(log1_path)
        events2 = _parse_log(log2_path)

        ds1 = [
            e["dreamscape_name"] for e in events1 if e["event"] == "dreamscape_enter"
        ]
        ds2 = [
            e["dreamscape_name"] for e in events2 if e["event"] == "dreamscape_enter"
        ]
        assert ds1 != ds2

        log1_path.unlink()
        log2_path.unlink()

    def test_same_seed_same_deck_composition(self) -> None:
        """Two runs with the same seed should produce identical final deck."""
        with patch("builtins.print"):
            state1, log1_path = _run_full_quest(42)
        with patch("builtins.print"):
            state2, log2_path = _run_full_quest(42)

        # Extract deck card names from DeckCard instances
        deck1_names = sorted(
            (
                dc.instance.design.name
                if hasattr(dc.instance, "design")
                else str(dc.instance)
            )
            for dc in state1.deck
        )
        deck2_names = sorted(
            (
                dc.instance.design.name
                if hasattr(dc.instance, "design")
                else str(dc.instance)
            )
            for dc in state2.deck
        )
        assert deck1_names == deck2_names

        assert state1.essence == state2.essence
        assert state1.completion_level == state2.completion_level

        log1_path.unlink()
        log2_path.unlink()


class TestVictoryScreen:
    """Test that the victory screen contains all required information."""

    def test_victory_screen_has_battles_won(self) -> None:
        with patch("builtins.print") as mock_print:
            state, log_path = _run_full_quest(42)
        output = _collect_printed_text(mock_print)
        assert "Battles won: 7/7" in output
        log_path.unlink()

    def test_victory_screen_has_quest_complete(self) -> None:
        with patch("builtins.print") as mock_print:
            state, log_path = _run_full_quest(42)
        output = _collect_printed_text(mock_print)
        assert "QUEST COMPLETE" in output
        log_path.unlink()

    def test_victory_screen_has_dreamcaller(self) -> None:
        with patch("builtins.print") as mock_print:
            state, log_path = _run_full_quest(42)
        output = _collect_printed_text(mock_print)
        assert "Dreamcaller:" in output
        log_path.unlink()

    def test_victory_screen_has_dreamsigns(self) -> None:
        with patch("builtins.print") as mock_print:
            state, log_path = _run_full_quest(42)
        output = _collect_printed_text(mock_print)
        assert "Dreamsigns:" in output
        log_path.unlink()

    def test_victory_screen_has_essence(self) -> None:
        with patch("builtins.print") as mock_print:
            state, log_path = _run_full_quest(42)
        output = _collect_printed_text(mock_print)
        assert "Essence remaining:" in output
        log_path.unlink()

    def test_victory_screen_has_log_path(self) -> None:
        with patch("builtins.print") as mock_print:
            state, log_path = _run_full_quest(42)
        output = _collect_printed_text(mock_print)
        assert "Log written to" in output
        log_path.unlink()


class TestSiteTypeCoverage:
    """Test that multiple site types are exercised across different seeds."""

    def test_all_site_types_exercisable(self) -> None:
        """All 16 site types should appear across a range of seeds."""
        all_site_types: set[str] = set()
        for seed in [42, 1, 100, 5, 7]:
            with patch("builtins.print"):
                _, log_path = _run_full_quest(seed)
            events = _parse_log(log_path)
            for e in events:
                if e["event"] == "site_visit":
                    all_site_types.add(e["site_type"])
            log_path.unlink()

        required = {
            "DreamcallerDraft",
            "Draft",
            "Shop",
            "DiscoveryDraft",
            "SpecialtyShop",
            "DreamsignOffering",
            "DreamsignDraft",
            "DreamJourney",
            "TemptingOffer",
            "Purge",
            "Essence",
            "Transfiguration",
            "Duplication",
            "RewardSite",
            "Cleanse",
            "Battle",
        }
        missing = required - all_site_types
        assert not missing, f"Missing site types: {missing}"


def _collect_printed_text(mock_print: Any) -> str:
    """Collect all text passed to print() during a mocked run."""
    parts: list[str] = []
    for call in mock_print.call_args_list:
        for arg in call.args:
            parts.append(str(arg))
    return "\n".join(parts)
