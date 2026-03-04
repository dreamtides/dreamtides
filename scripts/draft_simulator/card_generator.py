"""Synthetic card generation and file-based card loading.

Generates card designs with archetype fitness vectors, power, commit,
and flex attributes. Supports both procedural generation from configured
distributions and loading from JSON files. Validates all card attributes.
Stdlib-only, no external dependencies.
"""

import json
import random
import sys

from config import SimulatorConfig
from draft_models import CardDesign


def generate_cards(cfg: SimulatorConfig, rng: random.Random) -> list[CardDesign]:
    """Generate or load card designs based on configuration."""
    if cfg.cards.source == "file":
        if cfg.cards.file_path is None:
            raise ValueError("cards.source is 'file' but cards.file_path is not set")
        cards = _load_cards_from_file(cfg.cards.file_path, cfg.cards.archetype_count)
    else:
        cards = _generate_synthetic_cards(cfg, rng)

    _validate_cards(cards, cfg.cards.archetype_count)
    return cards


def _load_cards_from_file(path: str, archetype_count: int) -> list[CardDesign]:
    """Load card designs from a JSON file."""
    with open(path, "r") as f:
        raw = json.load(f)

    cards: list[CardDesign] = []
    for entry in raw:
        cards.append(
            CardDesign(
                card_id=entry["card_id"],
                name=entry["name"],
                fitness=list(entry["fitness"]),
                power=float(entry["power"]),
                commit=float(entry["commit"]),
                flex=float(entry["flex"]),
            )
        )
    return cards


def _generate_synthetic_cards(
    cfg: SimulatorConfig, rng: random.Random
) -> list[CardDesign]:
    """Generate synthetic card designs satisfying distribution invariants.

    Targets per archetype: roughly `cards_per_archetype` cards with fitness
    above 0.5, and roughly 30 additional cards with fitness in [0.3, 0.5).
    Bridge cards (fraction `bridge_fraction` of total) have fitness above
    0.5 in two or more archetypes.
    """
    archetype_count = cfg.cards.archetype_count
    distinct_cards = cfg.cube.distinct_cards
    bridge_fraction = cfg.cards.bridge_fraction
    cards_per_archetype = cfg.cards.cards_per_archetype

    bridge_count = int(distinct_cards * bridge_fraction)
    primary_count = distinct_cards - bridge_count

    # Estimate bridge contributions per archetype to calibrate strong ratio
    avg_bridge_archs = 2.25
    bridge_per_arch = bridge_count * avg_bridge_archs / max(archetype_count, 1)
    primaries_per_arch = primary_count / max(archetype_count, 1)
    strong_target = max(0.0, cards_per_archetype - bridge_per_arch)
    strong_ratio = min(1.0, strong_target / max(primaries_per_arch, 1.0))

    cards: list[CardDesign] = []

    # Generate bridge cards (fitness > 0.5 in two or more archetypes)
    for i in range(bridge_count):
        fitness = _generate_bridge_fitness(archetype_count, rng)
        power = rng.uniform(0.2, 0.9)
        commit = _sample_beta(2.0, 3.0, rng)
        flex = _compute_flex(fitness)
        cards.append(
            CardDesign(
                card_id=f"bridge_{i:04d}",
                name=f"Bridge Card {i}",
                fitness=fitness,
                power=power,
                commit=commit,
                flex=flex,
            )
        )

    # Generate primary cards (high fitness in exactly one archetype)
    # Distribute evenly across archetypes to meet per-archetype targets
    primaries_per_archetype_int = primary_count // archetype_count
    remainder = primary_count % archetype_count

    card_index = 0
    for arch in range(archetype_count):
        count = primaries_per_archetype_int + (1 if arch < remainder else 0)
        for _ in range(count):
            fitness = _generate_primary_fitness(
                arch, archetype_count, strong_ratio, rng
            )
            power = rng.uniform(0.2, 0.9)
            commit = _sample_beta(2.0, 3.0, rng)
            flex = _compute_flex(fitness)
            cards.append(
                CardDesign(
                    card_id=f"primary_{arch}_{card_index:04d}",
                    name=f"Archetype {arch} Card {card_index}",
                    fitness=fitness,
                    power=power,
                    commit=commit,
                    flex=flex,
                )
            )
            card_index += 1

    return cards


def _generate_bridge_fitness(archetype_count: int, rng: random.Random) -> list[float]:
    """Generate a fitness vector with high fitness in 2-3 archetypes."""
    fitness = [0.0] * archetype_count
    bridge_arch_count = min(rng.choice([2, 2, 2, 3]), archetype_count)
    bridge_archetypes = rng.sample(range(archetype_count), bridge_arch_count)

    for arch in bridge_archetypes:
        fitness[arch] = rng.uniform(0.55, 0.85)

    # Fill remaining with low values
    for i in range(archetype_count):
        if i not in bridge_archetypes:
            fitness[i] = rng.uniform(0.0, 0.25)

    return fitness


