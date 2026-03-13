"""Tests for TOML card loading and gap-filling in card_generator."""

import os
import random
import sys
import tempfile
import unittest

os.environ["NO_COLOR"] = "1"

sys.path.insert(0, os.path.dirname(__file__))

from card_generator import duplicate_real_cards, fill_card_pool_gaps, load_real_cards
from config import SimulatorConfig
from draft_models import CardDesign


def _write_fixture_files(
    tmp_dir: str,
) -> tuple[str, str]:
    """Write small fixture TOML files for testing."""
    rendered_toml = os.path.join(tmp_dir, "rendered-cards.toml")
    metadata_toml = os.path.join(tmp_dir, "card-metadata.toml")

    with open(rendered_toml, "w") as f:
        f.write("""\
[[cards]]
name = "Test Common Card"
id = "aaaa-1111"
energy-cost = 2
card-type = "Event"
subtype = ""
is-fast = false
spark = ""
rarity = "Common"
rendered-text = "Draw a card."

[[cards]]
name = "Test Rare Card"
id = "bbbb-2222"
energy-cost = 5
card-type = "Character"
subtype = "Ancient"
is-fast = false
spark = 3
rarity = "Rare"
rendered-text = "When you play this, gain 2 energy."

[[cards]]
name = "Test Legendary"
id = "cccc-3333"
energy-cost = 7
card-type = "Character"
subtype = "Dragon"
is-fast = true
spark = 5
rarity = "Legendary"
rendered-text = "Dissolve all enemies."

[[cards]]
name = "Special Token"
id = "dddd-4444"
energy-cost = 0
card-type = "Event"
subtype = ""
is-fast = false
spark = ""
rarity = "Special"
rendered-text = "This is a token."
""")

    with open(metadata_toml, "w") as f:
        f.write("""\
[[card-metadata]]
card-id = "aaaa-1111"
flash = 0.8
awaken = 0.1
flicker = 0.05
ignite = 0
shatter = 0
endure = 0
submerge = 0
surge = 0
power = 0.4
commit = 0.3
flex = 0.2

[[card-metadata]]
card-id = "bbbb-2222"
flash = 0
awaken = 0
flicker = 0
ignite = 0.9
shatter = 0.6
endure = 0
submerge = 0
surge = 0
power = 0.7
commit = 0.5
flex = 0.3

[[card-metadata]]
card-id = "cccc-3333"
flash = 0
awaken = 0
flicker = 0
ignite = 0
shatter = 0
endure = 0.95
submerge = 0
surge = 0
power = 0.9
commit = 0.8
flex = 0.1

[[card-metadata]]
card-id = "dddd-4444"
flash = 0
awaken = 0
flicker = 0
ignite = 0
shatter = 0
endure = 0
submerge = 0
surge = 0
power = 0.1
commit = 0.1
flex = 0.9
""")

    return rendered_toml, metadata_toml


class TestLoadRealCards(unittest.TestCase):
    """Tests for load_real_cards()."""

    def setUp(self) -> None:
        self.tmp_dir = tempfile.mkdtemp()
        self.rendered_path, self.metadata_path = _write_fixture_files(self.tmp_dir)

    def test_loads_correct_count(self) -> None:
        cards = load_real_cards(self.rendered_path, self.metadata_path)
        self.assertEqual(len(cards), 3)  # Special card excluded

    def test_special_cards_excluded(self) -> None:
        cards = load_real_cards(self.rendered_path, self.metadata_path)
        names = [c.name for c in cards]
        self.assertNotIn("Special Token", names)

    def test_legendary_maps_to_rare(self) -> None:
        cards = load_real_cards(self.rendered_path, self.metadata_path)
        legendary_card = [c for c in cards if c.name == "Test Legendary"][0]
        self.assertEqual(legendary_card.rarity, "rare")

    def test_common_rarity_lowercase(self) -> None:
        cards = load_real_cards(self.rendered_path, self.metadata_path)
        common_card = [c for c in cards if c.name == "Test Common Card"][0]
        self.assertEqual(common_card.rarity, "common")

    def test_is_real_flag_set(self) -> None:
        cards = load_real_cards(self.rendered_path, self.metadata_path)
        self.assertTrue(all(c.is_real for c in cards))

    def test_rules_text_populated(self) -> None:
        cards = load_real_cards(self.rendered_path, self.metadata_path)
        common_card = [c for c in cards if c.name == "Test Common Card"][0]
        self.assertEqual(common_card.rules_text, "Draw a card.")

    def test_energy_cost_populated(self) -> None:
        cards = load_real_cards(self.rendered_path, self.metadata_path)
        rare_card = [c for c in cards if c.name == "Test Rare Card"][0]
        self.assertEqual(rare_card.energy_cost, 5)

    def test_empty_spark_is_none(self) -> None:
        cards = load_real_cards(self.rendered_path, self.metadata_path)
        common_card = [c for c in cards if c.name == "Test Common Card"][0]
        self.assertIsNone(common_card.spark)

    def test_numeric_spark_parsed(self) -> None:
        cards = load_real_cards(self.rendered_path, self.metadata_path)
        rare_card = [c for c in cards if c.name == "Test Rare Card"][0]
        self.assertEqual(rare_card.spark, 3)

    def test_card_type_populated(self) -> None:
        cards = load_real_cards(self.rendered_path, self.metadata_path)
        common_card = [c for c in cards if c.name == "Test Common Card"][0]
        self.assertEqual(common_card.card_type, "Event")

    def test_is_fast_populated(self) -> None:
        cards = load_real_cards(self.rendered_path, self.metadata_path)
        legendary = [c for c in cards if c.name == "Test Legendary"][0]
        self.assertTrue(legendary.is_fast)

    def test_fitness_vector_from_metadata(self) -> None:
        cards = load_real_cards(self.rendered_path, self.metadata_path)
        common_card = [c for c in cards if c.name == "Test Common Card"][0]
        self.assertAlmostEqual(common_card.fitness[0], 0.8)  # flash


