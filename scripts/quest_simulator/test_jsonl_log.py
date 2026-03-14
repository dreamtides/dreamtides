"""Tests for JSONL session logging."""

import json
import sys
import tempfile
import unittest
from pathlib import Path

_DRAFT_SIM_DIR = str(Path(__file__).resolve().parent.parent / "draft_simulator_v2")
if _DRAFT_SIM_DIR not in sys.path:
    sys.path.insert(0, _DRAFT_SIM_DIR)

from draft_models import CardDesign, CardInstance
from models import (
    Biome,
    DeckCard,
    Dreamcaller,
    DreamscapeNode,
    Dreamsign,
    NodeState,
    Site,
    SiteType,
)

import jsonl_log

_NEXT_ID = 0


def _make_design(
    name: str = "Test Card",
    card_id: str = "test_001",
    rarity_value: float = 0.33,
    fitness: list[float] | None = None,
) -> CardDesign:
    return CardDesign(
        card_id=card_id,
        name=name,
        fitness=fitness if fitness is not None else [0.8, 0.2, 0.1],
        rarity_value=rarity_value,
    )


def _make_instance(design: CardDesign) -> CardInstance:
    global _NEXT_ID
    _NEXT_ID += 1
    return CardInstance(instance_id=_NEXT_ID, design=design)


