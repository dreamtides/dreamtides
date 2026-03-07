"""Pack generation strategies for the draft simulator.

Implements three strategies for generating packs of cards from the cube:
Uniform, Rarity-weighted, and Seeded/Themed. Each strategy produces a
Pack with a unique pack_id and an archetype profile snapshot.
Stdlib-only, no external dependencies.
"""

import random
import uuid
from collections.abc import Callable

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
    elif strategy == "seeded_themed" and cfg.rarity.enabled:
        cards = _generate_rarity_seeded_themed(cube, pack_size, cfg, rng)
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
    used_names: set[str] = set()

    # Fill primary slots: cards whose top fitness aligns with target archetypes
    cards.extend(
        _draw_primary_cards(
            cube, primary_slots, target_archetypes, used_ids, used_names, rng
        )
    )

    # Fill bridge slots: cards with fitness > 0.5 in 2+ selected archetypes
    cards.extend(
        _draw_bridge_cards(
            cube, bridge_slots, target_archetypes, used_ids, used_names, rng
        )
    )

    # Fill remaining slots by uniform sampling
    remaining_needed = pack_size - len(cards)
    if remaining_needed > 0:
        cards.extend(
            _draw_remaining_cards(cube, remaining_needed, used_ids, used_names, rng)
        )

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

    # At variance=0, return the top-N archetypes deterministically
    if variance == 0.0:
        ranked = sorted(
            range(archetype_count), key=lambda a: availability[a], reverse=True
        )
        return ranked[:target_count]

    # Blend between deterministic (sorted by availability) and random
    weights: list[float] = []
    for a in range(archetype_count):
        deterministic_w = availability[a]
        random_w = 1.0 / archetype_count
        blended = (1.0 - variance) * deterministic_w + variance * random_w
        weights.append(max(blended, 0.001))

    # Weighted sampling without replacement for target archetypes
    selected = _weighted_select_without_replacement(
        list(range(archetype_count)), weights, target_count, rng
    )

    return selected


def _weighted_select_without_replacement(
    items: list[int],
    weights: list[float],
    n: int,
    rng: random.Random,
) -> list[int]:
    """Select n items via weighted sampling without replacement."""
    selected: list[int] = []
    indices = list(range(len(items)))
    remaining_weights = list(weights)

    for _ in range(min(n, len(items))):
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
        selected.append(items[chosen_idx])
        indices.pop(chosen_pos)

    return selected


def _compute_archetype_availability(
    cube: cube_manager.CubeManager,
    archetype_count: int,
) -> list[float]:
    """Compute per-archetype card availability from the cube supply."""
    availability = [0.0] * archetype_count
    for inst in cube.supply:
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


def _draw_deduped(
    cube: cube_manager.CubeManager,
    count: int,
    used_ids: set[int],
    used_names: set[str],
    weight_fn: Callable[[CardInstance], float],
    rng: random.Random,
) -> list[CardInstance]:
    """Draw cards one at a time, deduplicating by card name within a pack.

    After each draw, zero out weights for all instances sharing the
    drawn card's name so that no two copies of the same card design
    appear in one pack.
    """
    result: list[CardInstance] = []

    for _ in range(count):
        current_supply = cube.supply
        weights: list[float] = []
        for inst in current_supply:
            if inst.instance_id in used_ids or inst.design.name in used_names:
                weights.append(0.0)
            else:
                weights.append(weight_fn(inst))

        if not any(w > 0 for w in weights):
            break

        drawn = cube.draw(1, rng, weights=weights)
        if not drawn:
            break
        card = drawn[0]
        result.append(card)
        used_ids.add(card.instance_id)
        used_names.add(card.design.name)

    return result


def _draw_primary_cards(
    cube: cube_manager.CubeManager,
    count: int,
    target_archetypes: list[int],
    used_ids: set[int],
    used_names: set[str],
    rng: random.Random,
) -> list[CardInstance]:
    """Draw cards whose top fitness aligns with primary archetypes."""
    target_set = set(target_archetypes)
    archetype_count = len(cube.supply[0].design.fitness) if cube.supply else 8

    def weight_fn(inst: CardInstance) -> float:
        top = _top_archetype(inst, archetype_count)
        if top in target_set:
            return max(inst.design.fitness[top], 0.001)
        return 0.0

    return _draw_deduped(cube, count, used_ids, used_names, weight_fn, rng)


def _draw_bridge_cards(
    cube: cube_manager.CubeManager,
    count: int,
    target_archetypes: list[int],
    used_ids: set[int],
    used_names: set[str],
    rng: random.Random,
) -> list[CardInstance]:
    """Draw bridge cards with fitness > 0.5 in 2+ selected archetypes."""
    target_set = set(target_archetypes)

    def weight_fn(inst: CardInstance) -> float:
        high_count = sum(
            1
            for a in target_set
            if a < len(inst.design.fitness) and inst.design.fitness[a] > 0.5
        )
        return 1.0 if high_count >= 2 else 0.0

    return _draw_deduped(cube, count, used_ids, used_names, weight_fn, rng)


def _draw_remaining_cards(
    cube: cube_manager.CubeManager,
    count: int,
    used_ids: set[int],
    used_names: set[str],
    rng: random.Random,
) -> list[CardInstance]:
    """Draw remaining cards uniformly, avoiding already-used instances."""

    def weight_fn(inst: CardInstance) -> float:
        return 1.0

    return _draw_deduped(cube, count, used_ids, used_names, weight_fn, rng)


