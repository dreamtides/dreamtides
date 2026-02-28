"""Tests for the Dream Atlas graph topology module."""

import random

from models import Biome, DreamscapeNode, NodeState, Site, SiteType


class TestInitializeAtlas:
    def test_initial_atlas_has_four_nodes(self) -> None:
        from atlas import initialize_atlas

        rng = random.Random(42)
        nodes = initialize_atlas(rng)
        assert len(nodes) == 4

    def test_nexus_node_is_completed(self) -> None:
        from atlas import initialize_atlas

        rng = random.Random(42)
        nodes = initialize_atlas(rng)
        nexus = nodes[0]
        assert nexus.name == "The Nexus"
        assert nexus.state == NodeState.COMPLETED

    def test_nexus_node_id_is_zero(self) -> None:
        from atlas import initialize_atlas

        rng = random.Random(42)
        nodes = initialize_atlas(rng)
        assert nodes[0].node_id == 0

    def test_three_available_nodes(self) -> None:
        from atlas import initialize_atlas

        rng = random.Random(42)
        nodes = initialize_atlas(rng)
        available = [n for n in nodes if n.state == NodeState.AVAILABLE]
        assert len(available) == 3

    def test_nexus_connected_to_all_available(self) -> None:
        from atlas import initialize_atlas

        rng = random.Random(42)
        nodes = initialize_atlas(rng)
        nexus = nodes[0]
        available_ids = [n.node_id for n in nodes if n.state == NodeState.AVAILABLE]
        for aid in available_ids:
            assert aid in nexus.adjacent

    def test_available_nodes_connected_to_nexus(self) -> None:
        from atlas import initialize_atlas

        rng = random.Random(42)
        nodes = initialize_atlas(rng)
        for node in nodes[1:]:
            assert 0 in node.adjacent

    def test_available_nodes_have_biomes(self) -> None:
        from atlas import initialize_atlas

        rng = random.Random(42)
        nodes = initialize_atlas(rng)
        for node in nodes[1:]:
            assert isinstance(node.biome, Biome)

    def test_available_nodes_have_unique_names(self) -> None:
        from atlas import initialize_atlas

        rng = random.Random(42)
        nodes = initialize_atlas(rng)
        names = [n.name for n in nodes[1:]]
        assert len(names) == len(set(names))

    def test_deterministic_with_same_seed(self) -> None:
        from atlas import initialize_atlas

        nodes1 = initialize_atlas(random.Random(42))
        nodes2 = initialize_atlas(random.Random(42))
        for n1, n2 in zip(nodes1, nodes2):
            assert n1.name == n2.name
            assert n1.biome == n2.biome

    def test_different_seeds_produce_different_atlases(self) -> None:
        from atlas import initialize_atlas

        nodes1 = initialize_atlas(random.Random(42))
        nodes2 = initialize_atlas(random.Random(99))
        # At least one name or biome should differ
        names1 = [n.name for n in nodes1[1:]]
        names2 = [n.name for n in nodes2[1:]]
        assert names1 != names2 or any(
            n1.biome != n2.biome for n1, n2 in zip(nodes1[1:], nodes2[1:])
        )

    def test_available_nodes_have_empty_sites(self) -> None:
        from atlas import initialize_atlas

        rng = random.Random(42)
        nodes = initialize_atlas(rng)
        for node in nodes[1:]:
            assert node.sites == []


