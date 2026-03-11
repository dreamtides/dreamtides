"""Synthetic card generation and file-based card loading.

Generates card designs with archetype fitness vectors, power, commit,
and flex attributes. Supports both procedural generation from configured
distributions and loading from JSON files. Validates all card attributes.
Stdlib-only, no external dependencies.
"""

import json
import random
import sys
from typing import Optional

import colors
from config import SimulatorConfig
from draft_models import CardDesign


def generate_cards(cfg: SimulatorConfig, rng: random.Random) -> list[CardDesign]:
    """Generate or load card designs based on configuration."""
    if cfg.cards.source == "toml":
        if cfg.cards.rendered_toml_path is None:
            raise ValueError(
                "cards.source is 'toml' but cards.rendered_toml_path is not set"
            )
        if cfg.cards.metadata_toml_path is None:
            raise ValueError(
                "cards.source is 'toml' but cards.metadata_toml_path is not set"
            )
        real_cards = load_real_cards(
            cfg.cards.rendered_toml_path, cfg.cards.metadata_toml_path
        )
        if cfg.cards.real_only:
            cards = duplicate_real_cards(real_cards, cfg, rng)
        else:
            cards = fill_card_pool_gaps(real_cards, cfg, rng)
    elif cfg.cards.source == "file":
        if cfg.cards.file_path is None:
            raise ValueError("cards.source is 'file' but cards.file_path is not set")
        cards = _load_cards_from_file(cfg.cards.file_path, cfg.cards.archetype_count)
    elif cfg.rarity.enabled:
        cards = _generate_rarity_cards(cfg, rng)
    else:
        cards = _generate_synthetic_cards(cfg, rng)

    _validate_cards(cards, cfg.cards.archetype_count)
    return cards


def load_real_cards(
    rendered_toml_path: str, metadata_toml_path: str
) -> list[CardDesign]:
    """Load real card designs from rendered TOML and metadata TOML files."""
    import tomllib

    import pool_analyzer

    with open(rendered_toml_path, "rb") as f:
        rendered_data = tomllib.load(f)
    with open(metadata_toml_path, "rb") as f:
        metadata_data = tomllib.load(f)

    # Build metadata lookup by card-id
    metadata_by_id: dict[str, dict] = {}
    for entry in metadata_data.get("card-metadata", []):
        metadata_by_id[entry["card-id"]] = entry

    cards: list[CardDesign] = []
    for card in rendered_data.get("cards", []):
        raw_rarity = card.get("rarity", "")
        if raw_rarity == "Special":
            continue

        card_id = card.get("id", "")
        meta = metadata_by_id.get(card_id)
        if meta is None:
            continue

        # Map rarity
        if raw_rarity == "Legendary":
            rarity = "rare"
        else:
            rarity = raw_rarity.lower()

        # Build fitness vector from archetype names
        fitness = [float(meta.get(name, 0.0)) for name in pool_analyzer.ARCHETYPE_NAMES]

        # Handle spark/energy-cost which can be empty string or non-numeric
        raw_spark = card.get("spark", "")
        spark = _parse_optional_int(raw_spark)
        raw_energy = card.get("energy-cost", "")
        energy_cost = _parse_optional_int(raw_energy)

        cards.append(
            CardDesign(
                card_id=card_id,
                name=card.get("name", ""),
                fitness=fitness,
                power=float(meta.get("power", 0.0)),
                commit=float(meta.get("commit", 0.0)),
                flex=float(meta.get("flex", 0.0)),
                rarity=rarity,
                rules_text=card.get("rendered text", ""),
                energy_cost=energy_cost,
                card_type=card.get("card-type", ""),
                subtype=card.get("subtype", ""),
                spark=spark,
                is_fast=card.get("is-fast", False),
                is_real=True,
                image_number=_parse_optional_int(card.get("image-number", "")),
                resonance=tuple(card.get("resonance", [])),
            )
        )

    return cards


