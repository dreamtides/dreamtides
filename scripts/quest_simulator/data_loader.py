"""Load all data files and construct typed model objects.

Reads cards.json, card_data.json, config.toml, and all TOML data files
from the data/ subdirectory. Called once at startup to produce the full
set of typed model objects for the quest simulator.
"""

import json
import tomllib
from pathlib import Path
from types import MappingProxyType
from typing import Any

from models import (
    AlgorithmParams,
    BaneCard,
    Boss,
    Card,
    CardType,
    DraftParams,
    Dreamcaller,
    Dreamsign,
    EffectType,
    Journey,
    PoolParams,
    Rarity,
    Resonance,
    TemptingOffer,
)

DATA_DIR: Path = Path(__file__).resolve().parent / "data"

_RESONANCE_MAP: dict[str, Resonance] = {r.value: r for r in Resonance}
_CARD_TYPE_MAP: dict[str, CardType] = {ct.value: ct for ct in CardType}
_RARITY_MAP: dict[str, Rarity] = {r.value: r for r in Rarity}
_EFFECT_TYPE_MAP: dict[str, EffectType] = {et.value: et for et in EffectType}


def _parse_resonance(value: str) -> Resonance:
    """Convert a resonance string to the Resonance enum."""
    result = _RESONANCE_MAP.get(value)
    if result is None:
        raise ValueError(f"Unknown resonance: {value!r}")
    return result


def _parse_resonances(values: list[str]) -> frozenset[Resonance]:
    """Convert a list of resonance strings to a frozenset of Resonance enums."""
    return frozenset(_parse_resonance(v) for v in values)


def _parse_card_type(value: str) -> CardType:
    """Convert a card type string to the CardType enum."""
    result = _CARD_TYPE_MAP.get(value)
    if result is None:
        raise ValueError(f"Unknown card type: {value!r}")
    return result


def _parse_rarity(value: str) -> Rarity:
    """Convert a rarity string to the Rarity enum."""
    result = _RARITY_MAP.get(value)
    if result is None:
        raise ValueError(f"Unknown rarity: {value!r}")
    return result


def _parse_effect_type(value: str) -> EffectType:
    """Convert an effect type string to the EffectType enum."""
    result = _EFFECT_TYPE_MAP.get(value)
    if result is None:
        raise ValueError(f"Unknown effect type: {value!r}")
    return result


def load_cards() -> list[Card]:
    """Load and merge cards.json with card_data.json into Card objects."""
    with open(DATA_DIR / "cards.json") as f:
        raw_cards: list[dict[str, Any]] = json.load(f)

    with open(DATA_DIR / "card_data.json") as f:
        raw_card_data: list[dict[str, Any]] = json.load(f)

    card_data_by_number: dict[int, dict[str, Any]] = {
        entry["card_number"]: entry for entry in raw_card_data
    }

    cards: list[Card] = []
    for raw in raw_cards:
        card_number: int = raw["card_number"]
        extra = card_data_by_number.get(card_number, {})

        cards.append(
            Card(
                name=raw["name"],
                card_number=card_number,
                energy_cost=raw.get("energy_cost"),
                card_type=_parse_card_type(raw["card_type"]),
                subtype=raw.get("subtype"),
                is_fast=raw["is_fast"],
                spark=raw.get("spark"),
                rarity=_parse_rarity(raw["rarity"]),
                rules_text=raw["rules_text"],
                resonances=_parse_resonances(extra.get("resonance", [])),
                tags=frozenset(extra.get("tags", [])),
            )
        )

    return cards


