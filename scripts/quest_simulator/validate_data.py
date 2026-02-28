"""Validate quest simulator TOML data files against schema invariants."""

import sys
import tomllib
from pathlib import Path

DATA_DIR = Path(__file__).parent / "data"

VALID_RESONANCES = frozenset({"Tide", "Ember", "Zephyr", "Stone", "Ruin"})

VALID_SUBTYPES = frozenset({
    "Survivor", "Warrior", "Spirit Animal", "Ancient", "Visitor",
    "Explorer", "Synth", "Outsider", "Musician", "Mage",
})

VALID_MECHANICS = frozenset({
    "foresee", "draw", "kindle", "fast", "prevent", "dissolve",
    "reclaim", "discard", "copy", "banish", "discover",
})

VALID_ROLES = frozenset({"finisher", "removal", "engine"})

VALID_EFFECT_TYPES = frozenset({
    "add_cards", "add_essence", "remove_cards", "add_dreamsign",
    "gain_resonance",
})

EFFECT_VALUE_RANGES: dict[str, tuple[int, int]] = {
    "add_cards": (1, 3),
    "add_essence": (50, 200),
    "remove_cards": (1, 3),
    "add_dreamsign": (1, 1),
    "gain_resonance": (1, 3),
}

MIN_EFFECT_TYPE_COUNTS: dict[str, int] = {
    "add_cards": 3,
    "add_essence": 3,
    "remove_cards": 2,
    "add_dreamsign": 2,
    "gain_resonance": 2,
}

VALID_BANE_CARD_TYPES = frozenset({"Event"})


def validate_tag(tag: str, errors: list[str], context: str) -> None:
    """Validate a tag string has a known prefix and valid value."""
    if ":" not in tag:
        errors.append(f"{context}: tag '{tag}' missing ':' separator")
        return
    prefix, value = tag.split(":", 1)
    if prefix == "tribal":
        normalized = value.replace("-", " ").title()
        if normalized not in VALID_SUBTYPES:
            errors.append(
                f"{context}: unknown tribal subtype '{value}' "
                f"(valid: {sorted(VALID_SUBTYPES)})"
            )
    elif prefix == "mechanic":
        if value not in VALID_MECHANICS:
            errors.append(
                f"{context}: unknown mechanic '{value}' "
                f"(valid: {sorted(VALID_MECHANICS)})"
            )
    elif prefix == "role":
        if value not in VALID_ROLES:
            errors.append(
                f"{context}: unknown role '{value}' "
                f"(valid: {sorted(VALID_ROLES)})"
            )
    else:
        errors.append(f"{context}: unknown tag prefix '{prefix}'")


def validate_dreamcallers() -> list[str]:
    """Validate dreamcallers.toml schema and invariants."""
    errors: list[str] = []
    path = DATA_DIR / "dreamcallers.toml"

    if not path.exists():
        return [f"File not found: {path}"]

    with open(path, "rb") as f:
        data = tomllib.load(f)

    if "dreamcallers" not in data:
        return ["Missing top-level 'dreamcallers' key"]

    entries = data["dreamcallers"]
    if not isinstance(entries, list):
        return ["'dreamcallers' must be an array of tables"]

    if len(entries) != 8:
        errors.append(f"Expected 8 dreamcallers, found {len(entries)}")

    required_fields = {
        "name": str,
        "resonance": list,
        "resonance_bonus": dict,
        "tags": list,
        "tag_bonus": dict,
        "essence_bonus": int,
        "ability_text": str,
    }

    names: set[str] = set()
    resonance_combos: set[frozenset[str]] = set()

    for i, entry in enumerate(entries):
        ctx = f"dreamcallers[{i}]"
        name = entry.get("name", f"<unnamed #{i}>")

        for field, expected_type in required_fields.items():
            if field not in entry:
                errors.append(f"{ctx} ({name}): missing required field '{field}'")
            elif not isinstance(entry[field], expected_type):
                errors.append(
                    f"{ctx} ({name}): field '{field}' should be "
                    f"{expected_type.__name__}, got {type(entry[field]).__name__}"
                )

        if name in names:
            errors.append(f"{ctx}: duplicate name '{name}'")
        names.add(name)

        resonance = entry.get("resonance", [])
        if not (1 <= len(resonance) <= 2):
            errors.append(
                f"{ctx} ({name}): resonance must have 1-2 entries, "
                f"got {len(resonance)}"
            )
        for r in resonance:
            if r not in VALID_RESONANCES:
                errors.append(
                    f"{ctx} ({name}): unknown resonance '{r}' "
                    f"(valid: {sorted(VALID_RESONANCES)})"
                )
        resonance_combos.add(frozenset(resonance))

        resonance_bonus = entry.get("resonance_bonus", {})
        for r, val in resonance_bonus.items():
            if r not in VALID_RESONANCES:
                errors.append(
                    f"{ctx} ({name}): resonance_bonus key '{r}' "
                    f"is not a valid resonance"
                )
            if not isinstance(val, int) or not (3 <= val <= 5):
                errors.append(
                    f"{ctx} ({name}): resonance_bonus['{r}'] = {val}, "
                    f"expected int in [3, 5]"
                )
        bonus_keys = set(resonance_bonus.keys())
        resonance_set = set(resonance)
        if bonus_keys != resonance_set:
            errors.append(
                f"{ctx} ({name}): resonance_bonus keys {bonus_keys} "
                f"don't match resonance {resonance_set}"
            )

        tags = entry.get("tags", [])
        if not (1 <= len(tags) <= 3):
            errors.append(
                f"{ctx} ({name}): tags must have 1-3 entries, got {len(tags)}"
            )
        for tag in tags:
            validate_tag(tag, errors, f"{ctx} ({name})")

        tag_bonus = entry.get("tag_bonus", {})
        for tag, val in tag_bonus.items():
            if tag not in tags:
                errors.append(
                    f"{ctx} ({name}): tag_bonus key '{tag}' not in tags list"
                )
            if not isinstance(val, int) or not (1 <= val <= 3):
                errors.append(
                    f"{ctx} ({name}): tag_bonus['{tag}'] = {val}, "
                    f"expected int in [1, 3]"
                )

        essence = entry.get("essence_bonus", 0)
        if not (0 <= essence <= 100):
            errors.append(
                f"{ctx} ({name}): essence_bonus = {essence}, "
                f"expected int in [0, 100]"
            )

    if len(resonance_combos) < 6:
        errors.append(
            f"Only {len(resonance_combos)} distinct resonance combinations, "
            f"expected at least 6"
        )

    has_mono = any(len(c) == 1 for c in resonance_combos)
    if not has_mono:
        errors.append("No mono-resonance dreamcaller found (at least 1 required)")

    essence_values = [e.get("essence_bonus", 0) for e in entries]
    if len(essence_values) >= 2:
        spread = max(essence_values) - min(essence_values)
        if spread < 50:
            errors.append(
                f"Essence bonus spread is {spread}, expected at least 50 "
                f"(values: {sorted(essence_values)})"
            )

    return errors


