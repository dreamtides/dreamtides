"""Generate resonance and tag assignments for quest simulator cards.

Reads data/cards.json (produced by the Rust card exporter) and writes
data/card_data.json with resonance and tag assignments for each card.

Uses curated resonance assignments from data/card_allocations.toml as
the primary source, falling back to heuristic scoring for any cards
not found in the allocations file.

Usage: python3 resonance_heuristic.py
"""

import json
import re
import tomllib
from collections import Counter
from pathlib import Path

RESONANCES = ["Tide", "Ember", "Zephyr", "Stone", "Ruin"]

SUBTYPE_AFFINITY: dict[str, list[str]] = {
    "Spirit Animal": ["Stone"],
    "Warrior": ["Ember", "Stone"],
    "Ancient": ["Stone", "Ruin"],
    "Explorer": ["Zephyr", "Tide"],
    "Survivor": ["Ruin"],
    "Visitor": ["Zephyr"],
    "Synth": ["Tide", "Zephyr"],
    "Outsider": ["Ruin"],
    "Musician": ["Tide", "Ember"],
    "Mage": ["Tide"],
}

KEYWORD_RESONANCE: dict[str, str] = {
    "foresee": "Tide",
    "discover": "Tide",
    "draw": "Tide",
    "kindle": "Ember",
    "point": "Ember",
    "fast": "Zephyr",
    "prevent": "Zephyr",
    "copy": "Zephyr",
    "banish": "Zephyr",
    "spark": "Stone",
    "energy": "Stone",
    "dissolve": "Ruin",
    "reclaim": "Ruin",
    "discard": "Ruin",
    "void": "Ruin",
}

TRIBAL_SUBTYPES = frozenset(
    {
        "Survivor",
        "Warrior",
        "Spirit Animal",
        "Ancient",
        "Visitor",
        "Explorer",
        "Synth",
        "Outsider",
        "Musician",
        "Mage",
    }
)

REMOVAL_KEYWORDS = frozenset({"dissolve", "banish", "prevent"})
ENGINE_KEYWORDS = frozenset({"draw", "foresee", "discover"})

SUBTYPE_WEIGHT = 2.0
KEYWORD_WEIGHT = 1.0
COST_WEIGHT = 1.5
FAST_WEIGHT = 1.0

SINGLE_THRESHOLD = 1.0
DUAL_THRESHOLD = 2.5
CONFIDENCE_THRESHOLD = 2.0


def load_allocations(path: Path) -> dict[str, dict[str, list[str]]]:
    """Load curated resonance allocations from TOML file.

    Returns a dict mapping card name to {"resonance": [...], "archetypes": [...]}.
    """
    with open(path, "rb") as f:
        data = tomllib.load(f)

    result: dict[str, dict[str, list[str]]] = {}
    for entry in data.get("cards", []):
        name = entry["name"]
        result[name] = {
            "resonance": entry.get("resonance", []),
            "archetypes": entry.get("archetypes", []),
        }
    return result


def score_resonances(card: dict[str, object]) -> dict[str, float]:
    """Compute a resonance score for each of the 5 resonances."""
    scores: dict[str, float] = {r: 0.0 for r in RESONANCES}

    subtype = card.get("subtype")
    if isinstance(subtype, str) and subtype in SUBTYPE_AFFINITY:
        for resonance in SUBTYPE_AFFINITY[subtype]:
            scores[resonance] += SUBTYPE_WEIGHT

    rules_text = card.get("rules_text", "")
    if isinstance(rules_text, str):
        text_lower = rules_text.lower()
        for keyword, resonance in KEYWORD_RESONANCE.items():
            if re.search(r"\b" + keyword + r"\b", text_lower):
                scores[resonance] += KEYWORD_WEIGHT

    if card.get("is_fast"):
        scores["Zephyr"] += FAST_WEIGHT

    energy_cost = card.get("energy_cost")
    card_type = card.get("card_type")
    if isinstance(energy_cost, int) and card_type == "Character":
        if energy_cost <= 2:
            scores["Ember"] += COST_WEIGHT
        elif energy_cost >= 5:
            scores["Stone"] += COST_WEIGHT

    return scores


def assign_resonance(scores: dict[str, float]) -> list[str]:
    """Select 0-2 resonances from scores using thresholds.

    Uses a confidence check on total signal strength to avoid
    assigning resonance to cards with only weak, isolated signals.
    This helps maintain the target distribution of roughly 70%
    single, 10% dual, and 20% neutral.
    """
    ranked = sorted(scores.items(), key=lambda x: x[1], reverse=True)
    top_name, top_score = ranked[0]
    second_name, second_score = ranked[1]
    total_signal = sum(scores.values())

    if top_score < SINGLE_THRESHOLD or total_signal < CONFIDENCE_THRESHOLD:
        return []

    if second_score >= DUAL_THRESHOLD:
        return sorted([top_name, second_name])

    return [top_name]