def fill_card_pool_gaps(
    real_cards: list[CardDesign],
    cfg: SimulatorConfig,
    rng: random.Random,
) -> list[CardDesign]:
    """Fill the gap between real cards and the target pool size with synthetics."""
    import pool_analyzer

    gap = cfg.cube.distinct_cards - len(real_cards)
    if gap <= 0:
        return real_cards

    # Analyze the real pool to find coverage gaps
    analysis = pool_analyzer.analyze_pool(real_cards, cfg)
    targets = pool_analyzer.compute_ideal_targets(cfg)
    gaps = pool_analyzer.compute_gaps(analysis, targets, cfg)

    # Determine per-rarity needs
    rarity_needs: dict[str, int] = {}
    if cfg.rarity.enabled and gaps.per_rarity_needed:
        total_rarity_need = sum(gaps.per_rarity_needed.values())
        if total_rarity_need > 0:
            for tier_name, need in gaps.per_rarity_needed.items():
                rarity_needs[tier_name] = round(need * gap / total_rarity_need)
            # Adjust rounding errors
            diff = gap - sum(rarity_needs.values())
            if diff != 0:
                # Add/subtract from the tier with the largest allocation
                largest_tier = max(rarity_needs, key=lambda t: rarity_needs[t])
                rarity_needs[largest_tier] += diff
        else:
            # Distribute proportionally to tier_design_counts
            for ti, tier_name in enumerate(cfg.rarity.tiers):
                rarity_needs[tier_name] = round(
                    cfg.rarity.tier_design_counts[ti] * gap / cfg.cube.distinct_cards
                )
            diff = gap - sum(rarity_needs.values())
            if diff != 0:
                largest_tier = max(rarity_needs, key=lambda t: rarity_needs[t])
                rarity_needs[largest_tier] += diff
    else:
        rarity_needs["common"] = gap

    # Generate synthetic cards per rarity tier
    archetype_count = cfg.cards.archetype_count
    bridge_fraction = cfg.cards.bridge_fraction
    coverage_needed = gaps.per_archetype_coverage_needed
    total_coverage_need = sum(coverage_needed)

    synthetic_cards: list[CardDesign] = []
    card_index = 0

    for tier_name, tier_count in rarity_needs.items():
        if tier_count <= 0:
            continue

        # Look up power range for this tier
        power_lo, power_hi = 0.2, 0.9
        if cfg.rarity.enabled:
            for ti, tn in enumerate(cfg.rarity.tiers):
                if tn == tier_name:
                    power_lo, power_hi = cfg.rarity.tier_power_ranges[ti]
                    break

        bridge_count = int(tier_count * bridge_fraction)
        primary_count = tier_count - bridge_count

        # Generate bridge cards
        for i in range(bridge_count):
            fitness = _generate_bridge_fitness(archetype_count, rng)
            power = rng.uniform(power_lo, power_hi)
            commit = _sample_beta(2.0, 3.0, rng)
            flex = _compute_flex(fitness)
            synthetic_cards.append(
                CardDesign(
                    card_id=f"synth_{tier_name}_bridge_{card_index:04d}",
                    name=f"Synthetic {tier_name.title()} #{card_index + 1:03d}",
                    fitness=fitness,
                    power=power,
                    commit=commit,
                    flex=flex,
                    rarity=tier_name,
                )
            )
            card_index += 1

        # Distribute primaries across archetypes weighted by coverage needs
        if total_coverage_need > 0:
            weights = [max(1, n) for n in coverage_needed]
        else:
            weights = [1] * archetype_count
        weight_sum = sum(weights)

        primaries_per_arch = [round(primary_count * w / weight_sum) for w in weights]
        # Fix rounding
        diff = primary_count - sum(primaries_per_arch)
        if diff != 0:
            best_arch = max(range(archetype_count), key=lambda a: weights[a])
            primaries_per_arch[best_arch] += diff

        for arch in range(archetype_count):
            for _ in range(primaries_per_arch[arch]):
                fitness = _generate_primary_fitness(arch, archetype_count, rng)
                power = rng.uniform(power_lo, power_hi)
                commit = _sample_beta(2.0, 3.0, rng)
                flex = _compute_flex(fitness)
                synthetic_cards.append(
                    CardDesign(
                        card_id=f"synth_{tier_name}_primary_{card_index:04d}",
                        name=f"Synthetic {tier_name.title()} #{card_index + 1:03d}",
                        fitness=fitness,
                        power=power,
                        commit=commit,
                        flex=flex,
                        rarity=tier_name,
                    )
                )
                card_index += 1

    return list(real_cards) + synthetic_cards