def validate_journeys() -> list[str]:
    """Validate journeys.toml schema and invariants."""
    errors: list[str] = []
    path = DATA_DIR / "journeys.toml"

    if not path.exists():
        return [f"File not found: {path}"]

    with open(path, "rb") as f:
        data = tomllib.load(f)

    if "journeys" not in data:
        return ["Missing top-level 'journeys' key"]

    entries = data["journeys"]
    if not isinstance(entries, list):
        return ["'journeys' must be an array of tables"]

    if not (10 <= len(entries) <= 20):
        errors.append(f"Expected 10-20 journeys, found {len(entries)}")

    required_fields = {
        "name": str,
        "description": str,
        "effect_type": str,
        "effect_value": int,
    }

    names: set[str] = set()
    effect_type_counts: dict[str, int] = {}

    for i, entry in enumerate(entries):
        ctx = f"journeys[{i}]"
        name = entry.get("name", f"<unnamed #{i}>")

        for field, expected_type in required_fields.items():
            if field not in entry:
                errors.append(f"{ctx} ({name}): missing required field '{field}'")
            elif not isinstance(entry[field], expected_type):
                errors.append(
                    f"{ctx} ({name}): field '{field}' should be "
                    f"{expected_type.__name__}, got {type(entry[field]).__name__}"
                )

        if name in names:
            errors.append(f"{ctx}: duplicate name '{name}'")
        names.add(name)

        effect_type = entry.get("effect_type", "")
        if effect_type not in VALID_EFFECT_TYPES:
            errors.append(
                f"{ctx} ({name}): unknown effect_type '{effect_type}' "
                f"(valid: {sorted(VALID_EFFECT_TYPES)})"
            )
        else:
            effect_type_counts[effect_type] = (
                effect_type_counts.get(effect_type, 0) + 1
            )

            effect_value = entry.get("effect_value", 0)
            lo, hi = EFFECT_VALUE_RANGES[effect_type]
            if not (lo <= effect_value <= hi):
                errors.append(
                    f"{ctx} ({name}): effect_value = {effect_value} "
                    f"for '{effect_type}', expected [{lo}, {hi}]"
                )

    for etype, min_count in MIN_EFFECT_TYPE_COUNTS.items():
        actual = effect_type_counts.get(etype, 0)
        if actual < min_count:
            errors.append(
                f"effect_type '{etype}': found {actual}, "
                f"expected at least {min_count}"
            )

    return errors