def _generate_primary_fitness(
    primary_arch: int,
    archetype_count: int,
    strong_ratio: float,
    rng: random.Random,
) -> list[float]:
    """Generate a fitness vector with high fitness in one archetype.

    Produces a mix of strong primaries (fitness > 0.5) and moderate
    supporters (fitness in [0.3, 0.5)) to satisfy distribution
    invariants. Non-primary archetypes have a chance of receiving
    moderate fitness values to meet the ~30 moderate cards target.
    """
    fitness = [0.0] * archetype_count

    if rng.random() < strong_ratio:
        fitness[primary_arch] = rng.uniform(0.55, 0.95)
    else:
        fitness[primary_arch] = rng.uniform(0.30, 0.49)

    # Fill non-primary slots: ~10% chance of moderate [0.3, 0.5),
    # remainder low [0.0, 0.20]
    for i in range(archetype_count):
        if i != primary_arch:
            if rng.random() < 0.10:
                fitness[i] = rng.uniform(0.30, 0.49)
            else:
                fitness[i] = rng.uniform(0.0, 0.20)

    return fitness


def _sample_beta(alpha: float, beta: float, rng: random.Random) -> float:
    """Sample from a beta distribution, clamped to [0, 1]."""
    return max(0.0, min(1.0, rng.betavariate(alpha, beta)))


def _compute_flex(fitness: list[float]) -> float:
    """Compute flex as 1 - gini(fitness)."""
    return 1.0 - _gini(fitness)


def _gini(values: list[float]) -> float:
    """Compute the Gini coefficient of a list of non-negative values."""
    n = len(values)
    if n == 0:
        return 0.0

    total: float = float(sum(values))
    if total == 0.0:
        return 0.0

    sorted_values = sorted(values)
    cumulative = 0.0
    weighted_sum = 0.0
    for i, v in enumerate(sorted_values):
        cumulative += v
        weighted_sum += (2 * (i + 1) - n - 1) * v

    return weighted_sum / (n * total)


def _validate_cards(cards: list[CardDesign], archetype_count: int) -> None:
    """Validate all card attributes. Raises ValueError for hard errors."""
    for card in cards:
        # Fitness vector length
        if len(card.fitness) != archetype_count:
            raise ValueError(
                f"Card {card.card_id!r}: fitness vector length "
                f"{len(card.fitness)} != archetype_count {archetype_count}"
            )

        # Fitness values in [0, 1]
        for i, f in enumerate(card.fitness):
            if f < 0.0 or f > 1.0:
                raise ValueError(
                    f"Card {card.card_id!r}: fitness[{i}]={f} not in [0, 1]"
                )

        # Power in [0, 1]
        if card.power < 0.0 or card.power > 1.0:
            raise ValueError(f"Card {card.card_id!r}: power={card.power} not in [0, 1]")

        # Commit in [0, 1]
        if card.commit < 0.0 or card.commit > 1.0:
            raise ValueError(
                f"Card {card.card_id!r}: commit={card.commit} not in [0, 1]"
            )

        # Flex in [0, 1]
        if card.flex < 0.0 or card.flex > 1.0:
            raise ValueError(f"Card {card.card_id!r}: flex={card.flex} not in [0, 1]")

        # No all-zero fitness
        if all(f == 0.0 for f in card.fitness):
            raise ValueError(f"Card {card.card_id!r}: fitness vector is all zeros")

        # Warnings (not errors)
        if max(card.fitness) < 0.3:
            print(
                f"WARNING: Card {card.card_id!r} has max fitness "
                f"{max(card.fitness):.3f} < 0.3",
                file=sys.stderr,
            )

        if card.power < 0.1 and card.commit > 0.7:
            print(
                f"WARNING: Card {card.card_id!r} has low power "
                f"({card.power:.3f}) with high commit ({card.commit:.3f})",
                file=sys.stderr,
            )


def print_card_pool_stats(cards: list[CardDesign], archetype_count: int) -> None:
    """Print summary statistics for a card pool."""
    print(f"\nCard Pool Statistics:")
    print(f"  Total cards: {len(cards)}")

    # Per-archetype cards with fitness > 0.5
    print(f"\n  Cards with fitness > 0.5 per archetype:")
    for arch in range(archetype_count):
        count = sum(1 for c in cards if c.fitness[arch] > 0.5)
        print(f"    Archetype {arch}: {count}")

    # Bridge card count
    bridge_count = sum(1 for c in cards if sum(1 for f in c.fitness if f > 0.5) >= 2)
    print(f"\n  Bridge cards (fitness > 0.5 in 2+ archetypes): {bridge_count}")

    # Power distribution
    powers = [c.power for c in cards]
    print(
        f"\n  Power:  mean={_mean(powers):.3f}  "
        f"min={min(powers):.3f}  max={max(powers):.3f}"
    )

    # Commit distribution
    commits = [c.commit for c in cards]
    print(
        f"  Commit: mean={_mean(commits):.3f}  "
        f"min={min(commits):.3f}  max={max(commits):.3f}"
    )

    # Flex distribution
    flexes = [c.flex for c in cards]
    print(
        f"  Flex:   mean={_mean(flexes):.3f}  "
        f"min={min(flexes):.3f}  max={max(flexes):.3f}"
    )


def _mean(values: list[float]) -> float:
    """Compute the arithmetic mean of a list of floats."""
    if not values:
        return 0.0
    return sum(values) / len(values)
