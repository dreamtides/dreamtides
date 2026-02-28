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

    if all_errors:
        print(f"\n{len(all_errors)} validation error(s):")
        for err in all_errors:
            print(f"  - {err}")
        return 1

    print("\nAll validations passed.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