class TestCompleteNode:
    def test_completed_node_becomes_completed(self) -> None:
        from atlas import complete_node, initialize_atlas

        rng = random.Random(42)
        nodes = initialize_atlas(rng)
        target = nodes[1]
        complete_node(nodes, target.node_id, rng)
        assert target.state == NodeState.COMPLETED

    def test_completion_generates_new_nodes(self) -> None:
        from atlas import complete_node, initialize_atlas

        rng = random.Random(42)
        nodes = initialize_atlas(rng)
        initial_count = len(nodes)
        target = nodes[1]
        complete_node(nodes, target.node_id, rng)
        new_count = len(nodes)
        assert 2 <= new_count - initial_count <= 4

    def test_new_nodes_are_unavailable_or_available(self) -> None:
        from atlas import complete_node, initialize_atlas

        rng = random.Random(42)
        nodes = initialize_atlas(rng)
        initial_count = len(nodes)
        target = nodes[1]
        complete_node(nodes, target.node_id, rng)
        for node in nodes[initial_count:]:
            assert node.state in (NodeState.UNAVAILABLE, NodeState.AVAILABLE)

    def test_new_nodes_adjacent_to_completed_become_available(self) -> None:
        from atlas import complete_node, initialize_atlas

        rng = random.Random(42)
        nodes = initialize_atlas(rng)
        target = nodes[1]
        complete_node(nodes, target.node_id, rng)
        # Any unavailable node adjacent to a completed node should be available
        completed_ids = {n.node_id for n in nodes if n.state == NodeState.COMPLETED}
        for node in nodes:
            if node.state == NodeState.UNAVAILABLE:
                for adj_id in node.adjacent:
                    assert adj_id not in completed_ids or node.state != NodeState.UNAVAILABLE

    def test_new_nodes_are_adjacent_to_completed_node(self) -> None:
        from atlas import complete_node, initialize_atlas

        rng = random.Random(42)
        nodes = initialize_atlas(rng)
        initial_count = len(nodes)
        target = nodes[1]
        complete_node(nodes, target.node_id, rng)
        for node in nodes[initial_count:]:
            assert target.node_id in node.adjacent

    def test_completed_node_is_adjacent_to_new_nodes(self) -> None:
        from atlas import complete_node, initialize_atlas

        rng = random.Random(42)
        nodes = initialize_atlas(rng)
        initial_count = len(nodes)
        target = nodes[1]
        complete_node(nodes, target.node_id, rng)
        new_ids = [n.node_id for n in nodes[initial_count:]]
        for nid in new_ids:
            assert nid in target.adjacent

    def test_new_nodes_have_unique_names(self) -> None:
        from atlas import complete_node, initialize_atlas

        rng = random.Random(42)
        nodes = initialize_atlas(rng)
        complete_node(nodes, nodes[1].node_id, rng)
        names = [n.name for n in nodes]
        assert len(names) == len(set(names))

    def test_unavailable_adjacent_to_completed_becomes_available(self) -> None:
        from atlas import complete_node, initialize_atlas

        # Complete first available node, generating unavailable neighbors
        rng = random.Random(42)
        nodes = initialize_atlas(rng)
        first_id = nodes[1].node_id
        complete_node(nodes, first_id, rng)

        # Now check: any node adjacent to both a completed node should be available
        completed_ids = {n.node_id for n in nodes if n.state == NodeState.COMPLETED}
        for node in nodes:
            if node.state == NodeState.UNAVAILABLE:
                # None of its adjacents should be completed
                adjacent_completed = any(
                    aid in completed_ids for aid in node.adjacent
                )
                assert not adjacent_completed


class TestGetAvailableNodes:
    def test_get_available_returns_only_available(self) -> None:
        from atlas import get_available_nodes, initialize_atlas

        rng = random.Random(42)
        nodes = initialize_atlas(rng)
        available = get_available_nodes(nodes)
        assert len(available) == 3
        for node in available:
            assert node.state == NodeState.AVAILABLE


class TestGetNodeById:
    def test_get_existing_node(self) -> None:
        from atlas import get_node_by_id, initialize_atlas

        rng = random.Random(42)
        nodes = initialize_atlas(rng)
        node = get_node_by_id(nodes, 0)
        assert node is not None
        assert node.node_id == 0

    def test_get_nonexistent_node_returns_none(self) -> None:
        from atlas import get_node_by_id, initialize_atlas

        rng = random.Random(42)
        nodes = initialize_atlas(rng)
        node = get_node_by_id(nodes, 999)
        assert node is None


