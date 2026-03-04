"""Pack generation strategies for the draft simulator.

Implements three strategies for generating packs of cards from the cube:
Uniform, Rarity-weighted, and Seeded/Themed. Each strategy produces a
Pack with a unique pack_id and an archetype profile snapshot.
Stdlib-only, no external dependencies.
"""

import random
import uuid

import config
import cube_manager
from draft_models import CardInstance, Pack


def generate_pack(
    strategy: str,
    cube: cube_manager.CubeManager,
    cfg: config.SimulatorConfig,
    rng: random.Random,
    rarity_weights: dict[str, float] | None = None,
) -> Pack:
    """Generate a pack using the specified strategy.

    Dispatches to the appropriate strategy implementation based on
    the strategy name. Returns a Pack with a unique ID and archetype
    profile.
    """
    pack_size = cfg.draft.pack_size

    if strategy == "uniform":
        cards = _generate_uniform(cube, pack_size, rng)
    elif strategy == "rarity_weighted":
        cards = _generate_rarity_weighted(cube, pack_size, rng, rarity_weights or {})
    elif strategy == "seeded_themed":
        cards = _generate_seeded_themed(cube, pack_size, cfg, rng)
    else:
        raise ValueError(f"Unknown pack generation strategy: {strategy!r}")

    pack_id = uuid.UUID(int=rng.getrandbits(128)).hex[:12]
    archetype_profile = _compute_archetype_profile(cards, cfg.cards.archetype_count)

    return Pack(pack_id=pack_id, cards=cards, archetype_profile=archetype_profile)


def _generate_uniform(
    cube: cube_manager.CubeManager,
    pack_size: int,
    rng: random.Random,
) -> list[CardInstance]:
    """Sample pack_size cards with uniform probability."""
    return cube.draw(pack_size, rng)


def _generate_rarity_weighted(
    cube: cube_manager.CubeManager,
    pack_size: int,
    rng: random.Random,
    rarity_weights: dict[str, float],
) -> list[CardInstance]:
    """Sample pack_size cards with per-card rarity weights.

    Weights map from card_id to float; cards not in the map default
    to 1.0. When all weights are 1.0, equivalent to uniform.
    """
    if not rarity_weights:
        return cube.draw(pack_size, rng)

    weights = _build_rarity_weight_vector(cube, rarity_weights)
    return cube.draw(pack_size, rng, weights=weights)


def _generate_seeded_themed(
    cube: cube_manager.CubeManager,
    pack_size: int,
    cfg: config.SimulatorConfig,
    rng: random.Random,
) -> list[CardInstance]:
    """Generate a collated pack with controlled archetype density.

    Selects target archetypes, fills primary and bridge slots with
    archetype-aligned cards, then fills remaining slots uniformly.
    """
    archetype_count = cfg.cards.archetype_count
    target_count = min(cfg.pack_generation.archetype_target_count, archetype_count)
    primary_density = cfg.pack_generation.primary_density
    bridge_density = cfg.pack_generation.bridge_density
    variance = cfg.pack_generation.variance

    # Select target archetypes
    target_archetypes = _select_target_archetypes(
        cube, archetype_count, target_count, variance, rng
    )

    primary_slots = int(pack_size * primary_density)
    bridge_slots = int(pack_size * bridge_density)

    cards: list[CardInstance] = []
    used_ids: set[int] = set()

    # Fill primary slots: cards whose top fitness aligns with target archetypes
    primary_cards = _draw_primary_cards(
        cube, primary_slots, target_archetypes, used_ids, rng
    )
    cards.extend(primary_cards)
    used_ids.update(c.instance_id for c in primary_cards)

    # Fill bridge slots: cards with fitness > 0.5 in 2+ selected archetypes
    bridge_cards = _draw_bridge_cards(
        cube, bridge_slots, target_archetypes, used_ids, rng
    )
    cards.extend(bridge_cards)
    used_ids.update(c.instance_id for c in bridge_cards)

    # Fill remaining slots by uniform sampling
    remaining_needed = pack_size - len(cards)
    if remaining_needed > 0:
        fill_cards = _draw_remaining_cards(cube, remaining_needed, used_ids, rng)
        cards.extend(fill_cards)

    return cards


