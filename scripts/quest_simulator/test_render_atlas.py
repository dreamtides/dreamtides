"""Tests for the render_atlas module."""

import os
import sys
import unittest


class TestSiteTypeName(unittest.TestCase):
    """site_type_name maps SiteType enum values to human-readable strings."""

    def test_battle(self) -> None:
        from models import SiteType
        from render_atlas import site_type_name

        self.assertEqual(site_type_name(SiteType.BATTLE), "Battle")

    def test_draft(self) -> None:
        from models import SiteType
        from render_atlas import site_type_name

        self.assertEqual(site_type_name(SiteType.DRAFT), "Draft")

    def test_dreamcaller_draft(self) -> None:
        from models import SiteType
        from render_atlas import site_type_name

        self.assertEqual(site_type_name(SiteType.DREAMCALLER_DRAFT), "Dreamcaller Draft")

    def test_discovery_draft(self) -> None:
        from models import SiteType
        from render_atlas import site_type_name

        self.assertEqual(site_type_name(SiteType.DISCOVERY_DRAFT), "Discovery Draft")

    def test_specialty_shop(self) -> None:
        from models import SiteType
        from render_atlas import site_type_name

        self.assertEqual(site_type_name(SiteType.SPECIALTY_SHOP), "Specialty Shop")

    def test_dreamsign_offering(self) -> None:
        from models import SiteType
        from render_atlas import site_type_name

        self.assertEqual(site_type_name(SiteType.DREAMSIGN_OFFERING), "Dreamsign Offering")

    def test_dreamsign_draft(self) -> None:
        from models import SiteType
        from render_atlas import site_type_name

        self.assertEqual(site_type_name(SiteType.DREAMSIGN_DRAFT), "Dreamsign Draft")

    def test_dream_journey(self) -> None:
        from models import SiteType
        from render_atlas import site_type_name

        self.assertEqual(site_type_name(SiteType.DREAM_JOURNEY), "Dream Journey")

    def test_tempting_offer(self) -> None:
        from models import SiteType
        from render_atlas import site_type_name

        self.assertEqual(site_type_name(SiteType.TEMPTING_OFFER), "Tempting Offer")

    def test_reward_site(self) -> None:
        from models import SiteType
        from render_atlas import site_type_name

        self.assertEqual(site_type_name(SiteType.REWARD_SITE), "Reward Site")

    def test_all_site_types_have_names(self) -> None:
        from models import SiteType
        from render_atlas import site_type_name

        for st in SiteType:
            name = site_type_name(st)
            self.assertIsInstance(name, str)
            self.assertTrue(len(name) > 0, f"Empty name for {st}")


class TestDreamscapeSiteSummary(unittest.TestCase):
    """dreamscape_site_summary produces a comma-separated string of site names."""

    def test_basic_summary(self) -> None:
        from models import Site, SiteType
        from render_atlas import dreamscape_site_summary

        sites = [
            Site(site_type=SiteType.SHOP),
            Site(site_type=SiteType.DREAMSIGN_DRAFT),
            Site(site_type=SiteType.ESSENCE),
            Site(site_type=SiteType.BATTLE),
        ]
        result = dreamscape_site_summary(sites)
        self.assertIn("Shop", result)
        self.assertIn("Dreamsign Draft", result)
        self.assertIn("Essence", result)
        self.assertIn("Battle", result)

    def test_enhanced_site_marked(self) -> None:
        from models import Site, SiteType
        from render_atlas import dreamscape_site_summary

        sites = [
            Site(site_type=SiteType.SHOP, is_enhanced=True),
            Site(site_type=SiteType.ESSENCE),
        ]
        result = dreamscape_site_summary(sites)
        # Enhanced site should have a marker (asterisk)
        self.assertIn("Shop*", result)
        # Non-enhanced should not
        self.assertNotIn("Essence*", result)

    def test_empty_sites(self) -> None:
        from render_atlas import dreamscape_site_summary

        result = dreamscape_site_summary([])
        self.assertEqual(result, "")


