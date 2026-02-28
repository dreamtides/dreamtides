"""Dream Atlas terminal display rendering.

Provides functions for rendering the Dream Atlas navigation hub,
dreamscape site lists, site summaries, and player status display.
Imports shared constants and utilities from render.py.
"""

from models import Biome, DreamscapeNode, NodeState, Site, SiteType
from render import (
    BOLD,
    CONTENT_WIDTH,
    DIM,
    RESET,
    draw_double_separator,
    draw_separator,
    pad_right,
    visible_len,
)

_SITE_TYPE_NAMES: dict[SiteType, str] = {
    SiteType.BATTLE: "Battle",
    SiteType.DRAFT: "Draft",
    SiteType.DREAMCALLER_DRAFT: "Dreamcaller Draft",
    SiteType.DISCOVERY_DRAFT: "Discovery Draft",
    SiteType.SHOP: "Shop",
    SiteType.SPECIALTY_SHOP: "Specialty Shop",
    SiteType.DREAMSIGN_OFFERING: "Dreamsign Offering",
    SiteType.DREAMSIGN_DRAFT: "Dreamsign Draft",
    SiteType.DREAM_JOURNEY: "Dream Journey",
    SiteType.TEMPTING_OFFER: "Tempting Offer",
    SiteType.PURGE: "Purge",
    SiteType.ESSENCE: "Essence",
    SiteType.TRANSFIGURATION: "Transfiguration",
    SiteType.DUPLICATION: "Duplication",
    SiteType.REWARD_SITE: "Reward Site",
    SiteType.CLEANSE: "Cleanse",
}

_BIOME_MARKERS: dict[Biome, str] = {
    Biome.VERDANT: "\033[92m",
    Biome.CELESTIAL: "\033[97m",
    Biome.TWILIGHT: "\033[95m",
    Biome.INFERNAL: "\033[91m",
    Biome.ASHEN: "\033[90m",
    Biome.CRYSTALLINE: "\033[96m",
    Biome.PRISMATIC: "\033[93m",
    Biome.MIRRORED: "\033[94m",
    Biome.ARCANE: "\033[35m",
}


def site_type_name(site_type: SiteType) -> str:
    """Map a SiteType enum value to a human-readable display string."""
    return _SITE_TYPE_NAMES.get(site_type, site_type.value)


def dreamscape_site_summary(sites: list[Site]) -> str:
    """Produce a comma-separated summary of site names.

    Enhanced sites are marked with an asterisk.
    """
    parts: list[str] = []
    for site in sites:
        name = site_type_name(site.site_type)
        if site.is_enhanced:
            name += "*"
        parts.append(name)
    return ", ".join(parts)


def render_atlas_header(
    essence: int,
    completion: int,
    total_battles: int,
    deck_count: int,
    dreamsign_count: int,
) -> str:
    """Build the atlas header with double-line borders.

    Shows the DREAM ATLAS title, essence count, completion level,
    deck count, and dreamsign count.
    """
    sep = draw_double_separator()

    left1 = f"  {BOLD}DREAM ATLAS{RESET}"
    right1 = f"Essence: {essence}"
    vis_left1 = visible_len(left1)
    gap1 = max(2, CONTENT_WIDTH - vis_left1 - len(right1))
    line1 = f"{left1}{' ' * gap1}{right1}"

    left2 = f"  Completion: {completion}/{total_battles}"
    right2 = f"Deck: {deck_count} cards | Dreamsigns: {dreamsign_count}"
    gap2 = max(2, CONTENT_WIDTH - len(left2) - len(right2))
    line2 = f"{left2}{' ' * gap2}{right2}"

    return "\n".join([sep, line1, line2, sep])


def render_available_dreamscapes(
    available_nodes: list[DreamscapeNode],
    selected_index: int,
) -> str:
    """Render the list of available dreamscapes with site summaries.

    The node at selected_index is highlighted with a '>' marker.
    Each node shows its name, biome, and site summary.
    """
    if not available_nodes:
        return f"  {DIM}No dreamscapes available.{RESET}"

    lines: list[str] = ["", "  Available Dreamscapes:"]
    for i, node in enumerate(available_nodes):
        marker = ">" if i == selected_index else " "
        biome_color = _BIOME_MARKERS.get(node.biome, "")
        biome_label = f"{biome_color}{node.biome.value}{RESET}"
        summary = dreamscape_site_summary(node.sites)
        lines.append(
            f"  {marker} [{node.name}] ({biome_label})  -- {summary}"
        )

    return "\n".join(lines)


def render_completed_trail(all_nodes: list[DreamscapeNode]) -> str:
    """Render the chain of completed dreamscapes.

    Shows completed nodes in order of their node_id, joined by '->'.
    """
    completed = [
        n for n in all_nodes if n.state == NodeState.COMPLETED
    ]
    completed.sort(key=lambda n: n.node_id)

    if not completed:
        return ""

    trail = " -> ".join(f"[{n.name}]" for n in completed)
    return f"\n  Completed:\n    {trail}"


def render_dreamscape_sites(
    sites: list[Site],
    selected_index: int,
) -> str:
    """Render the site list for an entered dreamscape.

    Visited sites show a checkmark and are dimmed. Unvisited sites
    show a selectable indicator. Battle is marked [locked] until
    all other sites are visited.
    """
    all_non_battle_visited = all(
        s.is_visited for s in sites if s.site_type != SiteType.BATTLE
    )

    lines: list[str] = []
    for i, site in enumerate(sites):
        name = site_type_name(site.site_type)
        if site.is_enhanced:
            name += "*"

        if site.is_visited:
            marker = " "
            lines.append(f"  {marker} {DIM}\u2713 {name}{RESET}")
        elif site.site_type == SiteType.BATTLE and not all_non_battle_visited:
            marker = " "
            lines.append(f"  {marker}   {name} [locked]")
        else:
            marker = ">" if i == selected_index else " "
            lines.append(f"  {marker}   {name}")

    return "\n".join(lines)


def render_full_atlas(
    available_nodes: list[DreamscapeNode],
    all_nodes: list[DreamscapeNode],
    selected_index: int,
    essence: int,
    completion: int,
    total_battles: int,
    deck_count: int,
    dreamsign_count: int,
) -> str:
    """Render the complete Dream Atlas display.

    Combines header, available dreamscapes, completed trail,
    and navigation prompt.
    """
    parts: list[str] = [
        render_atlas_header(
            essence=essence,
            completion=completion,
            total_battles=total_battles,
            deck_count=deck_count,
            dreamsign_count=dreamsign_count,
        ),
        render_available_dreamscapes(available_nodes, selected_index),
        render_completed_trail(all_nodes),
        "",
        f"  {DIM}Use arrow keys to select, Enter to visit.{RESET}",
        draw_double_separator(),
    ]

    return "\n".join(parts)
