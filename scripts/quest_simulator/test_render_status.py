"""Tests for the render_status module."""

import os
import sys
import unittest
from typing import Optional

# Ensure NO_COLOR is set before importing render modules so ANSI codes
# are empty strings, making assertions on visible content straightforward.
os.environ["NO_COLOR"] = "1"

sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "draft_simulator"))
sys.path.insert(0, os.path.join(os.path.dirname(__file__)))


class TestArchetypePreferenceFooter(unittest.TestCase):
    def test_contains_top_archetypes(self) -> None:
        from render_status import archetype_preference_footer

        w = [0.1, 0.05, 0.8, 0.03, 0.01, 0.0, 0.005, 0.005]
        result = archetype_preference_footer(w=w, deck_count=8, essence=300)
        self.assertIn("A2", result)  # top archetype by weight

    def test_contains_deck_and_essence(self) -> None:
        from render_status import archetype_preference_footer

        w = [0.0] * 8
        result = archetype_preference_footer(w=w, deck_count=8, essence=300)
        self.assertIn("8", result)
        self.assertIn("300", result)

    def test_has_separator_bookends(self) -> None:
        from render_status import archetype_preference_footer

        w = [0.0] * 8
        result = archetype_preference_footer(w=w, deck_count=5, essence=100)
        lines = result.split("\n")
        self.assertTrue(all(c == "\u2500" for c in lines[0]))
        self.assertTrue(all(c == "\u2500" for c in lines[-1]))

    def test_non_empty_for_zero_weights(self) -> None:
        from render_status import archetype_preference_footer

        w = [0.0] * 8
        result = archetype_preference_footer(w=w, deck_count=0, essence=0)
        self.assertTrue(len(result) > 0)

    def test_shows_percentages(self) -> None:
        from render_status import archetype_preference_footer

        w = [0.1, 0.05, 0.8, 0.03, 0.01, 0.0, 0.005, 0.005]
        result = archetype_preference_footer(w=w, deck_count=5, essence=100)
        self.assertIn("%", result)


class TestSiteHeader(unittest.TestCase):
    def test_contains_dreamscape_name_uppercased(self) -> None:
        from render_status import site_header

        result = site_header(
            dreamscape_name="Twilight Grove",
            site_type_label="Draft Site 1",
            dreamscape_number=3,
        )
        self.assertIn("TWILIGHT GROVE", result)

    def test_contains_site_type_label(self) -> None:
        from render_status import site_header

        result = site_header(
            dreamscape_name="Twilight Grove",
            site_type_label="Draft Site 1",
            dreamscape_number=3,
        )
        self.assertIn("Draft Site 1", result)

    def test_contains_dreamscape_number(self) -> None:
        from render_status import site_header

        result = site_header(
            dreamscape_name="Twilight Grove",
            site_type_label="Draft Site 1",
            dreamscape_number=3,
        )
        self.assertIn("[Dreamscape 3]", result)

    def test_optional_pick_counter(self) -> None:
        from render_status import site_header

        result = site_header(
            dreamscape_name="Twilight Grove",
            site_type_label="Draft Site 1",
            dreamscape_number=3,
            pick_info="Pick 1/5",
        )
        self.assertIn("Pick 1/5", result)

    def test_no_pick_counter(self) -> None:
        from render_status import site_header

        result = site_header(
            dreamscape_name="Crystal Spire",
            site_type_label="Shop",
            dreamscape_number=5,
        )
        self.assertNotIn("Pick", result)
        self.assertIn("CRYSTAL SPIRE", result)
        self.assertIn("Shop", result)

    def test_has_double_separators(self) -> None:
        from render_status import site_header

        result = site_header(
            dreamscape_name="Test",
            site_type_label="Battle",
            dreamscape_number=1,
        )
        lines = result.split("\n")
        self.assertTrue(all(c == "\u2550" for c in lines[0]))
        self.assertTrue(all(c == "\u2550" for c in lines[-1]))


