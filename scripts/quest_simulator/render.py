"""Terminal display formatting for the quest simulator.

Provides color wrappers (via AYU Mirage palette from the draft simulator),
box-drawing utilities, and header/footer/banner templates.
"""

import re

import colors

CONTENT_WIDTH = 70

ARCHETYPE_NAMES: list[str] = [
    "Flash",
    "Awaken",
    "Flicker",
    "Ignite",
    "Shatter",
    "Endure",
    "Submerge",
    "Surge",
]

ARCHETYPE_EMOJI: dict[str, str] = {
    "Endure": "🔄",
    "Shatter": "💀",
    "Ignite": "🔥",
    "Flicker": "✨",
    "Awaken": "🌿",
    "Flash": "🛡️",
    "Surge": "🌊",
    "Submerge": "🌀",
}

ARCHETYPE_RESONANCE: dict[str, tuple[str, str]] = {
    "Flash": ("Thunder", "Tide"),
    "Awaken": ("Thunder", "Flame"),
    "Flicker": ("Flame", "Thunder"),
    "Ignite": ("Flame", "Stone"),
    "Shatter": ("Stone", "Flame"),
    "Endure": ("Stone", "Tide"),
    "Submerge": ("Tide", "Stone"),
    "Surge": ("Tide", "Thunder"),
}


def visible_len(s: str) -> int:
    """Length of string excluding ANSI escape sequences."""
    return len(re.sub(r"\033\[[0-9;]*m", "", s))


def pad_right(s: str, width: int) -> str:
    """Pad string to visible width, accounting for ANSI escape sequences."""
    return s + " " * max(0, width - visible_len(s))


def truncate_visible(s: str, max_width: int) -> str:
    """Truncate a string (possibly with ANSI codes) to max visible width."""
    vlen = visible_len(s)
    if vlen <= max_width:
        return s
    result: list[str] = []
    vis = 0
    in_escape = False
    for ch in s:
        if in_escape:
            result.append(ch)
            if ch == "m":
                in_escape = False
            continue
        if ch == "\033":
            in_escape = True
            result.append(ch)
            continue
        if vis >= max_width:
            break
        result.append(ch)
        vis += 1
    reset_seq = colors.reset()
    if reset_seq:
        result.append(reset_seq)
    return "".join(result)


def draw_box(lines: list[str], min_width: int = 70) -> None:
    """Draw a double-line box around lines of text.

    Each line can contain ANSI escape sequences. The box auto-sizes to
    fit the longest line (with a minimum of `min_width` visible columns
    including the border characters). Content is left-aligned with
    2-space indent inside the borders.
    """
    max_content = max((visible_len(line) for line in lines), default=0)
    inner = max(min_width - 2, max_content + 4)
    print(f"\u2554{'\u2550' * inner}\u2557")
    for line in lines:
        print(f"\u2551  {pad_right(line, inner - 2)}\u2551")
    print(f"\u255a{'\u2550' * inner}\u255d")


def draw_separator() -> str:
    """Full-width single-line separator."""
    return "\u2500" * CONTENT_WIDTH


def draw_double_separator() -> str:
    """Full-width double-line separator."""
    return "\u2550" * CONTENT_WIDTH


def quest_start_banner(
    seed: int,
    starting_essence: int,
    card_count: int = 540,
) -> str:
    """Build the quest start banner text."""
    sep = draw_double_separator()
    title = colors.header("DREAMTIDES QUEST")
    seed_label = f"Seed: {seed}"
    title_vis = visible_len(title)
    gap = max(2, CONTENT_WIDTH - 2 - title_vis - len(seed_label) - 2)
    pool_desc = f"  Card pool: {colors.num(card_count)} card designs"
    lines: list[str] = [
        sep,
        f"  {title}{' ' * gap}{seed_label}",
        sep,
        "",
        f"  Starting essence: {colors.num(starting_essence)}",
        pool_desc,
        "",
        f"  {colors.dim('Press Enter to begin...')}",
        sep,
    ]
    return "\n".join(lines)


def atlas_header(
    essence: int,
    completion: int,
    total_battles: int,
    deck_count: int,
    dreamsign_count: int,
) -> str:
    """Build the atlas header text."""
    sep = draw_double_separator()
    left1 = f"  {colors.header('DREAM ATLAS')}"
    right1 = f"Essence: {essence}"
    vis_left1 = visible_len(left1)
    gap1 = max(2, CONTENT_WIDTH - vis_left1 - len(right1))
    line1 = f"{left1}{' ' * gap1}{right1}"

    left2 = f"  Completion: {completion}/{total_battles}"
    right2 = f"Deck: {deck_count} cards | Dreamsigns: {dreamsign_count}"
    gap2 = max(2, CONTENT_WIDTH - len(left2) - len(right2))
    line2 = f"{left2}{' ' * gap2}{right2}"

    return "\n".join([sep, line1, line2, sep])


def site_visit_header(
    dreamscape_name: str,
    site_type_label: str,
    pick_info: str,
    dreamscape_number: int,
) -> str:
    """Build a site visit header."""
    sep = draw_double_separator()
    left = f"  {dreamscape_name.upper()} -- {site_type_label} -- {pick_info}"
    right = f"[Dreamscape {dreamscape_number}]"
    gap = max(2, CONTENT_WIDTH - len(left) - len(right))
    return "\n".join([sep, f"{left}{' ' * gap}{right}", sep])


# ---------------------------------------------------------------------------
# Backward-compatibility shims for modules not yet migrated to colors API.
# These route through the AYU Mirage colors module instead of using raw ANSI.
# They will be removed once flow.py, render_atlas.py, sites_battle.py, etc.
# are updated.
# ---------------------------------------------------------------------------

BOLD = (
    colors.c("", "accent", bold=True).replace(colors.reset(), "")
    if colors._USE_COLOR
    else ""
)
DIM = colors.c("", "comment").replace(colors.reset(), "") if colors._USE_COLOR else ""
RESET = colors.reset()
STRIKETHROUGH = ""


def format_card(
    card_or_deck_card,
    highlighted: bool = False,
    max_width: int = CONTENT_WIDTH,
    show_images: bool = False,
) -> list[str]:
    """Compatibility shim routing to render_cards.format_card_display."""
    import render_cards

    return render_cards.format_card_display(
        card_or_deck_card,
        highlighted=highlighted,
        max_width=max_width,
        show_images=show_images,
    )


def resonance_profile_footer(
    counts=None, deck_count: int = 0, essence: int = 0, **kwargs
) -> str:
    """Compatibility shim routing to render_status.archetype_preference_footer."""
    import render_status

    w = [0.0] * 8
    return render_status.archetype_preference_footer(
        w=w, deck_count=deck_count, essence=essence
    )