def _select_target_archetypes(
    cube: cube_manager.CubeManager,
    archetype_count: int,
    target_count: int,
    variance: float,
    rng: random.Random,
) -> list[int]:
    """Select target archetypes weighted by cube availability.

    Variance controls randomness: 0 = deterministic top-N by
    availability, 1 = fully random selection.
    """
    availability = _compute_archetype_availability(cube, archetype_count)

    if sum(availability) == 0:
        return rng.sample(range(archetype_count), target_count)

    # Blend between deterministic (sorted by availability) and random
    weights: list[float] = []
    for a in range(archetype_count):
        deterministic_w = availability[a]
        random_w = 1.0 / archetype_count
        blended = (1.0 - variance) * deterministic_w + variance * random_w
        weights.append(max(blended, 0.001))

    # Weighted sampling without replacement for target archetypes
    selected: list[int] = []
    indices = list(range(archetype_count))
    remaining_weights = list(weights)

    for _ in range(min(target_count, archetype_count)):
        total = sum(remaining_weights[i] for i in indices)
        if total <= 0:
            break
        r = rng.uniform(0, total)
        cumulative = 0.0
        chosen_pos = 0
        for pos, idx in enumerate(indices):
            cumulative += remaining_weights[idx]
            if cumulative >= r:
                chosen_pos = pos
                break
        chosen_idx = indices[chosen_pos]
        selected.append(chosen_idx)
        indices.pop(chosen_pos)

    return selected


def _compute_archetype_availability(
    cube: cube_manager.CubeManager,
    archetype_count: int,
) -> list[float]:
    """Compute per-archetype card availability from the cube supply."""
    availability = [0.0] * archetype_count
    for inst in cube._supply:
        top_arch = _top_archetype(inst, archetype_count)
        if top_arch >= 0:
            availability[top_arch] += 1.0
    total = sum(availability)
    if total > 0:
        availability = [a / total for a in availability]
    return availability


def _top_archetype(inst: CardInstance, archetype_count: int) -> int:
    """Return the index of the highest fitness value for a card instance."""
    if not inst.design.fitness:
        return -1
    best_idx = 0
    best_val = inst.design.fitness[0]
    for i in range(1, min(len(inst.design.fitness), archetype_count)):
        if inst.design.fitness[i] > best_val:
            best_val = inst.design.fitness[i]
            best_idx = i
    return best_idx


def _draw_primary_cards(
    cube: cube_manager.CubeManager,
    count: int,
    target_archetypes: list[int],
    used_ids: set[int],
    rng: random.Random,
) -> list[CardInstance]:
    """Draw cards whose top fitness aligns with primary archetypes."""
    target_set = set(target_archetypes)
    archetype_count = len(cube._supply[0].design.fitness) if cube._supply else 8

    weights: list[float] = []
    for inst in cube._supply:
        if inst.instance_id in used_ids:
            weights.append(0.0)
            continue
        top = _top_archetype(inst, archetype_count)
        if top in target_set:
            weights.append(max(inst.design.fitness[top], 0.001))
        else:
            weights.append(0.0)

    available_count = sum(1 for w in weights if w > 0)
    draw_count = min(count, available_count)
    if draw_count <= 0:
        return []

    return cube.draw(draw_count, rng, weights=weights)


def _draw_bridge_cards(
    cube: cube_manager.CubeManager,
    count: int,
    target_archetypes: list[int],
    used_ids: set[int],
    rng: random.Random,
) -> list[CardInstance]:
    """Draw bridge cards with fitness > 0.5 in 2+ selected archetypes."""
    target_set = set(target_archetypes)

    weights: list[float] = []
    for inst in cube._supply:
        if inst.instance_id in used_ids:
            weights.append(0.0)
            continue
        high_count = sum(
            1
            for a in target_set
            if a < len(inst.design.fitness) and inst.design.fitness[a] > 0.5
        )
        if high_count >= 2:
            weights.append(1.0)
        else:
            weights.append(0.0)

    available_count = sum(1 for w in weights if w > 0)
    draw_count = min(count, available_count)
    if draw_count <= 0:
        return []

    return cube.draw(draw_count, rng, weights=weights)


def _draw_remaining_cards(
    cube: cube_manager.CubeManager,
    count: int,
    used_ids: set[int],
    rng: random.Random,
) -> list[CardInstance]:
    """Draw remaining cards uniformly, avoiding already-used instances."""
    weights: list[float] = []
    for inst in cube._supply:
        if inst.instance_id in used_ids:
            weights.append(0.0)
        else:
            weights.append(1.0)

    available_count = sum(1 for w in weights if w > 0)
    draw_count = min(count, available_count)
    if draw_count <= 0:
        return []

    return cube.draw(draw_count, rng, weights=weights)


def _build_rarity_weight_vector(
    cube: cube_manager.CubeManager,
    rarity_weights: dict[str, float],
) -> list[float]:
    """Build a weight vector parallel to the cube supply."""
    weights: list[float] = []
    for inst in cube._supply:
        w = rarity_weights.get(inst.design.card_id, 1.0)
        weights.append(max(w, 0.001))
    return weights


def _compute_archetype_profile(
    cards: list[CardInstance],
    archetype_count: int,
) -> list[float]:
    """Compute mean fitness vector across all cards in a pack."""
    if not cards:
        return [0.0] * archetype_count

    profile = [0.0] * archetype_count
    for card in cards:
        for i in range(min(len(card.design.fitness), archetype_count)):
            profile[i] += card.design.fitness[i]

    n = len(cards)
    return [v / n for v in profile]