class TestVictoryScreen(unittest.TestCase):
    def _build_victory(
        self,
        w: "Optional[list[float]]" = None,
        log_path: "Optional[str]" = ".logs/quest_test.jsonl",
    ) -> str:
        from render_status import victory_screen

        if w is None:
            w = [0.1, 0.05, 0.8, 0.03, 0.01, 0.0, 0.005, 0.005]
        return victory_screen(
            battles_won=7,
            total_battles=7,
            dreamscapes_visited=7,
            dreamcaller_name="Vesper, Twilight Arbiter",
            deck_size=34,
            dreamsign_count=4,
            essence=175,
            w=w,
            log_path=log_path,
        )

    def test_contains_victory_header(self) -> None:
        result = self._build_victory()
        self.assertIn("QUEST COMPLETE", result)
        self.assertIn("VICTORY", result)

    def test_contains_battle_stats(self) -> None:
        result = self._build_victory()
        self.assertIn("7/7", result)
        self.assertIn("Battles won", result)

    def test_contains_dreamscapes_visited(self) -> None:
        result = self._build_victory()
        self.assertIn("7", result)

    def test_contains_dreamcaller_name(self) -> None:
        result = self._build_victory()
        self.assertIn("Vesper, Twilight Arbiter", result)

    def test_contains_deck_size(self) -> None:
        result = self._build_victory()
        self.assertIn("34", result)

    def test_contains_archetype_preferences(self) -> None:
        result = self._build_victory()
        self.assertIn("Archetype Preferences", result)
        self.assertIn("A2", result)  # highest weight archetype

    def test_contains_dreamsigns_and_essence(self) -> None:
        result = self._build_victory()
        self.assertIn("4", result)
        self.assertIn("175", result)

    def test_contains_log_path(self) -> None:
        result = self._build_victory()
        self.assertIn(".logs/quest_test.jsonl", result)

    def test_no_log_path(self) -> None:
        result = self._build_victory(log_path=None)
        self.assertNotIn("Log written", result)

    def test_has_double_separator_bookends(self) -> None:
        result = self._build_victory()
        lines = result.split("\n")
        self.assertTrue(all(c == "\u2550" for c in lines[0]))
        self.assertTrue(all(c == "\u2550" for c in lines[-1]))

    def test_victory_without_w(self) -> None:
        from render_status import victory_screen

        result = victory_screen(
            battles_won=7,
            total_battles=7,
            dreamscapes_visited=7,
            dreamcaller_name="Luna",
            deck_size=15,
            dreamsign_count=2,
            essence=50,
        )
        self.assertIn("VICTORY", result)
        self.assertNotIn("Archetype", result)

    def test_accepts_w_parameter(self) -> None:
        import inspect
        from render_status import victory_screen

        sig = inspect.signature(victory_screen)
        self.assertIn("w", sig.parameters)


class TestBattleHeader(unittest.TestCase):
    """Tests for the battle_header rendering function."""

    def test_boss_header_contains_battle_number(self) -> None:
        from models import Boss
        from render_status import battle_header

        boss = Boss(
            name="Vesper, Twilight Arbiter",
            archetype="Graveyard Control",
            ability_text="Whenever a card is dissolved, draw a card.",
            deck_description="Control deck.",
            is_final=False,
        )
        result = battle_header(battle_number=4, total_battles=7, boss_info=boss)
        self.assertIn("BATTLE 4", result)

    def test_boss_header_contains_miniboss_label(self) -> None:
        from models import Boss
        from render_status import battle_header

        boss = Boss(
            name="Vesper, Twilight Arbiter",
            archetype="Graveyard Control",
            ability_text="Whenever a card is dissolved, draw a card.",
            deck_description="Control deck.",
            is_final=False,
        )
        result = battle_header(battle_number=4, total_battles=7, boss_info=boss)
        self.assertIn("MINIBOSS ENCOUNTER", result)

    def test_boss_header_contains_final_boss_label(self) -> None:
        from models import Boss
        from render_status import battle_header

        boss = Boss(
            name="Nihil, the Silence Between",
            archetype="Draw-Go Control",
            ability_text="Whenever your opponent plays a card, draw a card.",
            deck_description="Control deck.",
            is_final=True,
        )
        result = battle_header(battle_number=7, total_battles=7, boss_info=boss)
        self.assertIn("FINAL BOSS", result)

    def test_boss_header_contains_opponent_name(self) -> None:
        from models import Boss
        from render_status import battle_header

        boss = Boss(
            name="Vesper, Twilight Arbiter",
            archetype="Graveyard Control",
            ability_text="Whenever a card is dissolved, draw a card.",
            deck_description="Control deck.",
            is_final=False,
        )
        result = battle_header(battle_number=4, total_battles=7, boss_info=boss)
        self.assertIn("Vesper, Twilight Arbiter", result)

    def test_boss_header_contains_archetype(self) -> None:
        from models import Boss
        from render_status import battle_header

        boss = Boss(
            name="Vesper, Twilight Arbiter",
            archetype="Graveyard Control",
            ability_text="Whenever a card is dissolved, draw a card.",
            deck_description="Control deck.",
            is_final=False,
        )
        result = battle_header(battle_number=4, total_battles=7, boss_info=boss)
        self.assertIn("Archetype: Graveyard Control", result)

    def test_boss_header_contains_ability_text(self) -> None:
        from models import Boss
        from render_status import battle_header

        boss = Boss(
            name="Vesper, Twilight Arbiter",
            archetype="Graveyard Control",
            ability_text="Whenever a card is dissolved, draw a card.",
            deck_description="Control deck.",
            is_final=False,
        )
        result = battle_header(battle_number=4, total_battles=7, boss_info=boss)
        self.assertIn("Whenever a card is dissolved, draw a card.", result)

    def test_boss_header_has_double_separators(self) -> None:
        from models import Boss
        from render_status import battle_header

        boss = Boss(
            name="Vesper, Twilight Arbiter",
            archetype="Graveyard Control",
            ability_text="Whenever a card is dissolved, draw a card.",
            deck_description="Control deck.",
            is_final=False,
        )
        result = battle_header(battle_number=4, total_battles=7, boss_info=boss)
        lines = result.split("\n")
        self.assertTrue(all(c == "\u2550" for c in lines[0]))
        self.assertTrue(all(c == "\u2550" for c in lines[-1]))

    def test_guardian_header_contains_battle_number(self) -> None:
        from render_status import battle_header

        result = battle_header(battle_number=1, total_battles=7, boss_info=None)
        self.assertIn("BATTLE 1", result)

    def test_guardian_header_contains_guardian_name(self) -> None:
        from render_status import battle_header

        result = battle_header(battle_number=2, total_battles=7, boss_info=None)
        self.assertIn("Dream Guardian", result)

    def test_guardian_header_no_archetype(self) -> None:
        from render_status import battle_header

        result = battle_header(battle_number=1, total_battles=7, boss_info=None)
        self.assertNotIn("Archetype", result)

    def test_guardian_header_shorter_than_boss(self) -> None:
        from models import Boss
        from render_status import battle_header

        guardian = battle_header(battle_number=1, total_battles=7, boss_info=None)
        boss = Boss(
            name="Vesper, Twilight Arbiter",
            archetype="Graveyard Control",
            ability_text="Whenever a card is dissolved, draw a card.",
            deck_description="Control deck.",
            is_final=False,
        )
        boss_header = battle_header(battle_number=4, total_battles=7, boss_info=boss)
        self.assertLess(len(guardian.split("\n")), len(boss_header.split("\n")))


