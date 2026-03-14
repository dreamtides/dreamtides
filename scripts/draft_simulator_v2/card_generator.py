"""Card loading for the draft simulator v2.

Loads card designs exclusively from rendered-cards.toml using binary
archetype fitness vectors. No synthetic generation, no card-metadata.toml.
Stdlib-only except for tomllib (Python 3.11+).
"""

import sys
from typing import Optional

import colors
from config import SimulatorConfig
from draft_models import RARITY_VALUES, CardDesign

ARCHETYPE_KEYS = [
    "flash",
    "awaken",
    "flicker",
    "ignite",
    "shatter",
    "endure",
    "submerge",
    "surge",
]


def generate_cards(cfg: SimulatorConfig, rng: object) -> list[CardDesign]:
    """Load card designs from rendered-cards.toml."""
    if cfg.cards.rendered_toml_path is None:
        raise ValueError("cards.rendered_toml_path must be set")
    cards = load_cards(cfg.cards.rendered_toml_path, cfg.cards.original_only)
    _validate_cards(cards, cfg.cards.archetype_count)
    return cards


def load_cards(
    rendered_toml_path: str, original_only: bool = False
) -> list[CardDesign]:
    """Load card designs using only rendered-cards.toml.

    Builds fitness vectors from boolean archetype tags in the TOML file
    (e.g. ``shatter = true``). Computes rarity_value from the card's
    rarity tier.

    When ``original_only`` is True, only cards with card-number <= 220
    are included (the original card set).
    """
    import tomllib

    with open(rendered_toml_path, "rb") as f:
        rendered_data = tomllib.load(f)

    cards: list[CardDesign] = []
    for card in rendered_data.get("cards", []):
        raw_rarity = card.get("rarity", "")
        if raw_rarity == "Special":
            continue

        if original_only:
            card_number = card.get("card-number", None)
            if card_number is None or card_number > 220:
                continue

        card_id = card.get("id", "")
        fitness = [1.0 if card.get(key, False) else 0.0 for key in ARCHETYPE_KEYS]

        if raw_rarity == "Legendary":
            rarity = "rare"
        else:
            rarity = raw_rarity.lower()

        rarity_value = RARITY_VALUES.get(rarity, 0.0)

        raw_spark = card.get("spark", "")
        spark = _parse_optional_int(raw_spark)
        raw_energy = card.get("energy-cost", "")
        energy_cost = _parse_optional_int(raw_energy)

        cards.append(
            CardDesign(
                card_id=card_id,
                name=card.get("name", ""),
                fitness=fitness,
                rarity=rarity,
                rarity_value=rarity_value,
                rules_text=card.get("rendered-text", ""),
                energy_cost=energy_cost,
                card_type=card.get("card-type", ""),
                subtype=card.get("subtype", ""),
                spark=spark,
                is_fast=card.get("is-fast", False),
                image_number=_parse_optional_int(card.get("image-number", "")),
                resonance=tuple(card.get("resonance", [])),
                original_rarity=raw_rarity,
            )
        )

    return cards


def _parse_optional_int(value: object) -> Optional[int]:
    """Parse a value that may be an int, empty string, or non-numeric string."""
    if isinstance(value, int):
        return value
    if isinstance(value, str) and value != "":
        try:
            return int(value)
        except ValueError:
            return None
    return None


def _validate_cards(cards: list[CardDesign], archetype_count: int) -> None:
    """Validate all card attributes. Raises ValueError for hard errors."""
    for card in cards:
        if len(card.fitness) != archetype_count:
            raise ValueError(
                f"Card {card.card_id!r}: fitness vector length "
                f"{len(card.fitness)} != archetype_count {archetype_count}"
            )

        for i, f in enumerate(card.fitness):
            if f < 0.0 or f > 1.0:
                raise ValueError(
                    f"Card {card.card_id!r}: fitness[{i}]={f} not in [0, 1]"
                )