def validate_banes() -> list[str]:
    """Validate banes.toml schema and invariants."""
    errors: list[str] = []
    path = DATA_DIR / "banes.toml"

    if not path.exists():
        return [f"File not found: {path}"]

    with open(path, "rb") as f:
        data = tomllib.load(f)

    if "banes" not in data:
        return ["Missing top-level 'banes' key"]

    entries = data["banes"]
    if not isinstance(entries, list):
        return ["'banes' must be an array of tables"]

    if not (3 <= len(entries) <= 10):
        errors.append(f"Expected 3-10 banes, found {len(entries)}")

    required_fields = {
        "name": str,
        "rules_text": str,
        "card_type": str,
        "energy_cost": int,
    }

    names: set[str] = set()

    for i, entry in enumerate(entries):
        ctx = f"banes[{i}]"
        name = entry.get("name", f"<unnamed #{i}>")

        for field, expected_type in required_fields.items():
            if field not in entry:
                errors.append(f"{ctx} ({name}): missing required field '{field}'")
            elif not isinstance(entry[field], expected_type):
                errors.append(
                    f"{ctx} ({name}): field '{field}' should be "
                    f"{expected_type.__name__}, got {type(entry[field]).__name__}"
                )

        if name in names:
            errors.append(f"{ctx}: duplicate name '{name}'")
        names.add(name)

        card_type = entry.get("card_type", "")
        if card_type and card_type not in VALID_BANE_CARD_TYPES:
            errors.append(
                f"{ctx} ({name}): unknown card_type '{card_type}' "
                f"(valid: {sorted(VALID_BANE_CARD_TYPES)})"
            )

        energy_cost = entry.get("energy_cost", 0)
        if isinstance(energy_cost, int) and energy_cost < 0:
            errors.append(
                f"{ctx} ({name}): energy_cost = {energy_cost}, must be >= 0"
            )

        rules_text = entry.get("rules_text", "")
        if isinstance(rules_text, str) and not rules_text.strip():
            errors.append(f"{ctx} ({name}): rules_text must not be empty")

    return errors


def validate_bosses() -> list[str]:
    """Validate bosses.toml schema and invariants."""
    errors: list[str] = []
    path = DATA_DIR / "bosses.toml"

    if not path.exists():
        return [f"File not found: {path}"]

    with open(path, "rb") as f:
        data = tomllib.load(f)

    if "bosses" not in data:
        return ["Missing top-level 'bosses' key"]

    entries = data["bosses"]
    if not isinstance(entries, list):
        return ["'bosses' must be an array of tables"]

    if len(entries) != 20:
        errors.append(f"Expected 20 bosses, found {len(entries)}")

    required_fields = {
        "name": str,
        "archetype": str,
        "ability_text": str,
        "deck_description": str,
        "is_final": bool,
        "resonance": list,
    }

    names: set[str] = set()
    miniboss_count = 0
    final_boss_count = 0

    for i, entry in enumerate(entries):
        ctx = f"bosses[{i}]"
        name = entry.get("name", f"<unnamed #{i}>")

        for field, expected_type in required_fields.items():
            if field not in entry:
                errors.append(f"{ctx} ({name}): missing required field '{field}'")
            elif not isinstance(entry[field], expected_type):
                errors.append(
                    f"{ctx} ({name}): field '{field}' should be "
                    f"{expected_type.__name__}, got {type(entry[field]).__name__}"
                )

        if name in names:
            errors.append(f"{ctx}: duplicate name '{name}'")
        names.add(name)

        if entry.get("is_final"):
            final_boss_count += 1
        else:
            miniboss_count += 1

        resonance = entry.get("resonance", [])
        if not (1 <= len(resonance) <= 2):
            errors.append(
                f"{ctx} ({name}): resonance must have 1-2 entries, "
                f"got {len(resonance)}"
            )
        for r in resonance:
            if r not in VALID_RESONANCES:
                errors.append(
                    f"{ctx} ({name}): unknown resonance '{r}' "
                    f"(valid: {sorted(VALID_RESONANCES)})"
                )

        ability_text = entry.get("ability_text", "")
        if isinstance(ability_text, str) and not ability_text.strip():
            errors.append(f"{ctx} ({name}): ability_text must not be empty")

        deck_description = entry.get("deck_description", "")
        if isinstance(deck_description, str) and not deck_description.strip():
            errors.append(f"{ctx} ({name}): deck_description must not be empty")

    if miniboss_count != 10:
        errors.append(
            f"Expected 10 minibosses (is_final=false), found {miniboss_count}"
        )

    if final_boss_count != 10:
        errors.append(
            f"Expected 10 final bosses (is_final=true), found {final_boss_count}"
        )

    return errors


def main() -> int:
    """Run all validators and report results."""
    all_errors: list[str] = []

    print("Validating dreamcallers.toml...", end=" ")
    errors = validate_dreamcallers()
    if errors:
        print(f"FAILED ({len(errors)} errors)")
        all_errors.extend(errors)
    else:
        print("OK")

    print("Validating journeys.toml...", end=" ")
    errors = validate_journeys()
    if errors:
        print(f"FAILED ({len(errors)} errors)")
        all_errors.extend(errors)
    else:
        print("OK")

    print("Validating banes.toml...", end=" ")
    errors = validate_banes()
    if errors:
        print(f"FAILED ({len(errors)} errors)")
        all_errors.extend(errors)
    else:
        print("OK")

    print("Validating bosses.toml...", end=" ")
    errors = validate_bosses()
    if errors:
        print(f"FAILED ({len(errors)} errors)")
        all_errors.extend(errors)
    else:
        print("OK")

    if all_errors:
        print(f"\n{len(all_errors)} validation error(s):")
        for err in all_errors:
            print(f"  - {err}")
        return 1

    print("\nAll validations passed.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
