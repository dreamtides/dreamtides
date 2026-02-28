"""Dream Atlas graph topology management.

Manages the Dream Atlas: a graph of dreamscape nodes that the player
navigates. Handles initialization, expansion on dreamscape completion,
site generation, and state transitions.
"""

import random
from typing import Optional

from models import Biome, DreamscapeNode, NodeState, Site, SiteType

BIOME_ENHANCED_SITE: dict[Biome, SiteType] = {
    Biome.VERDANT: SiteType.SHOP,
    Biome.CELESTIAL: SiteType.DREAMSIGN_OFFERING,
    Biome.TWILIGHT: SiteType.DREAM_JOURNEY,
    Biome.INFERNAL: SiteType.TEMPTING_OFFER,
    Biome.ASHEN: SiteType.PURGE,
    Biome.CRYSTALLINE: SiteType.ESSENCE,
    Biome.PRISMATIC: SiteType.TRANSFIGURATION,
    Biome.MIRRORED: SiteType.DUPLICATION,
    Biome.ARCANE: SiteType.DISCOVERY_DRAFT,
}

DREAMSCAPE_NAMES: list[str] = [
    "Twilight Grove",
    "Crystal Spire",
    "Moonlit Shore",
    "Whispering Depths",
    "Ember Sanctum",
    "Shattered Oasis",
    "Starfall Basin",
    "Obsidian Reach",
    "Prismatic Hollow",
    "Silent Canopy",
    "Driftwood Haven",
    "Ashen Plateau",
    "Luminous Grotto",
    "Veiled Summit",
    "Coral Labyrinth",
    "Phantom Glade",
    "Frozen Expanse",
    "Gilded Threshold",
    "Sunken Archive",
    "Crimson Atrium",
    "Sapphire Terrace",
    "Winding Abyss",
    "Verdant Spires",
    "Iron Solitude",
    "Opal Cascade",
    "Thornwood Passage",
    "Mirrored Sanctum",
    "Celestial Landing",
    "Echoing Cavern",
    "Dusk Meridian",
    "Amber Threshold",
    "Jade Precipice",
    "Silver Expanse",
    "Hollow Zenith",
    "Wandering Isles",
]

_SITE_POOL_BY_LEVEL: list[list[SiteType]] = [
    [SiteType.ESSENCE, SiteType.SHOP],
    [SiteType.DREAMSIGN_OFFERING, SiteType.DREAM_JOURNEY, SiteType.DISCOVERY_DRAFT],
    [SiteType.TEMPTING_OFFER, SiteType.SPECIALTY_SHOP, SiteType.DREAMSIGN_DRAFT],
    [SiteType.PURGE, SiteType.TRANSFIGURATION, SiteType.DUPLICATION],
    [SiteType.REWARD_SITE, SiteType.CLEANSE],
]


def _available_site_pool(completion_level: int) -> list[SiteType]:
    """Return the full list of site types available at a given completion level."""
    pool: list[SiteType] = []
    for level, types in enumerate(_SITE_POOL_BY_LEVEL):
        if level <= completion_level:
            pool.extend(types)
    return pool


def _pick_name(rng: random.Random, used_names: set[str]) -> str:
    """Pick a dreamscape name that hasn't been used yet."""
    available = [n for n in DREAMSCAPE_NAMES if n not in used_names]
    if not available:
        # Fallback: generate a numbered name if pool is exhausted
        i = len(used_names) + 1
        while f"Dreamscape {i}" in used_names:
            i += 1
        return f"Dreamscape {i}"
    return rng.choice(available)


def initialize_atlas(rng: random.Random) -> list[DreamscapeNode]:
    """Create the initial atlas with a Nexus node and 3 Available neighbors."""
    used_names: set[str] = {"The Nexus"}

    nexus = DreamscapeNode(
        node_id=0,
        name="The Nexus",
        biome=rng.choice(list(Biome)),
        sites=[],
        state=NodeState.COMPLETED,
        adjacent=[],
    )

    nodes: list[DreamscapeNode] = [nexus]

    for i in range(1, 4):
        name = _pick_name(rng, used_names)
        used_names.add(name)
        node = DreamscapeNode(
            node_id=i,
            name=name,
            biome=rng.choice(list(Biome)),
            sites=[],
            state=NodeState.AVAILABLE,
            adjacent=[0],
        )
        nodes.append(node)
        nexus.adjacent.append(i)

    return nodes


