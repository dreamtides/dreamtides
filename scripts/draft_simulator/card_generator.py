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


def describe_card_pool(
    cards: list[CardDesign], archetype_count: int, source: str = "synthetic"
) -> str:
    """Return a 30-60 line summary describing the card pool.

    Covers composition, per-archetype coverage, fitness/power/commit/flex
    distributions with histograms and percentiles, cross-archetype diversity,
    and the strongest cards.
    """
    lines: list[str] = []

    # --- Header ---
    lines.append("=" * 60)
    lines.append("CARD POOL SUMMARY")
    lines.append("=" * 60)
    lines.append(
        f"Source: {source}  |  {len(cards)} distinct designs  |  "
        f"{archetype_count} archetypes"
    )
    lines.append("")

    # --- Composition ---
    bridge_cards = [c for c in cards if sum(1 for f in c.fitness if f > 0.5) >= 2]
    primary_cards = [c for c in cards if sum(1 for f in c.fitness if f > 0.5) < 2]
    lines.append("--- Composition ---")
    lines.append(
        f"  Primary cards (single archetype): "
        f"{len(primary_cards):>4d}  ({100*len(primary_cards)/len(cards):.1f}%)"
    )
    lines.append(
        f"  Bridge cards  (2+ archetypes):     "
        f"{len(bridge_cards):>4d}  ({100*len(bridge_cards)/len(cards):.1f}%)"
    )
    lines.append("")

    # --- Per-Archetype Coverage ---
    lines.append("--- Per-Archetype Coverage ---")
    header = (
        f"  {'Arch':>4s}  {'Strong(>0.5)':>12s}  {'Moderate':>12s}  {'Weak(<0.3)':>10s}"
    )
    lines.append(header)
    for arch in range(archetype_count):
        strong = sum(1 for c in cards if c.fitness[arch] > 0.5)
        moderate = sum(1 for c in cards if 0.3 <= c.fitness[arch] <= 0.5)
        weak = sum(1 for c in cards if c.fitness[arch] < 0.3)
        lines.append(f"  {arch:>4d}  {strong:>12d}  {moderate:>12d}  {weak:>10d}")
    lines.append("")

    # --- Fitness Distribution ---
    all_fitness = [f for c in cards for f in c.fitness]
    top_fitness = [max(c.fitness) for c in cards]
    lines.append("--- Fitness Distribution ---")
    lines.append(
        f"  All values:     mean={_mean(all_fitness):.3f}  "
        f"median={_percentile(all_fitness, 50):.3f}  "
        f"std={_std(all_fitness):.3f}"
    )
    lines.append(
        f"  Top per card:   mean={_mean(top_fitness):.3f}  "
        f"median={_percentile(top_fitness, 50):.3f}  "
        f"std={_std(top_fitness):.3f}"
    )
    lines.append("")

    # --- Power Distribution ---
    powers = [c.power for c in cards]
    lines.append("--- Power Distribution ---")
    lines.append(
        f"  min={min(powers):.3f}  p25={_percentile(powers, 25):.3f}  "
        f"median={_percentile(powers, 50):.3f}  "
        f"p75={_percentile(powers, 75):.3f}  max={max(powers):.3f}"
    )
    lines.append(_histogram(powers, "  ", bins=6, lo=0.0, hi=1.0))
    lines.append("")

    # --- Commit Distribution ---
    commits = [c.commit for c in cards]
    lines.append("--- Commit Distribution ---")
    lines.append(
        f"  min={min(commits):.3f}  p25={_percentile(commits, 25):.3f}  "
        f"median={_percentile(commits, 50):.3f}  "
        f"p75={_percentile(commits, 75):.3f}  max={max(commits):.3f}"
    )
    lines.append(_histogram(commits, "  ", bins=6, lo=0.0, hi=1.0))
    lines.append("")

    # --- Flex Distribution ---
    flexes = [c.flex for c in cards]
    lines.append("--- Flex Distribution ---")
    lines.append(
        f"  min={min(flexes):.3f}  p25={_percentile(flexes, 25):.3f}  "
        f"median={_percentile(flexes, 50):.3f}  "
        f"p75={_percentile(flexes, 75):.3f}  max={max(flexes):.3f}"
    )
    lines.append(_histogram(flexes, "  ", bins=6, lo=0.0, hi=1.0))
    lines.append("")

    # --- Cross-Archetype Diversity ---
    lines.append("--- Cross-Archetype Diversity ---")
    diversity_buckets: dict[int, int] = {}
    for c in cards:
        n_strong = sum(1 for f in c.fitness if f > 0.5)
        diversity_buckets[n_strong] = diversity_buckets.get(n_strong, 0) + 1
    for n_strong in sorted(diversity_buckets.keys()):
        count = diversity_buckets[n_strong]
        label = f"{n_strong} archetype" if n_strong == 1 else f"{n_strong} archetypes"
        if n_strong == 0:
            label = "no archetype"
        lines.append(
            f"  Fitness > 0.5 in {label + ':':<16s} "
            f"{count:>4d}  ({100*count/len(cards):.1f}%)"
        )
    lines.append("")

    # --- Strongest Cards ---
    sorted_by_power = sorted(cards, key=lambda c: c.power, reverse=True)
    lines.append("--- Strongest Cards (top 5 by power) ---")
    lines.append(f"  {'Name':<30s}  {'Power':>5s}  {'TopArch':>7s}  {'Fitness':>7s}")
    for c in sorted_by_power[:5]:
        top_arch = max(range(len(c.fitness)), key=lambda i: c.fitness[i])
        lines.append(
            f"  {c.name:<30s}  {c.power:>5.3f}  {top_arch:>7d}  "
            f"{c.fitness[top_arch]:>7.3f}"
        )

    lines.append("=" * 60)
    return "\n".join(lines)


def _mean(values: list[float]) -> float:
    """Compute the arithmetic mean of a list of floats."""
    if not values:
        return 0.0
    return sum(values) / len(values)


def _percentile(values: list[float], pct: float) -> float:
    """Compute a percentile using linear interpolation."""
    if not values:
        return 0.0
    s = sorted(values)
    k = (pct / 100.0) * (len(s) - 1)
    lo = int(k)
    hi = min(lo + 1, len(s) - 1)
    frac = k - lo
    return s[lo] + frac * (s[hi] - s[lo])


def _std(values: list[float]) -> float:
    """Compute population standard deviation."""
    if len(values) < 2:
        return 0.0
    m = _mean(values)
    return (sum((v - m) ** 2 for v in values) / len(values)) ** 0.5


def _histogram(
    values: list[float], indent: str, bins: int = 5, lo: float = 0.0, hi: float = 1.0
) -> str:
    """Render a multi-line histogram with bin counts and bars."""
    width = (hi - lo) / bins
    counts = [0] * bins
    for v in values:
        idx = min(int((v - lo) / width), bins - 1)
        idx = max(0, idx)
        counts[idx] += 1
    max_count = max(counts) if counts else 1
    lines: list[str] = []
    for i, count in enumerate(counts):
        if count == 0:
            continue
        edge = lo + i * width
        bar_len = max(1, round(count / max_count * 20))
        bar = "\u2588" * bar_len
        lines.append(f"{indent}[{edge:.2f}-{edge + width:.2f}] {bar} {count}")
    return "\n".join(lines)