def print_card_pool_stats(cards: list[CardDesign], archetype_count: int) -> None:
    """Print summary statistics for a card pool."""
    print(f"\n{colors.section('Card Pool Statistics:')}")
    print(f"  {colors.label('Total cards:')} {colors.num(len(cards))}")

    # Rarity distribution
    rarity_counts: dict[str, int] = {}
    for c in cards:
        tier = c.rarity if c.rarity else "unknown"
        rarity_counts[tier] = rarity_counts.get(tier, 0) + 1
    print(f"\n  {colors.label('Rarity distribution:')}")
    for tier, count in sorted(rarity_counts.items(), key=lambda x: -x[1]):
        print(
            f"    {colors.label(f'{tier.title():<12s}')} "
            f"{colors.num(count)} ({colors.num(f'{100*count/len(cards):.1f}')}%)"
        )

    # Per-archetype tag counts
    print(f"\n  {colors.label('Cards per archetype (tag = 1.0):')}")
    for arch_idx, arch_name in enumerate(ARCHETYPE_KEYS):
        count = sum(1 for c in cards if c.fitness[arch_idx] == 1.0)
        print(
            f"    {colors.label(f'{arch_name:<12s}')} {colors.num(count)}"
        )

    # Tag count distribution
    print(f"\n  {colors.label('Tag count distribution:')}")
    tag_counts: dict[int, int] = {}
    for c in cards:
        n_tags = sum(1 for f in c.fitness if f == 1.0)
        tag_counts[n_tags] = tag_counts.get(n_tags, 0) + 1
    for n_tags in sorted(tag_counts.keys()):
        count = tag_counts[n_tags]
        label = f"{n_tags} tag{'s' if n_tags != 1 else ''}"
        print(
            f"    {colors.label(f'{label:<8s}')} {colors.num(count)} "
            f"({colors.num(f'{100*count/len(cards):.1f}')}%)"
        )


def describe_card_pool(
    cards: list[CardDesign], archetype_count: int, source: str = "rendered-cards.toml"
) -> str:
    """Return a summary describing the card pool."""
    lines: list[str] = []

    lines.append(colors.c("=" * 60, "ui"))
    lines.append(colors.header("CARD POOL SUMMARY"))
    lines.append(colors.c("=" * 60, "ui"))
    lines.append(
        f"{colors.label('Source:')} {colors.c(source, 'special')}  |  "
        f"{colors.num(len(cards))} distinct designs  |  "
        f"{colors.num(archetype_count)} archetypes"
    )
    lines.append("")

    # Rarity Distribution
    rarity_counts: dict[str, int] = {}
    for c in cards:
        if c.rarity:
            rarity_counts[c.rarity] = rarity_counts.get(c.rarity, 0) + 1
    if rarity_counts:
        lines.append(colors.section("--- Rarity Distribution ---"))
        for tier, count in sorted(rarity_counts.items(), key=lambda x: -x[1]):
            lines.append(
                f"  {colors.label(f'{tier.title() + ":":<12s}')} "
                f"{colors.num(f'{count:>4d}')}  "
                f"({colors.num(f'{100*count/len(cards):.1f}')}%)"
            )
        lines.append("")

    # Per-Archetype Coverage
    lines.append(colors.section("--- Per-Archetype Coverage ---"))
    for arch_idx, arch_name in enumerate(ARCHETYPE_KEYS):
        count = sum(1 for c in cards if c.fitness[arch_idx] == 1.0)
        lines.append(
            f"  {colors.label(f'{arch_name:<12s}')} {colors.num(f'{count:>4d}')}"
        )
    lines.append("")

    # Tag count distribution
    lines.append(colors.section("--- Tag Count Distribution ---"))
    tag_counts: dict[int, int] = {}
    for c in cards:
        n_tags = sum(1 for f in c.fitness if f == 1.0)
        tag_counts[n_tags] = tag_counts.get(n_tags, 0) + 1
    for n_tags in sorted(tag_counts.keys()):
        count = tag_counts[n_tags]
        label = f"{n_tags} tag{'s' if n_tags != 1 else ''}"
        lines.append(
            f"  {colors.label(f'{label + ":":<10s}')} "
            f"{colors.num(f'{count:>4d}')}  "
            f"({colors.num(f'{100*count/len(cards):.1f}')}%)"
        )

    lines.append(colors.c("=" * 60, "ui"))
    return "\n".join(lines)
