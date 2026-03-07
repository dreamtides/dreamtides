"""Card display formatting for the quest simulator.

Provides card display formatting for DeckCards and the full deck
viewer. Uses AYU Mirage palette colors from the draft simulator.
"""

import base64
import io
import os
import sys
import textwrap
from typing import Optional

import colors
from models import DeckCard, Dreamcaller, Dreamsign

import image_cache
import render

ARCHETYPE_NAMES = render.ARCHETYPE_NAMES

try:
    from imgcat import imgcat as _imgcat

    _IMGCAT_AVAILABLE = True
except ImportError:
    _IMGCAT_AVAILABLE = False


def _render_image_line(image_number: int) -> str | None:
    """Return an imgcat escape sequence for the given image number."""
    if not _IMGCAT_AVAILABLE:
        return None
    path = image_cache.get_image_path(image_number)
    if path is None:
        return None
    try:
        buf = io.BytesIO()
        with open(path, "rb") as f:
            _imgcat(f.read(), width=20, height=8, fp=buf)
        return buf.getvalue().decode("utf-8", errors="replace")
    except Exception:
        return None


def _top_archetype(fitness: list[float]) -> str:
    """Return a label for the card's top archetype fitness value."""
    if not fitness:
        return ""
    best_idx = 0
    best_val = fitness[0]
    for i, v in enumerate(fitness):
        if v > best_val:
            best_val = v
            best_idx = i
    name = (
        ARCHETYPE_NAMES[best_idx] if best_idx < len(ARCHETYPE_NAMES) else f"A{best_idx}"
    )
    return f"{name}={best_val:.2f}"


def format_card_display(
    card_or_deck_card,
    highlighted: bool = False,
    max_width: int = render.CONTENT_WIDTH,
    show_images: bool = False,
) -> list[str]:
    """Format a card as display lines.

    Accepts a DeckCard, CardInstance, or CardDesign. Shows the card
    name (colored), power, commit, flex, and top archetype fitness.

    For real cards (is_real=True), returns up to 4+ lines:
      Line 1: marker + colored card name (with transfig/bane markers)
      Line 2: energy cost + card type + subtype + rarity
      Line 3+: rules text (word-wrapped)
      Last:   power/commit/flex + top archetype
    For synthetic cards, returns 2 lines (name + stats).
    """
    marker = ">" if highlighted else " "

    # Unwrap to get the design and deck-card metadata
    is_bane = False
    transfig_note = None
    design = None

    if isinstance(card_or_deck_card, DeckCard):
        is_bane = card_or_deck_card.is_bane
        transfig_note = card_or_deck_card.transfig_note
        instance = card_or_deck_card.instance
        if hasattr(instance, "design"):
            design = instance.design
        else:
            design = instance
    elif hasattr(card_or_deck_card, "design"):
        # CardInstance
        design = card_or_deck_card.design
    else:
        # CardDesign directly
        design = card_or_deck_card

    # Extract name
    if design is not None and hasattr(design, "name"):
        card_name = design.name
    else:
        card_name = str(card_or_deck_card)

    # Build display name
    if transfig_note is not None:
        display_name = transfig_note
    else:
        display_name = card_name

    prefix = f"  {marker} "
    max_name_width = max_width - len(prefix) - 2
    if max_name_width < 1:
        max_name_width = 1
    if len(display_name) > max_name_width:
        display_name = display_name[: max_name_width - 1] + "\u2026"

    colored_name = colors.card(display_name)
    bane_marker = f"  {colors.c('[BANE]', 'error', bold=True)}" if is_bane else ""
    line1 = f"{prefix}{colored_name}{bane_marker}"

    lines = [line1]

    # For real cards, add type line and rules text
    is_real = getattr(design, "is_real", False) if design else False
    if is_real and design is not None:
        # Line 2: energy cost + card type + subtype + rarity
        type_parts: list[str] = []
        energy = getattr(design, "energy_cost", None)
        if energy is not None:
            type_parts.append(f"{energy}E")
        ct = getattr(design, "card_type", "")
        if ct:
            type_parts.append(ct)
        sub = getattr(design, "subtype", "")
        if sub:
            type_parts.append(f"- {sub}")
        rarity = getattr(design, "rarity", "")
        if rarity:
            type_parts.append(f" ({rarity.title()})")
        type_line = " ".join(type_parts)
        lines.append(f"    {colors.dim(type_line)}")

        # Inline image (only in non-interactive contexts)
        if show_images:
            img_num = getattr(design, "image_number", None)
            if img_num is not None:
                img_line = _render_image_line(img_num)
                if img_line is not None:
                    lines.append(img_line)

        # Line 3+: rules text
        rules = getattr(design, "rules_text", "")
        if rules:
            wrap_width = max_width - 4
            wrapped = textwrap.wrap(rules, width=wrap_width)
            for wline in wrapped:
                lines.append(f"    {wline}")

    # Stats line: power/commit/flex + top archetype
    details_parts: list[str] = []
    if design is not None:
        if hasattr(design, "power"):
            details_parts.append(f"Power: {colors.num(f'{design.power:.2f}')}")
        if hasattr(design, "commit"):
            details_parts.append(f"Commit: {colors.num(f'{design.commit:.2f}')}")
        if hasattr(design, "flex"):
            details_parts.append(f"Flex: {colors.num(f'{design.flex:.2f}')}")
        if hasattr(design, "fitness"):
            arch = _top_archetype(design.fitness)
            if arch:
                details_parts.append(colors.dim(arch))

    detail_str = "  ".join(details_parts) if details_parts else ""
    lines.append(f"    {detail_str}" if detail_str else "    ")

    return lines