def duplicate_real_cards(
    real_cards: list[CardDesign],
    cfg: SimulatorConfig,
    rng: random.Random,
    max_copies: int = 2,
) -> list[CardDesign]:
    """Fill the gap to the target pool size by duplicating real cards.

    Uses the same pool analysis as fill_card_pool_gaps to determine
    per-rarity budgets, then samples real cards weighted by archetype
    coverage need. Each card can be duplicated at most max_copies times
    to keep the pool diverse.
    """
    import pool_analyzer

    gap = cfg.cube.distinct_cards - len(real_cards)
    if gap <= 0:
        return list(real_cards)

    # Analyze the real pool to find coverage gaps
    analysis = pool_analyzer.analyze_pool(real_cards, cfg)
    targets = pool_analyzer.compute_ideal_targets(cfg)
    gaps = pool_analyzer.compute_gaps(analysis, targets, cfg)

    # Determine per-rarity budgets (same logic as fill_card_pool_gaps)
    rarity_needs: dict[str, int] = {}
    if cfg.rarity.enabled and gaps.per_rarity_needed:
        total_rarity_need = sum(gaps.per_rarity_needed.values())
        if total_rarity_need > 0:
            for tier_name, need in gaps.per_rarity_needed.items():
                rarity_needs[tier_name] = round(need * gap / total_rarity_need)
            diff = gap - sum(rarity_needs.values())
            if diff != 0:
                largest_tier = max(rarity_needs, key=lambda t: rarity_needs[t])
                rarity_needs[largest_tier] += diff
        else:
            for ti, tier_name in enumerate(cfg.rarity.tiers):
                rarity_needs[tier_name] = round(
                    cfg.rarity.tier_design_counts[ti] * gap / cfg.cube.distinct_cards
                )
            diff = gap - sum(rarity_needs.values())
            if diff != 0:
                largest_tier = max(rarity_needs, key=lambda t: rarity_needs[t])
                rarity_needs[largest_tier] += diff
    else:
        rarity_needs["common"] = gap

    # Group real cards by rarity tier
    cards_by_rarity: dict[str, list[CardDesign]] = {}
    for card in real_cards:
        tier = card.rarity if card.rarity else "common"
        cards_by_rarity.setdefault(tier, []).append(card)

    coverage_needed = gaps.per_archetype_coverage_needed
    duplicates: list[CardDesign] = []
    counter = 0

    for tier_name, tier_count in rarity_needs.items():
        if tier_count <= 0:
            continue

        pool = cards_by_rarity.get(tier_name, [])
        if not pool:
            pool = real_cards

        base_weights = _compute_duplication_weights(pool, coverage_needed)
        dup_counts = [0] * len(pool)

        for _ in range(tier_count):
            # Zero out weights for cards that hit the cap
            weights = [
                base_weights[i] if dup_counts[i] < max_copies else 0.0
                for i in range(len(pool))
            ]
            # If all cards are capped, allow another round
            if sum(weights) == 0.0:
                weights = list(base_weights)
            idx = _weighted_choice_idx(pool, weights, rng)
            source = pool[idx]
            dup_counts[idx] += 1
            duplicates.append(
                CardDesign(
                    card_id=f"{source.card_id}_dup_{counter:04d}",
                    name=source.name,
                    fitness=list(source.fitness),
                    power=source.power,
                    commit=source.commit,
                    flex=source.flex,
                    rarity=source.rarity,
                    rules_text=source.rules_text,
                    energy_cost=source.energy_cost,
                    card_type=source.card_type,
                    subtype=source.subtype,
                    spark=source.spark,
                    is_fast=source.is_fast,
                    is_real=True,
                    image_number=source.image_number,
                    resonance=source.resonance,
                )
            )
            counter += 1

    return list(real_cards) + duplicates


