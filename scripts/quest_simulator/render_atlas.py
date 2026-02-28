"""Dream Atlas terminal display rendering.

Provides functions for rendering the Dream Atlas navigation hub,
dreamscape site lists, site summaries, and player status display.
Imports shared constants and utilities from render.py.
"""

import os
import sys
from typing import Optional

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

_PREVIEW_EXCLUDED_TYPES: frozenset[SiteType] = frozenset(
    {
        SiteType.BATTLE,
        SiteType.DRAFT,
        SiteType.DREAMCALLER_DRAFT,
    }
)

_BIOME_MARKERS: dict[Biome, str] = {
    Biome.VERDANT: "\033[92m",
    Biome.CELESTIAL: "\033[94m",
    Biome.TWILIGHT: "\033[95m",
    Biome.INFERNAL: "\033[91m",
    Biome.ASHEN: "\033[90m",
    Biome.CRYSTALLINE: "\033[96m",
    Biome.PRISMATIC: "\033[1m",
    Biome.MIRRORED: "\033[93m",
    Biome.ARCANE: "\033[1;34m",
}

_BIOME_ENHANCEMENT_TEXT: dict[Biome, str] = {
    Biome.VERDANT: "free reroll",
    Biome.CELESTIAL: "pick 1 of 3",
    Biome.TWILIGHT: "3rd option",
    Biome.INFERNAL: "3 pairs",
    Biome.ASHEN: "purge up to 6",
    Biome.CRYSTALLINE: "doubled",
    Biome.PRISMATIC: "select target",
    Biome.MIRRORED: "choose any",
    Biome.ARCANE: "pick any number",
}

# Disable biome colors when NO_COLOR is set or output is not a terminal
if os.environ.get("NO_COLOR") or not sys.stdout.isatty():
    _BIOME_MARKERS = {b: "" for b in Biome}


def biome_enhancement_text(biome: Biome) -> str:
    """Return the enhancement effect description for a biome."""
    return _BIOME_ENHANCEMENT_TEXT[biome]


def site_type_name(site_type: SiteType) -> str:
    """Map a SiteType enum value to a human-readable display string."""
    return _SITE_TYPE_NAMES.get(site_type, site_type.value)


def _enhanced_label(site: Site, biome: Optional[Biome]) -> str:
    """Build the display label for a site, with enhancement text if applicable."""
    name = site_type_name(site.site_type)
    if site.is_enhanced:
        if biome is not None:
            text = _BIOME_ENHANCEMENT_TEXT.get(biome, "")
            if text:
                name += f" *{text}*"
            else:
                name += "*"
        else:
            name += "*"
    return name


def dreamscape_site_summary(
    sites: list[Site],
    biome: Optional[Biome] = None,
) -> str:
    """Produce a comma-separated summary of site names.

    Enhanced sites are marked with a star and enhancement description
    when a biome is provided, or with just an asterisk otherwise.
    """
    parts: list[str] = []
    for site in sites:
        parts.append(_enhanced_label(site, biome))
    return ", ".join(parts)


def render_atlas_header(
    essence: int,
    completion: int,
    total_battles: int,
    deck_count: int,
    dreamsign_count: int,
    total_nodes: int = 0,
) -> str:
    """Build the atlas header with double-line borders.

    Shows the DREAM ATLAS title, essence count, completion level,
    deck count, dreamsign count, and total dreamscape count.
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

    header_lines = [sep, line1, line2]

    if total_nodes > 0:
        line3 = f"  Atlas: {total_nodes} dreamscapes"
        header_lines.append(line3)

    header_lines.append(sep)
    return "\n".join(header_lines)


def render_available_dreamscapes(
    available_nodes: list[DreamscapeNode],
    selected_index: int,
) -> str:
    """Render the list of available dreamscapes with site summaries.

    The node at selected_index is highlighted with a '>' marker.
    Each node shows its name, biome, and site summary with
    enhancement descriptions.
    """
    if not available_nodes:
        return f"  {DIM}No dreamscapes available.{RESET}"

    lines: list[str] = ["", "  Available Dreamscapes:"]
    for i, node in enumerate(available_nodes):
        marker = ">" if i == selected_index else " "
        biome_color = _BIOME_MARKERS.get(node.biome, "")
        biome_label = f"{biome_color}{node.biome.value}{RESET}"
        prefix = f"  {marker} [{node.name}] ({biome_label})"
        preview_sites = [
            s for s in node.sites if s.site_type not in _PREVIEW_EXCLUDED_TYPES
        ]
        summary = dreamscape_site_summary(preview_sites, biome=node.biome)
        if summary:
            prefix_vis = visible_len(prefix)
            sep = "  -- "
            max_summary = CONTENT_WIDTH - prefix_vis - len(sep)
            if max_summary > 4 and len(summary) > max_summary:
                summary = summary[: max_summary - 3] + "..."
            line = f"{prefix}{sep}{summary}"
        else:
            line = prefix
        lines.append(line)

    return "\n".join(lines)


def _wrap_trail(trail: str, indent: int, max_width: int) -> list[str]:
    """Wrap a trail string into lines that fit within max_width.

    Splits on ' -> ' boundaries so that arrow separators stay with the
    preceding node name.
    """
    prefix = " " * indent
    if len(prefix) + len(trail) <= max_width:
        return [prefix + trail]

    segments = trail.split(" -> ")
    lines: list[str] = []
    current = prefix
    for j, seg in enumerate(segments):
        if j == 0:
            candidate = current + seg
        else:
            candidate = current + " -> " + seg
        if len(candidate) > max_width and current != prefix:
            lines.append(current)
            current = prefix + seg
        else:
            current = candidate
    if current.strip():
        lines.append(current)
    return lines


def render_completed_trail(all_nodes: list[DreamscapeNode]) -> str:
    """Render the chain of completed dreamscapes.

    Shows completed nodes in order of their node_id, joined by '->'.
    Wraps to multiple lines if the trail exceeds 70 columns.
    """
    completed = [n for n in all_nodes if n.state == NodeState.COMPLETED]
    completed.sort(key=lambda n: n.node_id)

    if not completed:
        return ""

    trail = " -> ".join(f"[{n.name}]" for n in completed)
    wrapped = _wrap_trail(trail, indent=4, max_width=CONTENT_WIDTH)
    return "\n  Completed:\n" + "\n".join(wrapped)


def render_dreamscape_sites(
    sites: list[Site],
    selected_index: int,
    biome: Optional[Biome] = None,
) -> str:
    """Render the site list for an entered dreamscape.

    Visited sites show a checkmark and are dimmed. Unvisited sites
    show a selectable indicator. Battle is marked [locked] until
    all other sites are visited. When biome is provided, enhanced
    sites show the enhancement description.
    """
    all_non_battle_visited = all(
        s.is_visited for s in sites if s.site_type != SiteType.BATTLE
    )

    lines: list[str] = []
    for i, site in enumerate(sites):
        name = _enhanced_label(site, biome)

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
            total_nodes=len(all_nodes),
        ),
        render_available_dreamscapes(available_nodes, selected_index),
        render_completed_trail(all_nodes),
        "",
        f"  {DIM}Use arrow keys to select, Enter to visit.{RESET}",
        draw_double_separator(),
    ]

    return "\n".join(parts)