def card_name(card) -> str:
    """Extract the display name from a CardInstance or CardDesign."""
    if hasattr(card, "design") and hasattr(card.design, "name"):
        return card.design.name
    if hasattr(card, "name"):
        return card.name
    return str(card)


def _get_design(card):
    """Extract the card design from various card wrapper types."""
    if isinstance(card, DeckCard):
        instance = card.instance
        if hasattr(instance, "design"):
            return instance.design
        return instance
    if hasattr(card, "design"):
        return card.design
    return card


_IMG_WIDTH = 15
_IMG_HEIGHT = 6


def _overlay_image(card, lines_printed: int) -> None:
    """Overlay an image at the left of the current card block.

    After text lines have been printed, saves cursor position, moves
    up, renders the image via imgcat, then restores cursor.

    All writes go through sys.stdout (text mode) to avoid interleaving
    issues between sys.stdout and sys.stdout.buffer which can cause
    the terminal to silently drop escape sequences.
    """
    design = _get_design(card)
    img_num = getattr(design, "image_number", None) if design else None
    if img_num is None:
        return
    path = image_cache.get_image_path(img_num)
    if path is None:
        return
    try:
        with open(path, "rb") as f:
            img_data = f.read()
    except Exception:
        return

    is_tmux = "TMUX" in os.environ and "tmux" in os.environ.get("TMUX", "")

    if is_tmux:
        # Tmux: must use binary passthrough, flush text stream first
        sys.stdout.flush()
        fp = sys.stdout.buffer
        CSI = b"\033["
        OSC = b"\033]"
        ST = b"\a"
        fp.write(b"\0337")
        fp.write(CSI + str(lines_printed).encode() + b"F")
        fp.write(b"\033Ptmux;\033")
        fp.write(OSC + b"1337;File=inline=1")
        fp.write(b";size=" + str(len(img_data)).encode())
        fp.write(b";height=" + str(_IMG_HEIGHT).encode())
        fp.write(b";width=" + str(_IMG_WIDTH).encode())
        fp.write(b":")
        fp.write(base64.b64encode(img_data))
        fp.write(ST)
        fp.write(b"\033\\")
        fp.write(b"\0338")
        fp.flush()
    else:
        # Non-tmux: generate image via imgcat, write everything through
        # sys.stdout (text mode) so cursor controls and image data share
        # the same buffered stream as print() output.
        img_buf = io.BytesIO()
        _imgcat(img_data, width=_IMG_WIDTH, height=_IMG_HEIGHT, fp=img_buf)
        img_str = img_buf.getvalue().decode("utf-8", errors="replace")
        sys.stdout.write(f"\0337\033[{lines_printed}F")
        sys.stdout.write(img_str)
        sys.stdout.write("\0338")
        sys.stdout.flush()