class TestRenderAtlasHeader(unittest.TestCase):
    """render_atlas_header builds the atlas header box."""

    def test_contains_title(self) -> None:
        from render_atlas import render_atlas_header

        result = render_atlas_header(
            essence=450, completion=2, total_battles=7,
            deck_count=12, dreamsign_count=2,
        )
        self.assertIn("DREAM ATLAS", result)

    def test_contains_essence(self) -> None:
        from render_atlas import render_atlas_header

        result = render_atlas_header(
            essence=450, completion=2, total_battles=7,
            deck_count=12, dreamsign_count=2,
        )
        self.assertIn("Essence: 450", result)

    def test_contains_completion(self) -> None:
        from render_atlas import render_atlas_header

        result = render_atlas_header(
            essence=450, completion=2, total_battles=7,
            deck_count=12, dreamsign_count=2,
        )
        self.assertIn("2/7", result)

    def test_contains_deck_count(self) -> None:
        from render_atlas import render_atlas_header

        result = render_atlas_header(
            essence=100, completion=0, total_battles=7,
            deck_count=25, dreamsign_count=1,
        )
        self.assertIn("Deck: 25 cards", result)

    def test_contains_dreamsign_count(self) -> None:
        from render_atlas import render_atlas_header

        result = render_atlas_header(
            essence=100, completion=0, total_battles=7,
            deck_count=25, dreamsign_count=3,
        )
        self.assertIn("Dreamsigns: 3", result)

    def test_has_double_line_borders(self) -> None:
        from render_atlas import render_atlas_header

        result = render_atlas_header(
            essence=100, completion=0, total_battles=7,
            deck_count=10, dreamsign_count=0,
        )
        # Should contain double-line separator characters
        self.assertIn("\u2550", result)


class TestRenderAvailableDreamscapes(unittest.TestCase):
    """render_available_dreamscapes shows selectable dreamscape nodes."""

    def _make_nodes(self) -> list:  # list[DreamscapeNode]
        from models import Biome, DreamscapeNode, NodeState, Site, SiteType

        return [
            DreamscapeNode(
                node_id=1,
                name="Twilight Grove",
                biome=Biome.VERDANT,
                sites=[
                    Site(site_type=SiteType.SHOP, is_enhanced=True),
                    Site(site_type=SiteType.DREAMSIGN_DRAFT),
                    Site(site_type=SiteType.ESSENCE),
                    Site(site_type=SiteType.BATTLE),
                ],
                state=NodeState.AVAILABLE,
                adjacent=[0],
            ),
            DreamscapeNode(
                node_id=2,
                name="Crystal Spire",
                biome=Biome.ASHEN,
                sites=[
                    Site(site_type=SiteType.PURGE, is_enhanced=True),
                    Site(site_type=SiteType.TRANSFIGURATION),
                    Site(site_type=SiteType.DREAM_JOURNEY),
                    Site(site_type=SiteType.BATTLE),
                ],
                state=NodeState.AVAILABLE,
                adjacent=[0],
            ),
        ]

    def test_selected_node_has_marker(self) -> None:
        from render_atlas import render_available_dreamscapes

        nodes = self._make_nodes()
        result = render_available_dreamscapes(nodes, selected_index=0)
        lines = result.split("\n")
        # First dreamscape line should have >
        dreamscape_lines = [l for l in lines if "Twilight Grove" in l]
        self.assertTrue(any(">" in l for l in dreamscape_lines))

    def test_unselected_node_no_marker(self) -> None:
        from render_atlas import render_available_dreamscapes

        nodes = self._make_nodes()
        result = render_available_dreamscapes(nodes, selected_index=0)
        lines = result.split("\n")
        # Crystal Spire should not have >
        crystal_lines = [l for l in lines if "Crystal Spire" in l]
        self.assertTrue(len(crystal_lines) > 0)
        self.assertTrue(all(">" not in l for l in crystal_lines))

    def test_shows_site_summary(self) -> None:
        from render_atlas import render_available_dreamscapes

        nodes = self._make_nodes()
        result = render_available_dreamscapes(nodes, selected_index=0)
        # Site names appear in the summary (may be truncated if line is long)
        self.assertIn("Shop", result)
        self.assertIn("Dreamsign", result)

    def test_shows_biome(self) -> None:
        from render_atlas import render_available_dreamscapes

        nodes = self._make_nodes()
        result = render_available_dreamscapes(nodes, selected_index=0)
        self.assertIn("Verdant", result)

    def test_empty_list(self) -> None:
        from render_atlas import render_available_dreamscapes

        result = render_available_dreamscapes([], selected_index=0)
        self.assertIsInstance(result, str)


