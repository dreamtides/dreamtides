"""Tests for JSONL session logging."""

import json
import os
import tempfile
import time
import unittest
from pathlib import Path
from types import MappingProxyType
from unittest.mock import patch

from models import (
    AlgorithmParams,
    Biome,
    Card,
    CardType,
    DeckCard,
    Dreamcaller,
    DreamscapeNode,
    Dreamsign,
    NodeState,
    Rarity,
    Resonance,
    ResonanceProfile,
    Site,
    SiteType,
)

# Patch _LOG_DIR before importing SessionLogger
import jsonl_log


def _make_card(
    name: str = "Test Card",
    card_number: int = 1,
    energy_cost: int = 3,
    rarity: Rarity = Rarity.COMMON,
    resonances: frozenset[Resonance] = frozenset(),
) -> Card:
    return Card(
        name=name,
        card_number=card_number,
        energy_cost=energy_cost,
        card_type=CardType.CHARACTER,
        subtype=None,
        is_fast=False,
        spark=2,
        rarity=rarity,
        rules_text="Test rules",
        resonances=resonances,
        tags=frozenset(),
    )


class TestHelperFunctions(unittest.TestCase):
    def test_resonance_name(self) -> None:
        self.assertEqual(jsonl_log._resonance_name(Resonance.TIDE), "Tide")
        self.assertEqual(jsonl_log._resonance_name(Resonance.RUIN), "Ruin")

    def test_resonances_list_sorted(self) -> None:
        rs = frozenset({Resonance.RUIN, Resonance.TIDE})
        result = jsonl_log._resonances_list(rs)
        self.assertEqual(result, ["Ruin", "Tide"])

    def test_card_dict(self) -> None:
        card = _make_card(resonances=frozenset({Resonance.TIDE}))
        d = jsonl_log._card_dict(card)
        self.assertEqual(d["name"], "Test Card")
        self.assertEqual(d["card_number"], 1)
        self.assertEqual(d["resonances"], ["Tide"])
        self.assertEqual(d["rarity"], "Common")
        self.assertEqual(d["energy_cost"], 3)
        self.assertEqual(d["spark"], 2)

    def test_profile_dict(self) -> None:
        snapshot = {
            Resonance.TIDE: 5,
            Resonance.EMBER: 3,
            Resonance.ZEPHYR: 0,
            Resonance.STONE: 1,
            Resonance.RUIN: 2,
        }
        result = jsonl_log._profile_dict(snapshot)
        self.assertEqual(result, {
            "Ember": 3,
            "Ruin": 2,
            "Stone": 1,
            "Tide": 5,
            "Zephyr": 0,
        })


