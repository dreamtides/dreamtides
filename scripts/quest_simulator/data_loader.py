"""Load all data files and construct typed model objects.

Reads config.toml and all TOML data files from the data/ subdirectory.
Called once at startup to produce the full set of typed model objects
for the quest simulator.
"""

import tomllib
from pathlib import Path
from typing import Any

from models import (
    BaneCard,
    Boss,
    Dreamcaller,
    Dreamsign,
    EffectType,
    Journey,
    TemptingOffer,
)

DATA_DIR: Path = Path(__file__).resolve().parent / "data"

_EFFECT_TYPE_MAP: dict[str, EffectType] = {et.value: et for et in EffectType}


def _parse_effect_type(value: str) -> EffectType:
    """Convert an effect type string to the EffectType enum."""
    result = _EFFECT_TYPE_MAP.get(value)
    if result is None:
        raise ValueError(f"Unknown effect type: {value!r}")
    return result


def load_config() -> dict[str, dict[str, Any]]:
    """Load config.toml and return a dict of config sections.

    Returns a dict mapping section names to their key-value pairs.
    """
    with open(DATA_DIR / "config.toml", "rb") as f:
        raw: dict[str, Any] = tomllib.load(f)

    return {
        k: v
        for k, v in raw.items()
        if isinstance(v, dict)
    }


def load_dreamcallers() -> list[Dreamcaller]:
    """Load dreamcallers.toml and construct Dreamcaller objects."""
    with open(DATA_DIR / "dreamcallers.toml", "rb") as f:
        raw: dict[str, Any] = tomllib.load(f)

    return [
        Dreamcaller(
            name=entry["name"],
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
            card_type=entry["card_type"],
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
        )
        for entry in raw["bosses"]
    ]