class TestRenderCompletedTrail(unittest.TestCase):
    """render_completed_trail shows the chain of completed dreamscapes."""

    def test_single_completed(self) -> None:
        from models import Biome, DreamscapeNode, NodeState

        from render_atlas import render_completed_trail

        nodes = [
            DreamscapeNode(
                node_id=0, name="The Nexus", biome=Biome.VERDANT,
                sites=[], state=NodeState.COMPLETED, adjacent=[1],
            ),
        ]
        result = render_completed_trail(nodes)
        self.assertIn("The Nexus", result)

    def test_multiple_completed(self) -> None:
        from models import Biome, DreamscapeNode, NodeState

        from render_atlas import render_completed_trail

        nodes = [
            DreamscapeNode(
                node_id=0, name="The Nexus", biome=Biome.VERDANT,
                sites=[], state=NodeState.COMPLETED, adjacent=[1],
            ),
            DreamscapeNode(
                node_id=1, name="Whispering Depths", biome=Biome.TWILIGHT,
                sites=[], state=NodeState.COMPLETED, adjacent=[0, 2],
            ),
            DreamscapeNode(
                node_id=2, name="Moonlit Shore", biome=Biome.CELESTIAL,
                sites=[], state=NodeState.COMPLETED, adjacent=[1],
            ),
        ]
        result = render_completed_trail(nodes)
        self.assertIn("The Nexus", result)
        self.assertIn("Whispering Depths", result)
        self.assertIn("Moonlit Shore", result)
        self.assertIn("->", result)

    def test_no_completed(self) -> None:
        from render_atlas import render_completed_trail

        result = render_completed_trail([])
        self.assertIsInstance(result, str)


class TestRenderDreamscapeSites(unittest.TestCase):
    """render_dreamscape_sites shows site list after entering a dreamscape."""

    def _make_sites(self) -> list:  # list[Site]
        from models import Site, SiteType

        return [
            Site(site_type=SiteType.DRAFT, is_visited=True),
            Site(site_type=SiteType.SHOP),
            Site(site_type=SiteType.ESSENCE),
            Site(site_type=SiteType.BATTLE),
        ]

    def test_visited_site_has_checkmark(self) -> None:
        from render_atlas import render_dreamscape_sites

        sites = self._make_sites()
        result = render_dreamscape_sites(sites, selected_index=1)
        # Draft was visited, should have checkmark
        draft_lines = [l for l in result.split("\n") if "Draft" in l and "Dreamsign" not in l]
        self.assertTrue(any("\u2713" in l for l in draft_lines))

    def test_selected_site_has_marker(self) -> None:
        from render_atlas import render_dreamscape_sites

        sites = self._make_sites()
        result = render_dreamscape_sites(sites, selected_index=1)
        lines = result.split("\n")
        shop_lines = [l for l in lines if "Shop" in l]
        self.assertTrue(any(">" in l for l in shop_lines))

    def test_battle_locked_when_others_unvisited(self) -> None:
        from render_atlas import render_dreamscape_sites

        sites = self._make_sites()
        result = render_dreamscape_sites(sites, selected_index=0)
        battle_lines = [l for l in result.split("\n") if "Battle" in l]
        self.assertTrue(any("[locked]" in l for l in battle_lines))

    def test_battle_unlocked_when_all_others_visited(self) -> None:
        from models import Site, SiteType
        from render_atlas import render_dreamscape_sites

        sites = [
            Site(site_type=SiteType.DRAFT, is_visited=True),
            Site(site_type=SiteType.SHOP, is_visited=True),
            Site(site_type=SiteType.ESSENCE, is_visited=True),
            Site(site_type=SiteType.BATTLE),
        ]
        result = render_dreamscape_sites(sites, selected_index=3)
        battle_lines = [l for l in result.split("\n") if "Battle" in l]
        self.assertTrue(all("[locked]" not in l for l in battle_lines))

    def test_enhanced_site_shown(self) -> None:
        from models import Site, SiteType
        from render_atlas import render_dreamscape_sites

        sites = [
            Site(site_type=SiteType.SHOP, is_enhanced=True),
            Site(site_type=SiteType.BATTLE),
        ]
        result = render_dreamscape_sites(sites, selected_index=0)
        shop_lines = [l for l in result.split("\n") if "Shop" in l]
        # Enhanced marker
        self.assertTrue(any("*" in l for l in shop_lines))