class TestGenerateSites:
    def test_level_0_sites_include_battle(self) -> None:
        from atlas import generate_sites

        rng = random.Random(42)
        node = DreamscapeNode(
            node_id=1, name="Test", biome=Biome.VERDANT,
            sites=[], state=NodeState.AVAILABLE, adjacent=[0],
        )
        generate_sites(node, completion_level=0, rng=rng, is_first_dreamscape=False)
        battle_sites = [s for s in node.sites if s.site_type == SiteType.BATTLE]
        assert len(battle_sites) == 1

    def test_level_0_first_dreamscape_has_dreamcaller_draft(self) -> None:
        from atlas import generate_sites

        rng = random.Random(42)
        node = DreamscapeNode(
            node_id=1, name="Test", biome=Biome.VERDANT,
            sites=[], state=NodeState.AVAILABLE, adjacent=[0],
        )
        generate_sites(node, completion_level=0, rng=rng, is_first_dreamscape=True)
        dc_sites = [s for s in node.sites if s.site_type == SiteType.DREAMCALLER_DRAFT]
        assert len(dc_sites) == 1

    def test_level_0_non_first_no_dreamcaller_draft(self) -> None:
        from atlas import generate_sites

        rng = random.Random(42)
        node = DreamscapeNode(
            node_id=1, name="Test", biome=Biome.VERDANT,
            sites=[], state=NodeState.AVAILABLE, adjacent=[0],
        )
        generate_sites(node, completion_level=0, rng=rng, is_first_dreamscape=False)
        dc_sites = [s for s in node.sites if s.site_type == SiteType.DREAMCALLER_DRAFT]
        assert len(dc_sites) == 0

    def test_level_0_has_two_draft_sites(self) -> None:
        from atlas import generate_sites

        rng = random.Random(42)
        node = DreamscapeNode(
            node_id=1, name="Test", biome=Biome.VERDANT,
            sites=[], state=NodeState.AVAILABLE, adjacent=[0],
        )
        generate_sites(node, completion_level=0, rng=rng, is_first_dreamscape=False)
        draft_sites = [s for s in node.sites if s.site_type == SiteType.DRAFT]
        assert len(draft_sites) == 2

    def test_level_2_has_one_draft_site(self) -> None:
        from atlas import generate_sites

        rng = random.Random(42)
        node = DreamscapeNode(
            node_id=1, name="Test", biome=Biome.VERDANT,
            sites=[], state=NodeState.AVAILABLE, adjacent=[0],
        )
        generate_sites(node, completion_level=2, rng=rng, is_first_dreamscape=False)
        draft_sites = [s for s in node.sites if s.site_type == SiteType.DRAFT]
        assert len(draft_sites) == 1

    def test_level_4_has_no_draft_sites(self) -> None:
        from atlas import generate_sites

        rng = random.Random(42)
        node = DreamscapeNode(
            node_id=1, name="Test", biome=Biome.VERDANT,
            sites=[], state=NodeState.AVAILABLE, adjacent=[0],
        )
        generate_sites(node, completion_level=4, rng=rng, is_first_dreamscape=False)
        draft_sites = [s for s in node.sites if s.site_type == SiteType.DRAFT]
        assert len(draft_sites) == 0

    def test_total_sites_in_valid_range(self) -> None:
        from atlas import generate_sites

        for seed in range(50):
            rng = random.Random(seed)
            node = DreamscapeNode(
                node_id=1, name="Test", biome=Biome.VERDANT,
                sites=[], state=NodeState.AVAILABLE, adjacent=[0],
            )
            generate_sites(node, completion_level=1, rng=rng, is_first_dreamscape=False)
            assert 3 <= len(node.sites) <= 6, (
                f"seed {seed}: {len(node.sites)} sites"
            )

    def test_max_one_of_each_site_type_except_draft_and_essence(self) -> None:
        from atlas import generate_sites

        for seed in range(50):
            rng = random.Random(seed)
            node = DreamscapeNode(
                node_id=1, name="Test", biome=Biome.ARCANE,
                sites=[], state=NodeState.AVAILABLE, adjacent=[0],
            )
            generate_sites(node, completion_level=3, rng=rng, is_first_dreamscape=False)
            type_counts: dict[SiteType, int] = {}
            for s in node.sites:
                type_counts[s.site_type] = type_counts.get(s.site_type, 0) + 1
            for st, count in type_counts.items():
                if st == SiteType.DRAFT:
                    assert count <= 2, f"seed {seed}: {count} Draft sites"
                elif st == SiteType.ESSENCE:
                    assert count <= 2, f"seed {seed}: {count} Essence sites"
                else:
                    assert count <= 1, f"seed {seed}: {count} {st} sites"

    def test_biome_enhancement_applies(self) -> None:
        from atlas import generate_sites

        # Verdant biome should enhance a Shop site if present
        enhanced_found = False
        for seed in range(100):
            rng = random.Random(seed)
            node = DreamscapeNode(
                node_id=1, name="Test", biome=Biome.VERDANT,
                sites=[], state=NodeState.AVAILABLE, adjacent=[0],
            )
            generate_sites(node, completion_level=0, rng=rng, is_first_dreamscape=False)
            for s in node.sites:
                if s.site_type == SiteType.SHOP and s.is_enhanced:
                    enhanced_found = True
                    break
            if enhanced_found:
                break
        assert enhanced_found, "Never found an enhanced shop for Verdant biome"

    def test_at_most_one_enhanced_site(self) -> None:
        from atlas import generate_sites

        for seed in range(50):
            rng = random.Random(seed)
            node = DreamscapeNode(
                node_id=1, name="Test", biome=Biome.CRYSTALLINE,
                sites=[], state=NodeState.AVAILABLE, adjacent=[0],
            )
            generate_sites(node, completion_level=2, rng=rng, is_first_dreamscape=False)
            enhanced_count = sum(1 for s in node.sites if s.is_enhanced)
            assert enhanced_count <= 1, f"seed {seed}: {enhanced_count} enhanced"

    def test_level_0_site_pool_only_has_essence_and_shop(self) -> None:
        from atlas import generate_sites

        # At level 0, "other" sites can only be Essence or Shop
        allowed_other = {
            SiteType.BATTLE, SiteType.DRAFT, SiteType.DREAMCALLER_DRAFT,
            SiteType.ESSENCE, SiteType.SHOP,
        }
        for seed in range(50):
            rng = random.Random(seed)
            node = DreamscapeNode(
                node_id=1, name="Test", biome=Biome.VERDANT,
                sites=[], state=NodeState.AVAILABLE, adjacent=[0],
            )
            generate_sites(node, completion_level=0, rng=rng, is_first_dreamscape=False)
            for s in node.sites:
                assert s.site_type in allowed_other, (
                    f"seed {seed}: unexpected {s.site_type} at level 0"
                )