def _compute_duplication_weights(
    pool: list[CardDesign], coverage_needed: list[int]
) -> list[float]:
    """Compute per-card weights for duplication sampling."""
    weights: list[float] = []
    for card in pool:
        w = sum(
            max(0, coverage_needed[i]) * card.fitness[i]
            for i in range(len(coverage_needed))
        )
        weights.append(max(0.1, w))
    return weights


def _weighted_choice_idx(
    pool: list[CardDesign], weights: list[float], rng: random.Random
) -> int:
    """Return the index of a card chosen by weighted sampling."""
    total = sum(weights)
    r = rng.random() * total
    cumulative = 0.0
    for i, w in enumerate(weights):
        cumulative += w
        if r <= cumulative:
            return i
    return len(pool) - 1


def _weighted_choice(
    pool: list[CardDesign], weights: list[float], rng: random.Random
) -> CardDesign:
    """Choose a card from pool weighted by the given weights."""
    return pool[_weighted_choice_idx(pool, weights, rng)]


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
                rarity=entry.get("rarity", ""),
            )
        )
    return cards


def _generate_synthetic_cards(
    cfg: SimulatorConfig, rng: random.Random
) -> list[CardDesign]:
    """Generate synthetic card designs.

    Bridge cards (fraction `bridge_fraction` of total) have fitness above
    0.5 in two or more archetypes. Primary cards have strong fitness in
    exactly one archetype.
    """
    archetype_count = cfg.cards.archetype_count
    distinct_cards = cfg.cube.distinct_cards
    bridge_fraction = cfg.cards.bridge_fraction

    bridge_count = int(distinct_cards * bridge_fraction)
    primary_count = distinct_cards - bridge_count

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
            fitness = _generate_primary_fitness(arch, archetype_count, rng)
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


def _generate_rarity_cards(
    cfg: SimulatorConfig, rng: random.Random
) -> list[CardDesign]:
    """Generate synthetic cards with rarity tiers.

    For each tier, splits into bridge and primary cards, samples power
    from the tier's configured range, and tags each card with its rarity.
    """
    archetype_count = cfg.cards.archetype_count
    bridge_fraction = cfg.cards.bridge_fraction
    rarity_cfg = cfg.rarity

    cards: list[CardDesign] = []

    for tier_idx, tier_name in enumerate(rarity_cfg.tiers):
        tier_count = rarity_cfg.tier_design_counts[tier_idx]
        power_low, power_high = rarity_cfg.tier_power_ranges[tier_idx]

        bridge_count = int(tier_count * bridge_fraction)
        primary_count = tier_count - bridge_count

        # Generate bridge cards for this tier
        for i in range(bridge_count):
            fitness = _generate_bridge_fitness(archetype_count, rng)
            power = rng.uniform(power_low, power_high)
            commit = _sample_beta(2.0, 3.0, rng)
            flex = _compute_flex(fitness)
            cards.append(
                CardDesign(
                    card_id=f"{tier_name}_bridge_{i:04d}",
                    name=f"{tier_name.title()} Bridge {i}",
                    fitness=fitness,
                    power=power,
                    commit=commit,
                    flex=flex,
                    rarity=tier_name,
                )
            )

        # Generate primary cards distributed across archetypes
        primaries_per_arch = primary_count // archetype_count
        remainder = primary_count % archetype_count
        card_index = 0
        for arch in range(archetype_count):
            count = primaries_per_arch + (1 if arch < remainder else 0)
            for _ in range(count):
                fitness = _generate_primary_fitness(arch, archetype_count, rng)
                power = rng.uniform(power_low, power_high)
                commit = _sample_beta(2.0, 3.0, rng)
                flex = _compute_flex(fitness)
                cards.append(
                    CardDesign(
                        card_id=f"{tier_name}_primary_{arch}_{card_index:04d}",
                        name=f"{tier_name.title()} Arch {arch} Card {card_index}",
                        fitness=fitness,
                        power=power,
                        commit=commit,
                        flex=flex,
                        rarity=tier_name,
                    )
                )
                card_index += 1

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
    rng: random.Random,
) -> list[float]:
    """Generate a fitness vector with strong fitness in one archetype."""
    fitness = [0.0] * archetype_count
    fitness[primary_arch] = rng.uniform(0.55, 0.95)

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

        # No all-zero fitness (skip for real cards which may be flex/neutral)
        if all(f == 0.0 for f in card.fitness) and not card.is_real:
            raise ValueError(f"Card {card.card_id!r}: fitness vector is all zeros")

        # Warnings (not errors)
        if card.power < 0.1 and card.commit > 0.7:
            print(
                f"WARNING: Card {card.card_id!r} has low power "
                f"({card.power:.3f}) with high commit ({card.commit:.3f})",
                file=sys.stderr,
            )