class TestRenderFullAtlas(unittest.TestCase):
    """render_full_atlas combines header, available, completed, and prompt."""

    def test_returns_string(self) -> None:
        from models import Biome, DreamscapeNode, NodeState, Site, SiteType

        from render_atlas import render_full_atlas

        completed = [
            DreamscapeNode(
                node_id=0, name="The Nexus", biome=Biome.VERDANT,
                sites=[], state=NodeState.COMPLETED, adjacent=[1],
            ),
        ]
        available = [
            DreamscapeNode(
                node_id=1, name="Twilight Grove", biome=Biome.VERDANT,
                sites=[
                    Site(site_type=SiteType.SHOP),
                    Site(site_type=SiteType.BATTLE),
                ],
                state=NodeState.AVAILABLE, adjacent=[0],
            ),
        ]
        result = render_full_atlas(
            available_nodes=available,
            all_nodes=completed + available,
            selected_index=0,
            essence=250,
            completion=1,
            total_battles=7,
            deck_count=10,
            dreamsign_count=0,
        )
        self.assertIn("DREAM ATLAS", result)
        self.assertIn("Twilight Grove", result)
        self.assertIn("The Nexus", result)

    def test_contains_prompt(self) -> None:
        from models import Biome, DreamscapeNode, NodeState, Site, SiteType

        from render_atlas import render_full_atlas

        available = [
            DreamscapeNode(
                node_id=1, name="Crystal Spire", biome=Biome.ASHEN,
                sites=[Site(site_type=SiteType.BATTLE)],
                state=NodeState.AVAILABLE, adjacent=[0],
            ),
        ]
        result = render_full_atlas(
            available_nodes=available,
            all_nodes=available,
            selected_index=0,
            essence=100,
            completion=0,
            total_battles=7,
            deck_count=5,
            dreamsign_count=0,
        )
        # Should contain navigation prompt
        self.assertIn("Enter", result)