class TestSessionLogger(unittest.TestCase):
    def setUp(self) -> None:
        self._tmpdir_obj = tempfile.TemporaryDirectory()
        self._orig_log_dir = jsonl_log._LOG_DIR
        jsonl_log._LOG_DIR = Path(self._tmpdir_obj.name)

    def tearDown(self) -> None:
        jsonl_log._LOG_DIR = self._orig_log_dir
        self._tmpdir_obj.cleanup()

    def _read_events(self, logger: jsonl_log.SessionLogger) -> list[dict]:
        logger.close()
        lines = logger.path.read_text().strip().split("\n")
        return [json.loads(line) for line in lines]

    def test_creates_log_file_with_correct_name(self) -> None:
        logger = jsonl_log.SessionLogger(seed=42)
        self.assertTrue(logger.path.exists())
        self.assertIn("quest_", logger.path.name)
        self.assertIn("_seed42.jsonl", logger.path.name)
        logger.close()

    def test_write_flushes_immediately(self) -> None:
        logger = jsonl_log.SessionLogger(seed=1)
        logger._write({"event": "test"})
        # Read the file while still open
        content = logger.path.read_text()
        self.assertIn('"event":"test"', content)
        logger.close()

    def test_compact_json_separators(self) -> None:
        logger = jsonl_log.SessionLogger(seed=1)
        logger._write({"key": "value", "num": 42})
        content = logger.path.read_text().strip()
        # No spaces after colons or commas
        self.assertNotIn(": ", content)
        self.assertNotIn(", ", content)
        logger.close()

    def test_log_session_start(self) -> None:
        logger = jsonl_log.SessionLogger(seed=42)
        params = AlgorithmParams(
            exponent=1.4,
            floor_weight=0.5,
            neutral_base=3.0,
            staleness_factor=0.3,
        )
        nodes = [
            DreamscapeNode(
                node_id=0,
                name="Nexus",
                biome=Biome.VERDANT,
                sites=[],
                state=NodeState.COMPLETED,
                adjacent=[1, 2],
            ),
            DreamscapeNode(
                node_id=1,
                name="Grove",
                biome=Biome.TWILIGHT,
                sites=[],
                state=NodeState.AVAILABLE,
                adjacent=[0],
            ),
        ]
        logger.log_session_start(seed=42, params=params, atlas_topology=nodes)
        events = self._read_events(logger)
        self.assertEqual(len(events), 1)
        event = events[0]
        self.assertEqual(event["event"], "session_start")
        self.assertEqual(event["seed"], 42)
        self.assertEqual(event["algorithm_params"]["exponent"], 1.4)
        self.assertEqual(len(event["atlas_nodes"]), 2)
        self.assertEqual(event["atlas_nodes"][0]["name"], "Nexus")
        self.assertEqual(event["atlas_nodes"][0]["state"], "Completed")

    def test_log_dreamscape_enter(self) -> None:
        logger = jsonl_log.SessionLogger(seed=1)
        sites = [
            Site(site_type=SiteType.DRAFT, is_enhanced=False),
            Site(site_type=SiteType.SHOP, is_enhanced=True),
        ]
        logger.log_dreamscape_enter("Twilight Grove", 2, sites)
        events = self._read_events(logger)
        event = events[0]
        self.assertEqual(event["event"], "dreamscape_enter")
        self.assertEqual(event["dreamscape_name"], "Twilight Grove")
        self.assertEqual(event["completion_level"], 2)
        self.assertEqual(len(event["sites"]), 2)
        self.assertEqual(event["sites"][1]["site_type"], "Shop")
        self.assertTrue(event["sites"][1]["is_enhanced"])

    def test_log_site_visit(self) -> None:
        logger = jsonl_log.SessionLogger(seed=1)
        logger.log_site_visit(
            site_type="Essence",
            choices=["Accept"],
            choice_made="Accept",
            state_changes={"essence_delta": 200},
        )
        events = self._read_events(logger)
        event = events[0]
        self.assertEqual(event["event"], "site_visit")
        self.assertEqual(event["site_type"], "Essence")
        self.assertEqual(event["state_changes"]["essence_delta"], 200)

    def test_log_draft_pick(self) -> None:
        logger = jsonl_log.SessionLogger(seed=1)
        card_a = _make_card("Card A", card_number=1, resonances=frozenset({Resonance.TIDE}))
        card_b = _make_card("Card B", card_number=2, resonances=frozenset({Resonance.RUIN}))
        profile = {r: 0 for r in Resonance}
        profile[Resonance.TIDE] = 1
        logger.log_draft_pick(
            offered_cards=[card_a, card_b],
            weights=[1.5, 0.8],
            picked_card=card_a,
            profile_snapshot=profile,
        )
        events = self._read_events(logger)
        event = events[0]
        self.assertEqual(event["event"], "draft_pick")
        self.assertEqual(len(event["offered"]), 2)
        self.assertEqual(event["offered"][0]["weight"], 1.5)
        self.assertEqual(event["picked"]["name"], "Card A")
        self.assertEqual(event["profile_after"]["Tide"], 1)

    def test_log_draft_pick_mismatched_lengths(self) -> None:
        logger = jsonl_log.SessionLogger(seed=1)
        card_a = _make_card("Card A")
        profile = {r: 0 for r in Resonance}
        with self.assertRaises(ValueError):
            logger.log_draft_pick(
                offered_cards=[card_a],
                weights=[1.0, 2.0],
                picked_card=card_a,
                profile_snapshot=profile,
            )
        logger.close()

    def test_log_shop_purchase(self) -> None:
        logger = jsonl_log.SessionLogger(seed=1)
        card_a = _make_card("Card A")
        card_b = _make_card("Card B")
        logger.log_shop_purchase(
            items_shown=[card_a, card_b],
            items_bought=[card_a],
            essence_spent=50,
        )
        events = self._read_events(logger)
        event = events[0]
        self.assertEqual(event["event"], "shop_purchase")
        self.assertEqual(len(event["items_shown"]), 2)
        self.assertEqual(len(event["items_bought"]), 1)
        self.assertEqual(event["essence_spent"], 50)

    def test_log_battle_complete(self) -> None:
        logger = jsonl_log.SessionLogger(seed=1)
        card = _make_card("Rare Reward", rarity=Rarity.RARE)
        logger.log_battle_complete("Dream Guardian", 125, card)
        events = self._read_events(logger)
        event = events[0]
        self.assertEqual(event["event"], "battle_complete")
        self.assertEqual(event["opponent_name"], "Dream Guardian")
        self.assertEqual(event["essence_reward"], 125)
        self.assertEqual(event["rare_pick"]["name"], "Rare Reward")

    def test_log_battle_complete_no_rare_pick(self) -> None:
        logger = jsonl_log.SessionLogger(seed=1)
        logger.log_battle_complete("Dream Guardian", 100, None)
        events = self._read_events(logger)
        event = events[0]
        self.assertIsNone(event["rare_pick"])

    def test_log_session_end(self) -> None:
        logger = jsonl_log.SessionLogger(seed=1)
        deck = [
            DeckCard(card=_make_card("A", rarity=Rarity.COMMON)),
            DeckCard(card=_make_card("B", rarity=Rarity.RARE)),
        ]
        profile = ResonanceProfile()
        profile.add(Resonance.TIDE, 3)
        dreamsigns = [
            Dreamsign(
                name="Sign A",
                resonance=Resonance.TIDE,
                tags=frozenset(),
                effect_text="Test",
                is_bane=False,
            ),
        ]
        dreamcaller = Dreamcaller(
            name="Vesper, Twilight Arbiter",
            resonances=frozenset({Resonance.TIDE, Resonance.RUIN}),
            resonance_bonus=MappingProxyType({"Tide": 4, "Ruin": 4}),
            tags=frozenset({"mechanic:reclaim"}),
            tag_bonus=MappingProxyType({"mechanic:reclaim": 2}),
            essence_bonus=50,
            ability_text="Test ability",
        )
        logger.log_session_end(
            deck=deck,
            resonance_profile=profile,
            essence=175,
            completion_level=7,
            dreamsigns=dreamsigns,
            dreamcaller=dreamcaller,
        )
        events = self._read_events(logger)
        event = events[0]
        self.assertEqual(event["event"], "session_end")
        self.assertEqual(event["total_cards"], 2)
        self.assertEqual(event["rarity_breakdown"]["Common"], 1)
        self.assertEqual(event["rarity_breakdown"]["Rare"], 1)
        self.assertEqual(event["essence"], 175)
        self.assertEqual(event["completion_level"], 7)
        self.assertEqual(len(event["dreamsigns"]), 1)
        self.assertEqual(event["dreamsigns"][0]["name"], "Sign A")
        self.assertEqual(event["dreamcaller"], "Vesper, Twilight Arbiter")

    def test_log_session_end_no_dreamcaller(self) -> None:
        logger = jsonl_log.SessionLogger(seed=1)
        logger.log_session_end(
            deck=[],
            resonance_profile=ResonanceProfile(),
            essence=0,
            completion_level=0,
            dreamsigns=[],
            dreamcaller=None,
        )
        events = self._read_events(logger)
        self.assertIsNone(events[0]["dreamcaller"])

    def test_path_property(self) -> None:
        logger = jsonl_log.SessionLogger(seed=99)
        self.assertIsInstance(logger.path, Path)
        self.assertTrue(str(logger.path).endswith(".jsonl"))
        logger.close()

    def test_multiple_events_written(self) -> None:
        logger = jsonl_log.SessionLogger(seed=1)
        logger._write({"event": "a"})
        logger._write({"event": "b"})
        logger._write({"event": "c"})
        events = self._read_events(logger)
        self.assertEqual(len(events), 3)
        self.assertEqual([e["event"] for e in events], ["a", "b", "c"])

    def test_log_site_visit_includes_dreamscape(self) -> None:
        """site_visit events must include dreamscape name."""
        logger = jsonl_log.SessionLogger(seed=1)
        profile = {r: 0 for r in Resonance}
        profile[Resonance.TIDE] = 3
        logger.log_site_visit(
            site_type="Essence",
            dreamscape="Twilight Grove",
            is_enhanced=False,
            choices=[],
            choice_made=None,
            state_changes={"essence_gained": 200},
            profile_snapshot=profile,
        )
        events = self._read_events(logger)
        event = events[0]
        self.assertEqual(event["event"], "site_visit")
        self.assertEqual(event["dreamscape"], "Twilight Grove")

    def test_log_site_visit_includes_is_enhanced(self) -> None:
        """site_visit events must include is_enhanced flag."""
        logger = jsonl_log.SessionLogger(seed=1)
        profile = {r: 0 for r in Resonance}
        logger.log_site_visit(
            site_type="Essence",
            dreamscape="Crystal Spire",
            is_enhanced=True,
            choices=[],
            choice_made=None,
            state_changes={"essence_gained": 400},
            profile_snapshot=profile,
        )
        events = self._read_events(logger)
        event = events[0]
        self.assertTrue(event["is_enhanced"])

    def test_log_site_visit_includes_profile_snapshot(self) -> None:
        """site_visit events must include profile_snapshot."""
        logger = jsonl_log.SessionLogger(seed=1)
        profile = {
            Resonance.TIDE: 5,
            Resonance.EMBER: 3,
            Resonance.ZEPHYR: 0,
            Resonance.STONE: 1,
            Resonance.RUIN: 2,
        }
        logger.log_site_visit(
            site_type="Purge",
            dreamscape="Ashen Depths",
            is_enhanced=True,
            choices=["Card A", "Card B"],
            choice_made="Card A",
            state_changes={"cards_removed": ["Card A"]},
            profile_snapshot=profile,
        )
        events = self._read_events(logger)
        event = events[0]
        self.assertIn("profile_snapshot", event)
        self.assertEqual(event["profile_snapshot"]["Tide"], 5)
        self.assertEqual(event["profile_snapshot"]["Ember"], 3)
        self.assertEqual(event["profile_snapshot"]["Zephyr"], 0)
        self.assertEqual(event["profile_snapshot"]["Stone"], 1)
        self.assertEqual(event["profile_snapshot"]["Ruin"], 2)

    def test_log_site_visit_choices_offered_key(self) -> None:
        """site_visit events must use choices_offered key."""
        logger = jsonl_log.SessionLogger(seed=1)
        profile = {r: 0 for r in Resonance}
        logger.log_site_visit(
            site_type="DreamJourney",
            dreamscape="Twilight Grove",
            is_enhanced=False,
            choices=["Journey A", "Journey B"],
            choice_made="Journey A",
            state_changes={"effect_type": "add_essence"},
            profile_snapshot=profile,
        )
        events = self._read_events(logger)
        event = events[0]
        self.assertIn("choices_offered", event)
        self.assertEqual(event["choices_offered"], ["Journey A", "Journey B"])


if __name__ == "__main__":
    unittest.main()