def print_card_pool_stats(cards: list[CardDesign], archetype_count: int) -> None:
    """Print summary statistics for a card pool."""
    print(f"\n{colors.section('Card Pool Statistics:')}")
    print(f"  {colors.label('Total cards:')} {colors.num(len(cards))}")

    # Per-archetype cards with fitness > 0.5
    print(f"\n  {colors.label('Cards with fitness > 0.5 per archetype:')}")
    for arch in range(archetype_count):
        count = sum(1 for c in cards if c.fitness[arch] > 0.5)
        print(
            f"    {colors.label('Archetype')} {colors.c(arch, 'operator')}: "
            f"{colors.num(count)}"
        )

    # Bridge card count
    bridge_count = sum(1 for c in cards if sum(1 for f in c.fitness if f > 0.5) >= 2)
    print(
        f"\n  {colors.label('Bridge cards (fitness > 0.5 in 2+ archetypes):')} "
        f"{colors.num(bridge_count)}"
    )

    # Power distribution
    powers = [c.power for c in cards]
    print(
        f"\n  {colors.label('Power:')}  "
        f"{colors.dim('mean=')}{colors.num(f'{_mean(powers):.3f}')}  "
        f"{colors.dim('min=')}{colors.num(f'{min(powers):.3f}')}  "
        f"{colors.dim('max=')}{colors.num(f'{max(powers):.3f}')}"
    )

    # Commit distribution
    commits = [c.commit for c in cards]
    print(
        f"  {colors.label('Commit:')} "
        f"{colors.dim('mean=')}{colors.num(f'{_mean(commits):.3f}')}  "
        f"{colors.dim('min=')}{colors.num(f'{min(commits):.3f}')}  "
        f"{colors.dim('max=')}{colors.num(f'{max(commits):.3f}')}"
    )

    # Flex distribution
    flexes = [c.flex for c in cards]
    print(
        f"  {colors.label('Flex:')}   "
        f"{colors.dim('mean=')}{colors.num(f'{_mean(flexes):.3f}')}  "
        f"{colors.dim('min=')}{colors.num(f'{min(flexes):.3f}')}  "
        f"{colors.dim('max=')}{colors.num(f'{max(flexes):.3f}')}"
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
    lines.append(colors.c("=" * 60, "ui"))
    lines.append(colors.header("CARD POOL SUMMARY"))
    lines.append(colors.c("=" * 60, "ui"))
    lines.append(
        f"{colors.label('Source:')} {colors.c(source, 'special')}  |  "
        f"{colors.num(len(cards))} distinct designs  |  "
        f"{colors.num(archetype_count)} archetypes"
    )
    lines.append("")

    # --- Rarity Distribution (if present) ---
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

    # --- Composition ---
    bridge_cards = [c for c in cards if sum(1 for f in c.fitness if f > 0.5) >= 2]
    primary_cards = [c for c in cards if sum(1 for f in c.fitness if f > 0.5) < 2]
    lines.append(colors.section("--- Composition ---"))
    lines.append(
        f"  {colors.label('Primary cards (single archetype):')} "
        f"{colors.num(f'{len(primary_cards):>4d}')}  "
        f"({colors.num(f'{100*len(primary_cards)/len(cards):.1f}')}%)"
    )
    lines.append(
        f"  {colors.label('Bridge cards  (2+ archetypes):')}     "
        f"{colors.num(f'{len(bridge_cards):>4d}')}  "
        f"({colors.num(f'{100*len(bridge_cards)/len(cards):.1f}')}%)"
    )
    lines.append("")

    # --- Per-Archetype Coverage ---
    lines.append(colors.section("--- Per-Archetype Coverage ---"))
    cov_header = (
        f"  {colors.label(f'{'Arch':>4s}')}  "
        f"{colors.label(f'{'Strong(>0.5)':>12s}')}  "
        f"{colors.label(f'{'Moderate':>12s}')}  "
        f"{colors.label(f'{'Weak(<0.3)':>10s}')}"
    )
    lines.append(cov_header)
    for arch in range(archetype_count):
        strong = sum(1 for c in cards if c.fitness[arch] > 0.5)
        moderate = sum(1 for c in cards if 0.3 <= c.fitness[arch] <= 0.5)
        weak = sum(1 for c in cards if c.fitness[arch] < 0.3)
        lines.append(
            f"  {colors.c(f'{arch:>4d}', 'operator')}  "
            f"{colors.num(f'{strong:>12d}')}  "
            f"{colors.num(f'{moderate:>12d}')}  "
            f"{colors.num(f'{weak:>10d}')}"
        )
    lines.append("")

    # --- Fitness Distribution ---
    all_fitness = [f for c in cards for f in c.fitness]
    top_fitness = [max(c.fitness) for c in cards]
    lines.append(colors.section("--- Fitness Distribution ---"))
    lines.append(
        f"  {colors.label('All values:')}     "
        f"{colors.dim('mean=')}{colors.num(f'{_mean(all_fitness):.3f}')}  "
        f"{colors.dim('median=')}{colors.num(f'{_percentile(all_fitness, 50):.3f}')}  "
        f"{colors.dim('std=')}{colors.num(f'{_std(all_fitness):.3f}')}"
    )
    lines.append(
        f"  {colors.label('Top per card:')}   "
        f"{colors.dim('mean=')}{colors.num(f'{_mean(top_fitness):.3f}')}  "
        f"{colors.dim('median=')}{colors.num(f'{_percentile(top_fitness, 50):.3f}')}  "
        f"{colors.dim('std=')}{colors.num(f'{_std(top_fitness):.3f}')}"
    )
    lines.append("")

    # --- Power Distribution ---
    powers = [c.power for c in cards]
    lines.append(colors.section("--- Power Distribution ---"))
    lines.append(
        f"  {colors.dim('min=')}{colors.num(f'{min(powers):.3f}')}  "
        f"{colors.dim('p25=')}{colors.num(f'{_percentile(powers, 25):.3f}')}  "
        f"{colors.dim('median=')}{colors.num(f'{_percentile(powers, 50):.3f}')}  "
        f"{colors.dim('p75=')}{colors.num(f'{_percentile(powers, 75):.3f}')}  "
        f"{colors.dim('max=')}{colors.num(f'{max(powers):.3f}')}"
    )
    lines.append(_histogram(powers, "  ", bins=6, lo=0.0, hi=1.0))
    lines.append("")

    # --- Commit Distribution ---
    commits = [c.commit for c in cards]
    lines.append(colors.section("--- Commit Distribution ---"))
    lines.append(
        f"  {colors.dim('min=')}{colors.num(f'{min(commits):.3f}')}  "
        f"{colors.dim('p25=')}{colors.num(f'{_percentile(commits, 25):.3f}')}  "
        f"{colors.dim('median=')}{colors.num(f'{_percentile(commits, 50):.3f}')}  "
        f"{colors.dim('p75=')}{colors.num(f'{_percentile(commits, 75):.3f}')}  "
        f"{colors.dim('max=')}{colors.num(f'{max(commits):.3f}')}"
    )
    lines.append(_histogram(commits, "  ", bins=6, lo=0.0, hi=1.0))
    lines.append("")

    # --- Flex Distribution ---
    flexes = [c.flex for c in cards]
    lines.append(colors.section("--- Flex Distribution ---"))
    lines.append(
        f"  {colors.dim('min=')}{colors.num(f'{min(flexes):.3f}')}  "
        f"{colors.dim('p25=')}{colors.num(f'{_percentile(flexes, 25):.3f}')}  "
        f"{colors.dim('median=')}{colors.num(f'{_percentile(flexes, 50):.3f}')}  "
        f"{colors.dim('p75=')}{colors.num(f'{_percentile(flexes, 75):.3f}')}  "
        f"{colors.dim('max=')}{colors.num(f'{max(flexes):.3f}')}"
    )
    lines.append(_histogram(flexes, "  ", bins=6, lo=0.0, hi=1.0))
    lines.append("")

    # --- Cross-Archetype Diversity ---
    lines.append(colors.section("--- Cross-Archetype Diversity ---"))
    diversity_buckets: dict[int, int] = {}
    for c in cards:
        n_strong = sum(1 for f in c.fitness if f > 0.5)
        diversity_buckets[n_strong] = diversity_buckets.get(n_strong, 0) + 1
    for n_strong in sorted(diversity_buckets.keys()):
        count = diversity_buckets[n_strong]
        bucket_label = (
            f"{n_strong} archetype" if n_strong == 1 else f"{n_strong} archetypes"
        )
        if n_strong == 0:
            bucket_label = "no archetype"
        lines.append(
            f"  {colors.label(f'Fitness > 0.5 in {bucket_label + ":":<16s}')} "
            f"{colors.num(f'{count:>4d}')}  "
            f"({colors.num(f'{100*count/len(cards):.1f}')}%)"
        )
    lines.append("")

    # --- Strongest Cards ---
    sorted_by_power = sorted(cards, key=lambda c: c.power, reverse=True)
    lines.append(colors.section("--- Strongest Cards (top 5 by power) ---"))
    lines.append(
        f"  {colors.label(f'{'Name':<30s}')}  "
        f"{colors.label(f'{'Power':>5s}')}  "
        f"{colors.label(f'{'TopArch':>7s}')}  "
        f"{colors.label(f'{'Fitness':>7s}')}"
    )
    for c in sorted_by_power[:5]:
        top_arch = max(range(len(c.fitness)), key=lambda i: c.fitness[i])
        lines.append(
            f"  {colors.card(f'{c.name:<30s}')}  "
            f"{colors.num(f'{c.power:>5.3f}')}  "
            f"{colors.c(f'{top_arch:>7d}', 'operator')}  "
            f"{colors.num(f'{c.fitness[top_arch]:>7.3f}')}"
        )

    lines.append(colors.c("=" * 60, "ui"))
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
        lines.append(
            f"{indent}[{edge:.2f}-{edge + width:.2f}] "
            f"{colors.c(bar, 'accent')} {colors.num(count)}"
        )
    return "\n".join(lines)