class TestBiomeEnhancementText(unittest.TestCase):
    """biome_enhancement_text maps each biome to its enhancement effect."""

    def test_verdant_free_reroll(self) -> None:
        from models import Biome
        from render_atlas import biome_enhancement_text

        self.assertEqual(biome_enhancement_text(Biome.VERDANT), "free reroll")

    def test_celestial_pick_1_of_3(self) -> None:
        from models import Biome
        from render_atlas import biome_enhancement_text

        self.assertEqual(biome_enhancement_text(Biome.CELESTIAL), "pick 1 of 3")

    def test_twilight_3rd_option(self) -> None:
        from models import Biome
        from render_atlas import biome_enhancement_text

        self.assertEqual(biome_enhancement_text(Biome.TWILIGHT), "3rd option")

    def test_infernal_3_pairs(self) -> None:
        from models import Biome
        from render_atlas import biome_enhancement_text

        self.assertEqual(biome_enhancement_text(Biome.INFERNAL), "3 pairs")

    def test_ashen_purge_up_to_6(self) -> None:
        from models import Biome
        from render_atlas import biome_enhancement_text

        self.assertEqual(biome_enhancement_text(Biome.ASHEN), "purge up to 6")

    def test_crystalline_doubled(self) -> None:
        from models import Biome
        from render_atlas import biome_enhancement_text

        self.assertEqual(biome_enhancement_text(Biome.CRYSTALLINE), "doubled")

    def test_prismatic_select_target(self) -> None:
        from models import Biome
        from render_atlas import biome_enhancement_text

        self.assertEqual(biome_enhancement_text(Biome.PRISMATIC), "select target")

    def test_mirrored_choose_any(self) -> None:
        from models import Biome
        from render_atlas import biome_enhancement_text

        self.assertEqual(biome_enhancement_text(Biome.MIRRORED), "choose any")

    def test_arcane_pick_any_number(self) -> None:
        from models import Biome
        from render_atlas import biome_enhancement_text

        self.assertEqual(biome_enhancement_text(Biome.ARCANE), "pick any number")

    def test_all_biomes_have_text(self) -> None:
        from models import Biome
        from render_atlas import biome_enhancement_text

        for biome in Biome:
            text = biome_enhancement_text(biome)
            self.assertIsInstance(text, str)
            self.assertTrue(len(text) > 0, f"Empty text for {biome}")


class TestEnhancedSiteDescription(unittest.TestCase):
    """Enhanced sites show the biome's enhancement description."""

    def test_summary_with_biome_shows_enhancement_text(self) -> None:
        from models import Biome, Site, SiteType
        from render_atlas import dreamscape_site_summary

        sites = [
            Site(site_type=SiteType.SHOP, is_enhanced=True),
            Site(site_type=SiteType.ESSENCE),
        ]
        result = dreamscape_site_summary(sites, biome=Biome.VERDANT)
        self.assertIn("*free reroll*", result)

    def test_summary_without_biome_uses_star_only(self) -> None:
        from models import Site, SiteType
        from render_atlas import dreamscape_site_summary

        sites = [
            Site(site_type=SiteType.SHOP, is_enhanced=True),
            Site(site_type=SiteType.ESSENCE),
        ]
        result = dreamscape_site_summary(sites)
        self.assertIn("Shop*", result)

    def test_available_dreamscapes_show_enhancement_text(self) -> None:
        from models import Biome, DreamscapeNode, NodeState, Site, SiteType
        from render_atlas import render_available_dreamscapes

        nodes = [
            DreamscapeNode(
                node_id=1,
                name="Verdant Hollow",
                biome=Biome.VERDANT,
                sites=[
                    Site(site_type=SiteType.SHOP, is_enhanced=True),
                    Site(site_type=SiteType.BATTLE),
                ],
                state=NodeState.AVAILABLE,
                adjacent=[0],
            ),
        ]
        result = render_available_dreamscapes(nodes, selected_index=0)
        self.assertIn("*free reroll*", result)

    def test_dreamscape_sites_show_enhancement_text(self) -> None:
        from models import Biome, Site, SiteType
        from render_atlas import render_dreamscape_sites

        sites = [
            Site(site_type=SiteType.SHOP, is_enhanced=True),
            Site(site_type=SiteType.BATTLE),
        ]
        result = render_dreamscape_sites(
            sites, selected_index=0, biome=Biome.VERDANT
        )
        self.assertIn("*free reroll*", result)


class TestAvailableDreamscapesWidth(unittest.TestCase):
    """Available dreamscape lines fit within 70 columns."""

    def test_long_summary_truncated(self) -> None:
        from models import Biome, DreamscapeNode, NodeState, Site, SiteType
        from render import visible_len
        from render_atlas import render_available_dreamscapes

        nodes = [
            DreamscapeNode(
                node_id=1,
                name="Moonlit Shore",
                biome=Biome.CELESTIAL,
                sites=[
                    Site(site_type=SiteType.DREAMSIGN_OFFERING, is_enhanced=True),
                    Site(site_type=SiteType.DRAFT),
                    Site(site_type=SiteType.ESSENCE),
                    Site(site_type=SiteType.BATTLE),
                ],
                state=NodeState.AVAILABLE,
                adjacent=[0],
            ),
        ]
        result = render_available_dreamscapes(nodes, selected_index=0)
        for line in result.split("\n"):
            self.assertLessEqual(
                visible_len(line), 70,
                f"Line too wide: {line!r}",
            )