class TestFillCardPoolGaps(unittest.TestCase):
    """Tests for fill_card_pool_gaps()."""

    def _make_real_cards(self, count: int) -> list[CardDesign]:
        cards = []
        for i in range(count):
            arch = i % 8
            fitness = [0.0] * 8
            fitness[arch] = 0.7
            cards.append(
                CardDesign(
                    card_id=f"real_{i:04d}",
                    name=f"Real Card {i}",
                    fitness=fitness,
                    power=0.5,
                    commit=0.4,
                    flex=0.3,
                    rarity="common",
                    is_real=True,
                )
            )
        return cards

    def test_fills_to_target_count(self) -> None:
        cfg = SimulatorConfig()
        cfg.cube.distinct_cards = 360
        real_cards = self._make_real_cards(220)
        result = fill_card_pool_gaps(real_cards, cfg, random.Random(42))
        self.assertEqual(len(result), 360)

    def test_preserves_real_cards(self) -> None:
        cfg = SimulatorConfig()
        cfg.cube.distinct_cards = 360
        real_cards = self._make_real_cards(220)
        result = fill_card_pool_gaps(real_cards, cfg, random.Random(42))
        real_in_result = [c for c in result if c.is_real]
        self.assertEqual(len(real_in_result), 220)

    def test_synthetic_cards_not_real(self) -> None:
        cfg = SimulatorConfig()
        cfg.cube.distinct_cards = 360
        real_cards = self._make_real_cards(220)
        result = fill_card_pool_gaps(real_cards, cfg, random.Random(42))
        synthetic = [c for c in result if not c.is_real]
        self.assertEqual(len(synthetic), 140)
        for c in synthetic:
            self.assertFalse(c.is_real)

    def test_no_gap_returns_original(self) -> None:
        cfg = SimulatorConfig()
        cfg.cube.distinct_cards = 100
        real_cards = self._make_real_cards(100)
        result = fill_card_pool_gaps(real_cards, cfg, random.Random(42))
        self.assertEqual(len(result), 100)

    def test_synthetic_cards_have_rarity(self) -> None:
        cfg = SimulatorConfig()
        cfg.cube.distinct_cards = 360
        real_cards = self._make_real_cards(220)
        result = fill_card_pool_gaps(real_cards, cfg, random.Random(42))
        synthetic = [c for c in result if not c.is_real]
        for c in synthetic:
            self.assertIn(c.rarity, ["common", "uncommon", "rare"])


class TestDuplicateRealCards(unittest.TestCase):
    """Tests for duplicate_real_cards()."""

    def _make_real_cards(self, count: int) -> list[CardDesign]:
        cards = []
        for i in range(count):
            arch = i % 8
            fitness = [0.0] * 8
            fitness[arch] = 0.7
            cards.append(
                CardDesign(
                    card_id=f"real_{i:04d}",
                    name=f"Real Card {i}",
                    fitness=fitness,
                    power=0.5,
                    commit=0.4,
                    flex=0.3,
                    rarity="common",
                    is_real=True,
                )
            )
        return cards

    def test_fills_to_target_count(self) -> None:
        cfg = SimulatorConfig()
        cfg.cube.distinct_cards = 360
        real_cards = self._make_real_cards(220)
        result = duplicate_real_cards(real_cards, cfg, random.Random(42))
        self.assertEqual(len(result), 360)

    def test_all_cards_are_real(self) -> None:
        cfg = SimulatorConfig()
        cfg.cube.distinct_cards = 360
        real_cards = self._make_real_cards(220)
        result = duplicate_real_cards(real_cards, cfg, random.Random(42))
        for c in result:
            self.assertTrue(c.is_real)

    def test_unique_card_ids(self) -> None:
        cfg = SimulatorConfig()
        cfg.cube.distinct_cards = 360
        real_cards = self._make_real_cards(220)
        result = duplicate_real_cards(real_cards, cfg, random.Random(42))
        ids = [c.card_id for c in result]
        self.assertEqual(len(ids), len(set(ids)))

    def test_no_gap_returns_original(self) -> None:
        cfg = SimulatorConfig()
        cfg.cube.distinct_cards = 100
        real_cards = self._make_real_cards(100)
        result = duplicate_real_cards(real_cards, cfg, random.Random(42))
        self.assertEqual(len(result), 100)


if __name__ == "__main__":
    unittest.main()