def load_config() -> (
    tuple[AlgorithmParams, DraftParams, PoolParams, dict[str, dict[str, Any]]]
):
    """Load config.toml and return typed parameter objects.

    Returns a tuple of (AlgorithmParams, DraftParams, PoolParams, extra_config)
    where extra_config is a dict of remaining config sections.
    """
    with open(DATA_DIR / "config.toml", "rb") as f:
        raw: dict[str, Any] = tomllib.load(f)

    resonance_cfg = raw["resonance"]
    staleness_cfg = raw["staleness"]
    algorithm_params = AlgorithmParams(
        exponent=float(resonance_cfg["exponent"]),
        floor_weight=float(resonance_cfg["floor_weight"]),
        neutral_base=float(resonance_cfg["neutral_base"]),
        staleness_factor=float(staleness_cfg["factor"]),
    )

    draft_cfg = raw["draft"]
    draft_params = DraftParams(
        cards_per_pick=draft_cfg["cards_per_pick"],
        picks_per_site=draft_cfg["picks_per_site"],
    )

    pool_cfg = raw["draft_pool"]
    pool_params = PoolParams(
        copies_common=pool_cfg["copies_common"],
        copies_uncommon=pool_cfg["copies_uncommon"],
        copies_rare=pool_cfg["copies_rare"],
        copies_legendary=pool_cfg["copies_legendary"],
        variance_min=float(pool_cfg["variance_min"]),
        variance_max=float(pool_cfg["variance_max"]),
    )

    extra_sections = {
        k: v
        for k, v in raw.items()
        if k not in ("resonance", "staleness", "draft", "draft_pool")
        and isinstance(v, dict)
    }

    return algorithm_params, draft_params, pool_params, extra_sections


def load_dreamcallers() -> list[Dreamcaller]:
    """Load dreamcallers.toml and construct Dreamcaller objects."""
    with open(DATA_DIR / "dreamcallers.toml", "rb") as f:
        raw: dict[str, Any] = tomllib.load(f)

    return [
        Dreamcaller(
            name=entry["name"],
            resonances=_parse_resonances(entry["resonance"]),
            resonance_bonus=MappingProxyType(dict(entry["resonance_bonus"])),
            tags=frozenset(entry["tags"]),
            tag_bonus=MappingProxyType(dict(entry["tag_bonus"])),
            essence_bonus=entry["essence_bonus"],
            ability_text=entry["ability_text"],
        )
        for entry in raw["dreamcallers"]
    ]


def load_dreamsigns() -> list[Dreamsign]:
    """Load dreamsigns.toml and construct Dreamsign objects."""
    with open(DATA_DIR / "dreamsigns.toml", "rb") as f:
        raw: dict[str, Any] = tomllib.load(f)

    return [
        Dreamsign(
            name=entry["name"],
            resonance=_parse_resonance(entry["resonance"]),
            tags=frozenset(entry["tags"]),
            effect_text=entry["effect_text"],
            is_bane=entry.get("is_bane", False),
        )
        for entry in raw["dreamsigns"]
    ]


def load_journeys() -> list[Journey]:
    """Load journeys.toml and construct Journey objects."""
    with open(DATA_DIR / "journeys.toml", "rb") as f:
        raw: dict[str, Any] = tomllib.load(f)

    return [
        Journey(
            name=entry["name"],
            description=entry["description"],
            effect_type=_parse_effect_type(entry["effect_type"]),
            effect_value=entry["effect_value"],
        )
        for entry in raw["journeys"]
    ]


def load_offers() -> list[TemptingOffer]:
    """Load offers.toml and construct TemptingOffer objects."""
    with open(DATA_DIR / "offers.toml", "rb") as f:
        raw: dict[str, Any] = tomllib.load(f)

    return [
        TemptingOffer(
            reward_name=entry["reward_name"],
            reward_description=entry["reward_description"],
            reward_effect_type=_parse_effect_type(entry["reward_effect_type"]),
            reward_value=entry["reward_value"],
            cost_name=entry["cost_name"],
            cost_description=entry["cost_description"],
            cost_effect_type=_parse_effect_type(entry["cost_effect_type"]),
            cost_value=entry["cost_value"],
        )
        for entry in raw["offers"]
    ]


def load_banes() -> list[BaneCard]:
    """Load banes.toml and construct BaneCard objects."""
    with open(DATA_DIR / "banes.toml", "rb") as f:
        raw: dict[str, Any] = tomllib.load(f)

    return [
        BaneCard(
            name=entry["name"],
            rules_text=entry["rules_text"],
            card_type=_parse_card_type(entry["card_type"]),
            energy_cost=entry["energy_cost"],
        )
        for entry in raw["banes"]
    ]


def load_bosses() -> list[Boss]:
    """Load bosses.toml and construct Boss objects."""
    with open(DATA_DIR / "bosses.toml", "rb") as f:
        raw: dict[str, Any] = tomllib.load(f)

    return [
        Boss(
            name=entry["name"],
            archetype=entry["archetype"],
            ability_text=entry["ability_text"],
            deck_description=entry["deck_description"],
            is_final=entry["is_final"],
            resonances=_parse_resonances(entry["resonance"]),
        )
        for entry in raw["bosses"]
    ]