class TestBiomeEnhancementMapping:
    def test_biome_to_site_mapping_is_dict(self) -> None:
        from atlas import BIOME_ENHANCED_SITE

        assert isinstance(BIOME_ENHANCED_SITE, dict)
        assert len(BIOME_ENHANCED_SITE) == 9

    def test_verdant_enhances_shop(self) -> None:
        from atlas import BIOME_ENHANCED_SITE

        assert BIOME_ENHANCED_SITE[Biome.VERDANT] == SiteType.SHOP

    def test_celestial_enhances_dreamsign_offering(self) -> None:
        from atlas import BIOME_ENHANCED_SITE

        assert BIOME_ENHANCED_SITE[Biome.CELESTIAL] == SiteType.DREAMSIGN_OFFERING

    def test_twilight_enhances_dream_journey(self) -> None:
        from atlas import BIOME_ENHANCED_SITE

        assert BIOME_ENHANCED_SITE[Biome.TWILIGHT] == SiteType.DREAM_JOURNEY

    def test_infernal_enhances_tempting_offer(self) -> None:
        from atlas import BIOME_ENHANCED_SITE

        assert BIOME_ENHANCED_SITE[Biome.INFERNAL] == SiteType.TEMPTING_OFFER

    def test_ashen_enhances_purge(self) -> None:
        from atlas import BIOME_ENHANCED_SITE

        assert BIOME_ENHANCED_SITE[Biome.ASHEN] == SiteType.PURGE

    def test_crystalline_enhances_essence(self) -> None:
        from atlas import BIOME_ENHANCED_SITE

        assert BIOME_ENHANCED_SITE[Biome.CRYSTALLINE] == SiteType.ESSENCE

    def test_prismatic_enhances_transfiguration(self) -> None:
        from atlas import BIOME_ENHANCED_SITE

        assert BIOME_ENHANCED_SITE[Biome.PRISMATIC] == SiteType.TRANSFIGURATION

    def test_mirrored_enhances_duplication(self) -> None:
        from atlas import BIOME_ENHANCED_SITE

        assert BIOME_ENHANCED_SITE[Biome.MIRRORED] == SiteType.DUPLICATION

    def test_arcane_enhances_discovery_draft(self) -> None:
        from atlas import BIOME_ENHANCED_SITE

        assert BIOME_ENHANCED_SITE[Biome.ARCANE] == SiteType.DISCOVERY_DRAFT