def find_keywords(rules_text: str) -> set[str]:
    """Find all matching keywords in rules text."""
    text_lower = rules_text.lower()
    found: set[str] = set()
    for keyword in KEYWORD_RESONANCE:
        if re.search(r"\b" + keyword + r"\b", text_lower):
            found.add(keyword)
    return found


def assign_tags(
    card: dict[str, object],
    archetypes: list[str] | None = None,
) -> list[str]:
    """Assign tags to a card based on its attributes and archetypes."""
    tags: list[str] = []

    # Add archetype tags first (from curated allocations)
    if archetypes:
        for archetype in archetypes:
            tags.append(f"archetype:{archetype.lower()}")

    subtype = card.get("subtype")
    if isinstance(subtype, str) and subtype in TRIBAL_SUBTYPES:
        tag_value = subtype.lower().replace(" ", "-")
        tags.append(f"tribal:{tag_value}")

    rules_text = card.get("rules_text", "")
    keywords: set[str] = set()
    if isinstance(rules_text, str):
        keywords = find_keywords(rules_text)
        for keyword in sorted(keywords):
            if len(tags) < 6:
                tags.append(f"mechanic:{keyword}")

    spark = card.get("spark")
    if isinstance(spark, int) and spark >= 4 and len(tags) < 6:
        tags.append("role:finisher")

    if keywords & REMOVAL_KEYWORDS and len(tags) < 6:
        tags.append("role:removal")
    if keywords & ENGINE_KEYWORDS and len(tags) < 6:
        tags.append("role:engine")

    if not tags:
        card_type = card.get("card_type")
        if card_type == "Event":
            tags.append("mechanic:event")
        else:
            tags.append("mechanic:general")

    return tags


def run() -> None:
    """Read cards and allocations, assign resonances and tags, write output."""
    script_dir = Path(__file__).parent
    input_path = script_dir / "data" / "cards.json"
    output_path = script_dir / "data" / "card_data.json"
    allocations_path = script_dir / "data" / "card_allocations.toml"

    with open(input_path) as f:
        cards = json.load(f)

    allocations = load_allocations(allocations_path)

    results: list[dict[str, object]] = []
    curated_count = 0
    heuristic_count = 0

    for card in cards:
        card_name = card.get("name", "")
        allocation = allocations.get(card_name) if isinstance(card_name, str) else None

        if allocation is not None:
            resonance = list(allocation["resonance"])
            archetypes = list(allocation["archetypes"])
            curated_count += 1
        else:
            scores = score_resonances(card)
            resonance = assign_resonance(scores)
            archetypes = []
            heuristic_count += 1

        tags = assign_tags(card, archetypes)
        results.append(
            {
                "card_number": card["card_number"],
                "resonance": resonance,
                "tags": tags,
            }
        )

    with open(output_path, "w") as f:
        json.dump(results, f, indent=2)
        f.write("\n")

    print(f"Curated allocations: {curated_count}")
    print(f"Heuristic fallback:  {heuristic_count}")
    print_summary(results)


def _get_resonance_list(entry: dict[str, object]) -> list[object]:
    """Extract the resonance list from a result entry, defaulting to empty."""
    val = entry.get("resonance")
    if isinstance(val, list):
        return val
    return []


def print_summary(results: list[dict[str, object]]) -> None:
    """Print distribution summary to stdout."""
    total = len(results)
    single = sum(1 for r in results if len(_get_resonance_list(r)) == 1)
    dual = sum(1 for r in results if len(_get_resonance_list(r)) == 2)
    neutral = sum(1 for r in results if len(_get_resonance_list(r)) == 0)

    print(f"Total cards: {total}")
    print(f"Single resonance: {single} ({100 * single / total:.1f}%)")
    print(f"Dual resonance:   {dual} ({100 * dual / total:.1f}%)")
    print(f"Neutral:          {neutral} ({100 * neutral / total:.1f}%)")

    resonance_counts: Counter[str] = Counter()
    for r in results:
        res_list = r["resonance"]
        if isinstance(res_list, list):
            for res in res_list:
                if isinstance(res, str):
                    resonance_counts[res] += 1
    print("\nResonance distribution:")
    for res in RESONANCES:
        count = resonance_counts[res]
        print(f"  {res:8s}: {count}")

    tag_counts: Counter[str] = Counter()
    for r in results:
        tag_list = r["tags"]
        if isinstance(tag_list, list):
            for tag in tag_list:
                if isinstance(tag, str):
                    tag_counts[tag] += 1
    print(f"\nTag distribution ({len(tag_counts)} unique tags):")
    for tag, count in tag_counts.most_common(30):
        print(f"  {tag:30s}: {count}")


if __name__ == "__main__":
    run()