class TestCompletedTrailWrapping(unittest.TestCase):
    """Completed trail wraps to multiple lines at 70 columns."""

    def test_short_trail_single_line(self) -> None:
        from models import Biome, DreamscapeNode, NodeState
        from render_atlas import render_completed_trail

        nodes = [
            DreamscapeNode(
                node_id=0, name="The Nexus", biome=Biome.VERDANT,
                sites=[], state=NodeState.COMPLETED, adjacent=[1],
            ),
            DreamscapeNode(
                node_id=1, name="Shore", biome=Biome.CELESTIAL,
                sites=[], state=NodeState.COMPLETED, adjacent=[0],
            ),
        ]
        result = render_completed_trail(nodes)
        # Trail content should be on a single indented line
        trail_lines = [
            l for l in result.split("\n")
            if "[" in l and "]" in l and "Completed" not in l
        ]
        self.assertEqual(len(trail_lines), 1)

    def test_long_trail_wraps(self) -> None:
        from models import Biome, DreamscapeNode, NodeState
        from render_atlas import render_completed_trail

        nodes = [
            DreamscapeNode(
                node_id=i,
                name=f"Dreamscape With Long Name {i}",
                biome=Biome.VERDANT,
                sites=[],
                state=NodeState.COMPLETED,
                adjacent=[],
            )
            for i in range(8)
        ]
        result = render_completed_trail(nodes)
        trail_lines = [
            l for l in result.split("\n")
            if "[" in l and "]" in l and "Completed" not in l
        ]
        self.assertGreater(len(trail_lines), 1)

    def test_wrapped_lines_within_70_columns(self) -> None:
        from models import Biome, DreamscapeNode, NodeState
        from render_atlas import render_completed_trail

        nodes = [
            DreamscapeNode(
                node_id=i,
                name=f"Dreamscape With Long Name {i}",
                biome=Biome.VERDANT,
                sites=[],
                state=NodeState.COMPLETED,
                adjacent=[],
            )
            for i in range(8)
        ]
        result = render_completed_trail(nodes)
        for line in result.split("\n"):
            self.assertLessEqual(len(line), 70, f"Line too long: {line!r}")


class TestNodeCount(unittest.TestCase):
    """Atlas header shows the total dreamscape count."""

    def test_header_shows_node_count(self) -> None:
        from render_atlas import render_atlas_header

        result = render_atlas_header(
            essence=100, completion=2, total_battles=7,
            deck_count=10, dreamsign_count=0,
            total_nodes=12,
        )
        self.assertIn("12 dreamscapes", result)

    def test_full_atlas_shows_node_count(self) -> None:
        from models import Biome, DreamscapeNode, NodeState, Site, SiteType
        from render_atlas import render_full_atlas

        nodes = [
            DreamscapeNode(
                node_id=i,
                name=f"Node {i}",
                biome=Biome.VERDANT,
                sites=[Site(site_type=SiteType.BATTLE)],
                state=NodeState.AVAILABLE if i > 0 else NodeState.COMPLETED,
                adjacent=[],
            )
            for i in range(5)
        ]
        result = render_full_atlas(
            available_nodes=[n for n in nodes if n.state == NodeState.AVAILABLE],
            all_nodes=nodes,
            selected_index=0,
            essence=100,
            completion=1,
            total_battles=7,
            deck_count=10,
            dreamsign_count=0,
        )
        self.assertIn("5 dreamscapes", result)