class TestDeckCardDict(unittest.TestCase):
    def test_serializes_card_design_fields(self) -> None:
        design = _make_design(
            name="Test Card",
            card_id="test_001",
            rarity_value=0.5,
            fitness=[0.9, 0.2],
        )
        dc = DeckCard(instance=_make_instance(design))
        d = jsonl_log._deck_card_dict(dc)
        self.assertEqual(d["name"], "Test Card")
        self.assertEqual(d["card_id"], "test_001")
        self.assertEqual(d["rarity_value"], 0.5)
        self.assertEqual(d["top_fitness"], [0.9, 0.2])
        self.assertFalse(d["is_bane"])
        self.assertFalse(d["is_transfigured"])

    def test_includes_instance_id(self) -> None:
        design = _make_design()
        instance = _make_instance(design)
        dc = DeckCard(instance=instance)
        d = jsonl_log._deck_card_dict(dc)
        self.assertEqual(d["instance_id"], instance.instance_id)

    def test_bane_flag(self) -> None:
        design = _make_design()
        dc = DeckCard(instance=_make_instance(design), is_bane=True)
        d = jsonl_log._deck_card_dict(dc)
        self.assertTrue(d["is_bane"])

    def test_top_fitness_limited_to_three(self) -> None:
        design = _make_design(fitness=[0.9, 0.8, 0.7, 0.6, 0.5])
        dc = DeckCard(instance=_make_instance(design))
        d = jsonl_log._deck_card_dict(dc)
        top_fitness = d["top_fitness"]
        assert isinstance(top_fitness, list)
        self.assertEqual(len(top_fitness), 3)
        self.assertEqual(top_fitness, [0.9, 0.8, 0.7])


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
        content = logger.path.read_text()
        self.assertIn('"event":"test"', content)
        logger.close()

    def test_compact_json_separators(self) -> None:
        logger = jsonl_log.SessionLogger(seed=1)
        logger._write({"key": "value", "num": 42})
        content = logger.path.read_text().strip()
        self.assertNotIn(": ", content)
        self.assertNotIn(", ", content)
        logger.close()

    def test_log_session_start(self) -> None:
        logger = jsonl_log.SessionLogger(seed=42)
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
        logger.log_session_start(seed=42, atlas_topology=nodes)
        events = self._read_events(logger)
        self.assertEqual(len(events), 1)
        event = events[0]
        self.assertEqual(event["event"], "session_start")
        self.assertEqual(event["seed"], 42)
        self.assertEqual(len(event["atlas_nodes"]), 2)
        self.assertEqual(event["atlas_nodes"][0]["name"], "Nexus")
        self.assertEqual(event["atlas_nodes"][0]["state"], "Completed")

    def test_log_session_start_with_draft_config(self) -> None:
        logger = jsonl_log.SessionLogger(seed=42)
        nodes = [
            DreamscapeNode(
                node_id=0,
                name="Nexus",
                biome=Biome.VERDANT,
                sites=[],
                state=NodeState.AVAILABLE,
                adjacent=[],
            ),
        ]
        draft_cfg: dict[str, object] = {
            "seat_count": 6,
            "pack_size": 20,
            "archetype_count": 8,
        }
        logger.log_session_start(seed=42, atlas_topology=nodes, draft_config=draft_cfg)
        events = self._read_events(logger)
        event = events[0]
        self.assertEqual(event["draft_config"]["seat_count"], 6)
        self.assertEqual(event["draft_config"]["pack_size"], 20)

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
        inst_a = _make_instance(
            _make_design("Card A", card_id="a_001", rarity_value=0.6)
        )
        inst_b = _make_instance(
            _make_design("Card B", card_id="b_001", rarity_value=0.4)
        )
        logger.log_draft_pick(
            offered_cards=[inst_a, inst_b],
            weights=[1.5, 0.8],
            picked_card=inst_a,
        )
        events = self._read_events(logger)
        event = events[0]
        self.assertEqual(event["event"], "draft_pick")
        self.assertEqual(len(event["offered"]), 2)
        self.assertEqual(event["offered"][0]["weight"], 1.5)
        self.assertEqual(event["offered"][0]["name"], "Card A")
        self.assertEqual(event["offered"][0]["card_id"], "a_001")
        self.assertEqual(event["offered"][0]["rarity_value"], 0.6)
        self.assertEqual(event["picked"]["name"], "Card A")
        self.assertEqual(event["picked"]["card_id"], "a_001")

    def test_log_draft_pick_mismatched_lengths(self) -> None:
        logger = jsonl_log.SessionLogger(seed=1)
        inst = _make_instance(_make_design("Card A"))
        with self.assertRaises(ValueError):
            logger.log_draft_pick(
                offered_cards=[inst],
                weights=[1.0, 2.0],
                picked_card=inst,
            )
        logger.close()

    def test_log_shop_purchase(self) -> None:
        logger = jsonl_log.SessionLogger(seed=1)
        inst_a = _make_instance(
            _make_design("Card A", card_id="a_001", rarity_value=0.5)
        )
        inst_b = _make_instance(
            _make_design("Card B", card_id="b_001", rarity_value=0.6)
        )
        logger.log_shop_purchase(
            items_shown=[inst_a, inst_b],
            items_bought=[inst_a],
            essence_spent=50,
        )
        events = self._read_events(logger)
        event = events[0]
        self.assertEqual(event["event"], "shop_purchase")
        self.assertEqual(len(event["items_shown"]), 2)
        self.assertEqual(event["items_shown"][0]["name"], "Card A")
        self.assertEqual(event["items_shown"][0]["card_id"], "a_001")
        self.assertEqual(event["items_shown"][0]["rarity_value"], 0.5)
        self.assertEqual(len(event["items_bought"]), 1)
        self.assertEqual(event["essence_spent"], 50)

    def test_log_battle_complete(self) -> None:
        logger = jsonl_log.SessionLogger(seed=1)
        inst = _make_instance(_make_design("Rare Reward", card_id="rare_001"))
        logger.log_battle_complete("Dream Guardian", 125, inst)
        events = self._read_events(logger)
        event = events[0]
        self.assertEqual(event["event"], "battle_complete")
        self.assertEqual(event["opponent_name"], "Dream Guardian")
        self.assertEqual(event["essence_reward"], 125)
        self.assertEqual(event["rare_pick"], "Rare Reward")

    def test_log_battle_complete_no_rare_pick(self) -> None:
        logger = jsonl_log.SessionLogger(seed=1)
        logger.log_battle_complete("Dream Guardian", 100, None)
        events = self._read_events(logger)
        event = events[0]
        self.assertIsNone(event["rare_pick"])

    def test_log_session_end(self) -> None:
        logger = jsonl_log.SessionLogger(seed=1)
        deck = [
            DeckCard(instance=_make_instance(_make_design("A", card_id="a_001"))),
            DeckCard(instance=_make_instance(_make_design("B", card_id="b_001"))),
        ]
        dreamsigns = [
            Dreamsign(name="Sign A", effect_text="Test", is_bane=False),
        ]
        dreamcaller = Dreamcaller(
            name="Shatter Archetype Dreamcaller",
            archetype="Shatter",
            essence_bonus=50,
            ability_text="Test ability",
        )
        logger.log_session_end(
            deck=deck,
            essence=175,
            completion_level=7,
            dreamsigns=dreamsigns,
            dreamcaller=dreamcaller,
            preference_vector=[0.3, 0.2, 0.1, 0.4],
        )
        events = self._read_events(logger)
        event = events[0]
        self.assertEqual(event["event"], "session_end")
        self.assertEqual(event["total_cards"], 2)
        self.assertEqual(len(event["deck"]), 2)
        self.assertEqual(event["deck"][0]["name"], "A")
        self.assertEqual(event["deck"][0]["card_id"], "a_001")
        self.assertIn("rarity_value", event["deck"][0])
        self.assertIn("top_fitness", event["deck"][0])
        self.assertEqual(event["essence"], 175)
        self.assertEqual(event["completion_level"], 7)
        self.assertEqual(len(event["dreamsigns"]), 1)
        self.assertEqual(event["dreamsigns"][0]["name"], "Sign A")
        self.assertEqual(event["dreamcaller"], "Shatter Archetype Dreamcaller")
        self.assertEqual(event["preference_vector"], [0.3, 0.2, 0.1, 0.4])

    def test_log_session_end_no_dreamcaller(self) -> None:
        logger = jsonl_log.SessionLogger(seed=1)
        logger.log_session_end(
            deck=[],
            essence=0,
            completion_level=0,
            dreamsigns=[],
            dreamcaller=None,
        )
        events = self._read_events(logger)
        self.assertIsNone(events[0]["dreamcaller"])

    def test_log_session_end_no_preference_vector(self) -> None:
        logger = jsonl_log.SessionLogger(seed=1)
        logger.log_session_end(
            deck=[],
            essence=0,
            completion_level=0,
            dreamsigns=[],
            dreamcaller=None,
        )
        events = self._read_events(logger)
        self.assertNotIn("preference_vector", events[0])

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
        logger.log_site_visit(
            site_type="Essence",
            dreamscape="Twilight Grove",
            is_enhanced=False,
            choices=[],
            choice_made=None,
            state_changes={"essence_gained": 200},
        )
        events = self._read_events(logger)
        event = events[0]
        self.assertEqual(event["event"], "site_visit")
        self.assertEqual(event["dreamscape"], "Twilight Grove")

    def test_log_site_visit_includes_is_enhanced(self) -> None:
        """site_visit events must include is_enhanced flag."""
        logger = jsonl_log.SessionLogger(seed=1)
        logger.log_site_visit(
            site_type="Essence",
            dreamscape="Crystal Spire",
            is_enhanced=True,
            choices=[],
            choice_made=None,
            state_changes={"essence_gained": 400},
        )
        events = self._read_events(logger)
        event = events[0]
        self.assertTrue(event["is_enhanced"])

    def test_log_site_visit_choices_offered_key(self) -> None:
        """site_visit events must use choices_offered key."""
        logger = jsonl_log.SessionLogger(seed=1)
        logger.log_site_visit(
            site_type="DreamJourney",
            dreamscape="Twilight Grove",
            is_enhanced=False,
            choices=["Journey A", "Journey B"],
            choice_made="Journey A",
            state_changes={"effect_type": "add_essence"},
        )
        events = self._read_events(logger)
        event = events[0]
        self.assertIn("choices_offered", event)
        self.assertEqual(event["choices_offered"], ["Journey A", "Journey B"])

    def test_log_error(self) -> None:
        logger = jsonl_log.SessionLogger(seed=1)
        logger.log_error(
            site_type="Essence",
            error_message="RuntimeError: something broke\n  ...",
        )
        events = self._read_events(logger)
        event = events[0]
        self.assertEqual(event["event"], "error")
        self.assertEqual(event["site_type"], "Essence")
        self.assertIn("something broke", event["error_message"])

    def test_log_round_start(self) -> None:
        logger = jsonl_log.SessionLogger(seed=1)
        logger.log_round_start(
            round_index=2,
            global_pick_index=20,
            pack_card_count=20,
            seat_count=6,
        )
        events = self._read_events(logger)
        event = events[0]
        self.assertEqual(event["event"], "round_start")
        self.assertEqual(event["round_index"], 2)
        self.assertEqual(event["global_pick_index"], 20)
        self.assertEqual(event["pack_card_count"], 20)
        self.assertEqual(event["seat_count"], 6)

    def test_log_ai_pick(self) -> None:
        logger = jsonl_log.SessionLogger(seed=1)
        inst = _make_instance(
            _make_design("AI Card", card_id="ai_001", rarity_value=0.7)
        )
        alt: dict[str, object] = {"name": "Alt Card", "score": 0.65}
        logger.log_ai_pick(
            seat_index=3,
            round_index=1,
            global_pick_index=15,
            chosen=inst,
            chosen_score=0.72,
            candidates_count=18,
            top_alternatives=[alt],
            was_random=False,
            agent_w_top3=[(2, 0.35), (5, 0.20), (0, 0.15)],
        )
        events = self._read_events(logger)
        event = events[0]
        self.assertEqual(event["event"], "ai_pick")
        self.assertEqual(event["seat_index"], 3)
        self.assertEqual(event["round_index"], 1)
        self.assertEqual(event["global_pick_index"], 15)
        self.assertEqual(event["chosen"]["name"], "AI Card")
        self.assertEqual(event["chosen"]["card_id"], "ai_001")
        self.assertEqual(event["chosen_score"], 0.72)
        self.assertEqual(event["candidates_count"], 18)
        self.assertEqual(len(event["top_alternatives"]), 1)
        self.assertFalse(event["was_random"])
        self.assertEqual(len(event["agent_w_top3"]), 3)
        self.assertEqual(event["agent_w_top3"][0]["archetype"], 2)
        self.assertEqual(event["agent_w_top3"][0]["value"], 0.35)

    def test_log_ai_pick_was_random(self) -> None:
        logger = jsonl_log.SessionLogger(seed=1)
        inst = _make_instance(_make_design("Random Pick"))
        logger.log_ai_pick(
            seat_index=1,
            round_index=0,
            global_pick_index=0,
            chosen=inst,
            chosen_score=0.3,
            candidates_count=20,
            top_alternatives=[],
            was_random=True,
            agent_w_top3=[(0, 0.125)],
        )
        events = self._read_events(logger)
        self.assertTrue(events[0]["was_random"])

    def test_log_show_n_filter(self) -> None:
        logger = jsonl_log.SessionLogger(seed=1)
        shown: list[dict[str, object]] = [
            {"name": "Card A", "score": 0.8},
            {"name": "Card B", "score": 0.7},
        ]
        filtered: list[dict[str, object]] = [{"name": "Card C", "score": 0.3}]
        logger.log_show_n_filter(
            strategy="sharpened_preference",
            pack_size=20,
            shown_count=2,
            shown_cards_with_scores=shown,
            filtered_out_top3=filtered,
            context="draft",
            global_pick_index=5,
            round_index=0,
        )
        events = self._read_events(logger)
        event = events[0]
        self.assertEqual(event["event"], "show_n_filter")
        self.assertEqual(event["strategy"], "sharpened_preference")
        self.assertEqual(event["pack_size"], 20)
        self.assertEqual(event["shown_count"], 2)
        self.assertEqual(len(event["shown_cards_with_scores"]), 2)
        self.assertEqual(event["shown_cards_with_scores"][0]["name"], "Card A")
        self.assertEqual(len(event["filtered_out_top3"]), 1)
        self.assertEqual(event["context"], "draft")
        self.assertEqual(event["global_pick_index"], 5)
        self.assertEqual(event["round_index"], 0)

    def test_log_preference_snapshot(self) -> None:
        logger = jsonl_log.SessionLogger(seed=1)
        logger.log_preference_snapshot(
            global_pick_index=10,
            preference_vector=[0.3, 0.2, 0.1, 0.4],
            top_archetype_index=3,
            concentration=0.4,
        )
        events = self._read_events(logger)
        event = events[0]
        self.assertEqual(event["event"], "preference_snapshot")
        self.assertEqual(event["global_pick_index"], 10)
        self.assertEqual(event["preference_vector"], [0.3, 0.2, 0.1, 0.4])
        self.assertEqual(event["top_archetype_index"], 3)
        self.assertEqual(event["concentration"], 0.4)

    def test_log_draft_pick_enriched(self) -> None:
        """log_draft_pick with optional enrichment kwargs."""
        logger = jsonl_log.SessionLogger(seed=1)
        inst_a = _make_instance(
            _make_design("Card A", card_id="a_001", rarity_value=0.6)
        )
        inst_b = _make_instance(
            _make_design("Card B", card_id="b_001", rarity_value=0.4)
        )
        logger.log_draft_pick(
            offered_cards=[inst_a, inst_b],
            weights=[0.85, 0.65],
            picked_card=inst_a,
            global_pick_index=7,
            round_index=0,
            round_pick_count=3,
            human_w_top3=[(2, 0.35), (5, 0.20), (0, 0.15)],
            context="draft",
        )
        events = self._read_events(logger)
        event = events[0]
        self.assertEqual(event["event"], "draft_pick")
        self.assertEqual(event["global_pick_index"], 7)
        self.assertEqual(event["round_index"], 0)
        self.assertEqual(event["round_pick_count"], 3)
        self.assertEqual(event["context"], "draft")
        self.assertEqual(len(event["human_w_top3"]), 3)
        self.assertEqual(event["human_w_top3"][0]["archetype"], 2)
        self.assertEqual(event["human_w_top3"][0]["value"], 0.35)
        # Weights should use real scores, not 1.0
        self.assertEqual(event["offered"][0]["weight"], 0.85)
        self.assertEqual(event["offered"][1]["weight"], 0.65)

    def test_log_draft_pick_without_enrichment(self) -> None:
        """log_draft_pick without optional kwargs should omit enrichment fields."""
        logger = jsonl_log.SessionLogger(seed=1)
        inst = _make_instance(_make_design("Card A"))
        logger.log_draft_pick(
            offered_cards=[inst],
            weights=[1.0],
            picked_card=inst,
        )
        events = self._read_events(logger)
        event = events[0]
        self.assertEqual(event["event"], "draft_pick")
        self.assertNotIn("global_pick_index", event)
        self.assertNotIn("round_index", event)
        self.assertNotIn("round_pick_count", event)
        self.assertNotIn("human_w_top3", event)
        self.assertNotIn("context", event)


if __name__ == "__main__":
    unittest.main()