class TestBattleVictoryMessage(unittest.TestCase):
    def test_contains_victory(self) -> None:
        from render_status import battle_victory_message

        result = battle_victory_message()
        self.assertIn("VICTORY", result)

    def test_has_separator(self) -> None:
        from render_status import battle_victory_message

        result = battle_victory_message()
        has_box_chars = any(
            c in result for c in ["\u2550", "\u2500", "\u2554", "\u2557"]
        )
        self.assertTrue(has_box_chars)


class TestBattleRewardSummary(unittest.TestCase):
    def test_contains_essence_reward(self) -> None:
        from render_status import battle_reward_summary

        result = battle_reward_summary(essence_reward=175, rare_pick_count=3)
        self.assertIn("175", result)

    def test_contains_essence_label(self) -> None:
        from render_status import battle_reward_summary

        result = battle_reward_summary(essence_reward=100, rare_pick_count=3)
        self.assertIn("Essence reward", result)

    def test_contains_battle_reward_label(self) -> None:
        from render_status import battle_reward_summary

        result = battle_reward_summary(essence_reward=100, rare_pick_count=3)
        self.assertIn("Battle Reward", result)


class TestBattleCompletionProgress(unittest.TestCase):
    def test_contains_completion_count(self) -> None:
        from render_status import battle_completion_progress

        result = battle_completion_progress(new_completion=4, total_battles=7)
        self.assertIn("4/7", result)

    def test_contains_completion_label(self) -> None:
        from render_status import battle_completion_progress

        result = battle_completion_progress(new_completion=4, total_battles=7)
        self.assertIn("Completion", result)


class TestNoOldFunctions(unittest.TestCase):
    def test_no_resonance_functions(self) -> None:
        import render_status

        self.assertFalse(hasattr(render_status, "pool_bias_line"))
        self.assertFalse(hasattr(render_status, "pool_composition_summary"))
        self.assertFalse(hasattr(render_status, "algorithm_params_line"))
        self.assertFalse(hasattr(render_status, "profile_bar"))
        self.assertFalse(hasattr(render_status, "resonance_profile_footer"))


class TestImportability(unittest.TestCase):
    def test_wildcard_import(self) -> None:
        """Verify the module can be imported with wildcard syntax."""
        import subprocess

        result = subprocess.run(
            [
                sys.executable,
                "-c",
                (
                    "import sys;"
                    "sys.path.insert(0,'scripts/draft_simulator');"
                    "sys.path.insert(0,'scripts/quest_simulator');"
                    "from render_status import *"
                ),
            ],
            capture_output=True,
            text=True,
            env={**os.environ, "NO_COLOR": "1"},
            cwd=os.path.join(os.path.dirname(__file__), "..", ".."),
        )
        self.assertEqual(result.returncode, 0, f"stderr: {result.stderr}")


if __name__ == "__main__":
    unittest.main()