def _render_card_block(card) -> None:
    """Render a single card with image on the left and text on the right."""
    design = _get_design(card)
    text_col = _IMG_WIDTH + 2
    text_width = render.CONTENT_WIDTH - text_col

    # Build text lines
    text_lines: list[str] = []

    name = card_name(card)
    if len(name) > text_width:
        name = name[: text_width - 1] + "\u2026"
    text_lines.append(colors.card(name))

    is_real = getattr(design, "is_real", False) if design else False
    if is_real and design is not None:
        type_parts: list[str] = []
        energy = getattr(design, "energy_cost", None)
        if energy is not None:
            type_parts.append(f"{energy}E")
        ct = getattr(design, "card_type", "")
        if ct:
            type_parts.append(ct)
        sub = getattr(design, "subtype", "")
        if sub:
            type_parts.append(f"- {sub}")
        text_lines.append(colors.dim(" ".join(type_parts)))

        rules = getattr(design, "rules_text", "")
        if rules:
            wrapped = textwrap.wrap(rules, width=text_width)
            text_lines.extend(wrapped)

    # Ensure at least img_height lines so image area is fully allocated
    while len(text_lines) < _IMG_HEIGHT:
        text_lines.append("")

    # Print text with left padding to leave space for image
    padding = " " * text_col
    for line in text_lines:
        print(f"{padding}{line}")

    # Overlay image on the left
    if _IMGCAT_AVAILABLE:
        _overlay_image(card, len(text_lines))


def render_card_columns(cards) -> None:
    """Render cards with image on the left and text on the right.

    Each card is displayed as a block: image at columns 1-15, card
    info (name, type, rules text) at columns 17+. Prints directly
    to stdout.
    """
    if not cards:
        return

    for card in cards:
        _render_card_block(card)


def _deck_card_sort_key(dc: DeckCard) -> tuple[str, str]:
    """Sort key for deck cards: card name."""
    instance = dc.instance
    if hasattr(instance, "design") and hasattr(instance.design, "name"):
        return ("", instance.design.name)
    if hasattr(instance, "name"):
        return ("", instance.name)
    return ("\xff", "")


def render_full_deck_view(
    deck_cards: list[DeckCard],
    dreamsigns: Optional[list[Dreamsign]] = None,
    dreamcaller: Optional[Dreamcaller] = None,
    essence: Optional[int] = None,
) -> str:
    """Render a full deck viewer display.

    Shows all cards sorted by name, with dreamsigns, dreamcaller info,
    and essence.
    """
    sep = render.draw_double_separator()
    single_sep = render.draw_separator()
    total = len(deck_cards)
    lines: list[str] = [sep]

    # Header
    header_left = f"  {colors.header('DECK VIEWER')}"
    header_right = f"Deck: {total} cards"
    vis_left = render.visible_len(header_left)
    gap = max(2, render.CONTENT_WIDTH - vis_left - len(header_right))
    lines.append(f"{header_left}{' ' * gap}{header_right}")

    if essence is not None:
        lines.append(f"  Essence: {colors.num(essence)}")

    lines.append(sep)
    lines.append("")

    # Card list sorted by name
    sorted_deck = sorted(deck_cards, key=_deck_card_sort_key)

    for dc in sorted_deck:
        card_lines = format_card_display(dc, highlighted=False, show_images=True)
        lines.extend(card_lines)

    lines.append("")
    lines.append(single_sep)

    # Dreamsigns section
    if dreamsigns:
        lines.append("")
        lines.append(f"  {colors.section('Dreamsigns')} ({len(dreamsigns)})")
        for ds in dreamsigns:
            bane_label = (
                f" {colors.c('[BANE]', 'error', bold=True)}" if ds.is_bane else ""
            )
            lines.append(f"    {ds.name}{bane_label}")

    # Dreamcaller section
    if dreamcaller is not None:
        lines.append("")
        lines.append(single_sep)
        lines.append(f"  {colors.section('Dreamcaller')}")
        lines.append(f"    {dreamcaller.name}")
        lines.append(f'    "{dreamcaller.ability_text}"')

    lines.append("")
    lines.append(sep)

    return "\n".join(lines)