class TestBiomeColors(unittest.TestCase):
    """Biome colors match the specification."""

    def test_biome_color_mapping_complete(self) -> None:
        from models import Biome
        from render_atlas import _BIOME_MARKERS

        for biome in Biome:
            self.assertIn(biome, _BIOME_MARKERS)


class TestImportClean(unittest.TestCase):
    """Verify module imports cleanly."""

    def test_import_succeeds(self) -> None:
        import subprocess

        result = subprocess.run(
            [
                sys.executable,
                "-c",
                (
                    "import sys; sys.path.insert(0, 'scripts/quest_simulator');"
                    "from render_atlas import *;"
                    "print('OK')"
                ),
            ],
            capture_output=True,
            text=True,
            env={**os.environ, "NO_COLOR": "1"},
            cwd=os.path.join(os.path.dirname(__file__), "..", ".."),
        )
        self.assertEqual(result.returncode, 0, f"stderr: {result.stderr}")
        self.assertIn("OK", result.stdout)


class TestAtlasPreviewExcludesDraftAndBattle(unittest.TestCase):
    """Atlas preview summary excludes draft and battle sites."""

    def test_preview_excludes_battle(self) -> None:
        from models import Biome, DreamscapeNode, NodeState, Site, SiteType
        from render_atlas import render_available_dreamscapes

        nodes = [
            DreamscapeNode(
                node_id=1,
                name="Test Node",
                biome=Biome.VERDANT,
                sites=[
                    Site(site_type=SiteType.SHOP),
                    Site(site_type=SiteType.ESSENCE),
                    Site(site_type=SiteType.BATTLE),
                ],
                state=NodeState.AVAILABLE,
                adjacent=[0],
            ),
        ]
        result = render_available_dreamscapes(nodes, selected_index=0)
        self.assertNotIn("Battle", result)

    def test_preview_excludes_draft(self) -> None:
        from models import Biome, DreamscapeNode, NodeState, Site, SiteType
        from render_atlas import render_available_dreamscapes

        nodes = [
            DreamscapeNode(
                node_id=1,
                name="Test Node",
                biome=Biome.VERDANT,
                sites=[
                    Site(site_type=SiteType.SHOP),
                    Site(site_type=SiteType.DRAFT),
                    Site(site_type=SiteType.BATTLE),
                ],
                state=NodeState.AVAILABLE,
                adjacent=[0],
            ),
        ]
        result = render_available_dreamscapes(nodes, selected_index=0)
        self.assertNotIn("Draft", result)

    def test_preview_excludes_dreamcaller_draft(self) -> None:
        from models import Biome, DreamscapeNode, NodeState, Site, SiteType
        from render_atlas import render_available_dreamscapes

        nodes = [
            DreamscapeNode(
                node_id=1,
                name="Test Node",
                biome=Biome.VERDANT,
                sites=[
                    Site(site_type=SiteType.SHOP),
                    Site(site_type=SiteType.DREAMCALLER_DRAFT),
                    Site(site_type=SiteType.BATTLE),
                ],
                state=NodeState.AVAILABLE,
                adjacent=[0],
            ),
        ]
        result = render_available_dreamscapes(nodes, selected_index=0)
        self.assertNotIn("Dreamcaller Draft", result)

    def test_preview_includes_non_draft_non_battle_sites(self) -> None:
        from models import Biome, DreamscapeNode, NodeState, Site, SiteType
        from render_atlas import render_available_dreamscapes

        nodes = [
            DreamscapeNode(
                node_id=1,
                name="Test Node",
                biome=Biome.VERDANT,
                sites=[
                    Site(site_type=SiteType.SHOP),
                    Site(site_type=SiteType.ESSENCE),
                    Site(site_type=SiteType.DRAFT),
                    Site(site_type=SiteType.BATTLE),
                ],
                state=NodeState.AVAILABLE,
                adjacent=[0],
            ),
        ]
        result = render_available_dreamscapes(nodes, selected_index=0)
        self.assertIn("Shop", result)
        self.assertIn("Essence", result)


if __name__ == "__main__":
    unittest.main()