def complete_node(
    nodes: list[DreamscapeNode],
    node_id: int,
    rng: random.Random,
) -> None:
    """Mark a node as Completed and expand the atlas.

    Generates 2-4 new Unavailable nodes adjacent to the completed node.
    Then promotes any Unavailable node adjacent to a Completed node to
    Available.
    """
    target = get_node_by_id(nodes, node_id)
    if target is None:
        raise ValueError(f"No node with id {node_id} exists in the atlas")
    if target.state == NodeState.COMPLETED:
        return
    target.state = NodeState.COMPLETED

    used_names = {n.name for n in nodes}
    next_id = max(n.node_id for n in nodes) + 1
    new_count = rng.randint(2, 4)

    for i in range(new_count):
        name = _pick_name(rng, used_names)
        used_names.add(name)
        new_node = DreamscapeNode(
            node_id=next_id + i,
            name=name,
            biome=rng.choice(list(Biome)),
            sites=[],
            state=NodeState.UNAVAILABLE,
            adjacent=[node_id],
        )
        nodes.append(new_node)
        target.adjacent.append(new_node.node_id)

    # Promote unavailable nodes adjacent to any completed node
    completed_ids = {n.node_id for n in nodes if n.state == NodeState.COMPLETED}
    for node in nodes:
        if node.state == NodeState.UNAVAILABLE:
            if any(adj_id in completed_ids for adj_id in node.adjacent):
                node.state = NodeState.AVAILABLE


def generate_sites(
    node: DreamscapeNode,
    completion_level: int,
    rng: random.Random,
    is_first_dreamscape: bool = False,
) -> None:
    """Generate the site list for a dreamscape node.

    Populates node.sites based on completion level, biome enhancement,
    and randomization rules.
    """
    sites: list[Site] = []
    type_counts: dict[SiteType, int] = {}

    def _add_site(site_type: SiteType, enhanced: bool = False) -> None:
        sites.append(Site(site_type=site_type, is_enhanced=enhanced))
        type_counts[site_type] = type_counts.get(site_type, 0) + 1

    def _can_add(site_type: SiteType) -> bool:
        current = type_counts.get(site_type, 0)
        if site_type in (SiteType.DRAFT, SiteType.ESSENCE):
            return current < 2
        return current < 1

    # Battle: always 1
    _add_site(SiteType.BATTLE)

    # Dreamcaller Draft: only at level 0, first dreamscape
    if is_first_dreamscape and completion_level == 0:
        _add_site(SiteType.DREAMCALLER_DRAFT)

    # Draft sites: 2 at level 0-1, 1 at level 2-3, 0 at level 4+
    if completion_level <= 1:
        draft_count = 2
    elif completion_level <= 3:
        draft_count = 1
    else:
        draft_count = 0
    for _ in range(draft_count):
        _add_site(SiteType.DRAFT)

    # Other sites from the evolving pool
    pool = _available_site_pool(completion_level)

    # Determine how many "other" sites to add
    # Total target: 3-6, minus what we already have
    current_count = len(sites)
    min_other = max(0, 3 - current_count)
    max_other = max(0, 6 - current_count)
    if max_other <= 0:
        other_count = 0
    elif min_other >= max_other:
        other_count = min_other
    else:
        other_count = rng.randint(min_other, max_other)

    # Filter pool to what can still be added
    eligible = [st for st in pool if _can_add(st)]
    rng.shuffle(eligible)

    added = 0
    for st in eligible:
        if added >= other_count:
            break
        if _can_add(st):
            _add_site(st)
            added += 1

    # Apply biome enhancement
    enhanced_type = BIOME_ENHANCED_SITE.get(node.biome)
    if enhanced_type is not None:
        for site in sites:
            if site.site_type == enhanced_type and not site.is_enhanced:
                site.is_enhanced = True
                break

    node.sites = sites


def get_available_nodes(nodes: list[DreamscapeNode]) -> list[DreamscapeNode]:
    """Return all nodes in the Available state."""
    return [n for n in nodes if n.state == NodeState.AVAILABLE]


def get_node_by_id(
    nodes: list[DreamscapeNode], node_id: int
) -> Optional[DreamscapeNode]:
    """Return the node with the given ID, or None if not found."""
    for n in nodes:
        if n.node_id == node_id:
            return n
    return None