def _build_rarity_weight_vector(
    cube: cube_manager.CubeManager,
    rarity_weights: dict[str, float],
) -> list[float]:
    """Build a weight vector parallel to the cube supply."""
    current_supply = cube.supply
    weights: list[float] = []
    for inst in current_supply:
        w = rarity_weights.get(inst.design.card_id, 1.0)
        weights.append(max(w, 0.001))
    return weights


def _generate_rarity_seeded_themed(
    cube: cube_manager.CubeManager,
    pack_size: int,
    cfg: config.SimulatorConfig,
    rng: random.Random,
) -> list[CardInstance]:
    """Generate a collated pack with rarity-tier slot allocation.

    Selects target archetypes, then fills each rarity tier's allocated
    slots using primary/bridge/filler logic. Tracks both instance IDs
    and card names to prevent the same design appearing twice in a pack.
    """
    archetype_count = cfg.cards.archetype_count
    target_count = min(cfg.pack_generation.archetype_target_count, archetype_count)
    variance = cfg.pack_generation.variance
    primary_density = cfg.pack_generation.primary_density
    bridge_density = cfg.pack_generation.bridge_density
    rarity_cfg = cfg.rarity

    target_archetypes = _select_target_archetypes(
        cube, archetype_count, target_count, variance, rng
    )

    cards: list[CardInstance] = []
    used_instance_ids: set[int] = set()
    used_card_names: set[str] = set()

    for tier_idx, tier_name in enumerate(rarity_cfg.tiers):
        tier_slots = rarity_cfg.pack_tier_slots[tier_idx]
        primary_slots = int(tier_slots * primary_density)
        bridge_slots = int(tier_slots * bridge_density)

        # Fill primary slots for this tier
        cards.extend(
            _draw_primary_cards_for_tier(
                cube,
                primary_slots,
                target_archetypes,
                tier_name,
                used_instance_ids,
                used_card_names,
                rng,
            )
        )

        # Fill bridge slots for this tier
        cards.extend(
            _draw_bridge_cards_for_tier(
                cube,
                bridge_slots,
                target_archetypes,
                tier_name,
                used_instance_ids,
                used_card_names,
                rng,
            )
        )

        # Fill remaining tier slots
        remaining_needed = tier_slots - len(
            [c for c in cards if c.design.rarity == tier_name]
        )
        if remaining_needed > 0:
            cards.extend(
                _draw_remaining_cards_for_tier(
                    cube,
                    remaining_needed,
                    tier_name,
                    used_instance_ids,
                    used_card_names,
                    rng,
                )
            )

    # Any-tier backfill if still underfilled
    remaining_needed = pack_size - len(cards)
    if remaining_needed > 0:
        fill_cards = _draw_remaining_cards_deduped(
            cube, remaining_needed, used_instance_ids, used_card_names, rng
        )
        cards.extend(fill_cards)

    return cards


def _draw_primary_cards_for_tier(
    cube: cube_manager.CubeManager,
    count: int,
    target_archetypes: list[int],
    tier_name: str,
    used_instance_ids: set[int],
    used_card_names: set[str],
    rng: random.Random,
) -> list[CardInstance]:
    """Draw primary cards filtered by rarity tier with design-level dedup."""
    target_set = set(target_archetypes)
    archetype_count = len(cube.supply[0].design.fitness) if cube.supply else 8

    def weight_fn(inst: CardInstance) -> float:
        if inst.design.rarity != tier_name:
            return 0.0
        top = _top_archetype(inst, archetype_count)
        if top in target_set:
            return max(inst.design.fitness[top], 0.001)
        return 0.0

    return _draw_deduped(
        cube, count, used_instance_ids, used_card_names, weight_fn, rng
    )


def _draw_bridge_cards_for_tier(
    cube: cube_manager.CubeManager,
    count: int,
    target_archetypes: list[int],
    tier_name: str,
    used_instance_ids: set[int],
    used_card_names: set[str],
    rng: random.Random,
) -> list[CardInstance]:
    """Draw bridge cards filtered by rarity tier with design-level dedup."""
    target_set = set(target_archetypes)

    def weight_fn(inst: CardInstance) -> float:
        if inst.design.rarity != tier_name:
            return 0.0
        high_count = sum(
            1
            for a in target_set
            if a < len(inst.design.fitness) and inst.design.fitness[a] > 0.5
        )
        return 1.0 if high_count >= 2 else 0.0

    return _draw_deduped(
        cube, count, used_instance_ids, used_card_names, weight_fn, rng
    )


def _draw_remaining_cards_for_tier(
    cube: cube_manager.CubeManager,
    count: int,
    tier_name: str,
    used_instance_ids: set[int],
    used_card_names: set[str],
    rng: random.Random,
) -> list[CardInstance]:
    """Draw remaining cards from a specific tier with design-level dedup."""

    def weight_fn(inst: CardInstance) -> float:
        return 1.0 if inst.design.rarity == tier_name else 0.0

    return _draw_deduped(
        cube, count, used_instance_ids, used_card_names, weight_fn, rng
    )


def _draw_remaining_cards_deduped(
    cube: cube_manager.CubeManager,
    count: int,
    used_instance_ids: set[int],
    used_card_names: set[str],
    rng: random.Random,
) -> list[CardInstance]:
    """Draw remaining cards from any tier with design-level dedup."""

    def weight_fn(inst: CardInstance) -> float:
        return 1.0

    return _draw_deduped(
        cube, count, used_instance_ids, used_card_names, weight_fn, rng
    )


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
