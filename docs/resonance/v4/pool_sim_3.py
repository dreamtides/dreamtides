#!/usr/bin/env python3
"""
Agent 3: Archetype Breakdown Simulation for Pack Widening v3.

Investigates: How should cards be distributed across archetypes, and how many
generic/bridge cards should exist?

Tests 5 breakdown models:
  1. Equal cards per archetype + small generic pool (~10%)
  2. Equal + large generic pool (~25%)
  3. Equal + explicit bridge card category
  4. Asymmetric archetype sizes
  5. Mono-symbol only (all archetype cards have just [Primary])

Uses Pack Widening v3 draft algorithm throughout.
"""

import random
import statistics
from dataclasses import dataclass, field
from enum import Enum
from collections import defaultdict
from typing import Optional

# ---------------------------------------------------------------------------
# Core types
# ---------------------------------------------------------------------------

class Resonance(Enum):
    EMBER = "Ember"
    STONE = "Stone"
    TIDE = "Tide"
    ZEPHYR = "Zephyr"

class Tier(Enum):
    S = "S"
    A = "A"
    B = "B"
    C = "C"
    F = "F"

# The 8 archetypes on a circle: (name, primary_resonance, secondary_resonance)
ARCHETYPES = [
    ("Flash",        Resonance.ZEPHYR, Resonance.EMBER),   # 0
    ("Blink",        Resonance.EMBER,  Resonance.ZEPHYR),  # 1
    ("Storm",        Resonance.EMBER,  Resonance.STONE),   # 2
    ("Self-Discard", Resonance.STONE,  Resonance.EMBER),   # 3
    ("Self-Mill",    Resonance.STONE,  Resonance.TIDE),    # 4
    ("Sacrifice",    Resonance.TIDE,   Resonance.STONE),   # 5
    ("Warriors",     Resonance.TIDE,   Resonance.ZEPHYR),  # 6
    ("Ramp",         Resonance.ZEPHYR, Resonance.TIDE),    # 7
]

ARCHETYPE_NAMES = [a[0] for a in ARCHETYPES]
NUM_ARCHETYPES = 8

# Adjacency: archetypes at distance 1 on the circle
def circle_distance(i: int, j: int) -> int:
    d = abs(i - j)
    return min(d, NUM_ARCHETYPES - d)

def get_adjacent_indices(arch_idx: int) -> list[int]:
    """Return indices of the two adjacent archetypes on the circle."""
    return [(arch_idx - 1) % NUM_ARCHETYPES, (arch_idx + 1) % NUM_ARCHETYPES]

def compute_fitness(card_arch_idx: int, player_arch_idx: int) -> Tier:
    """Compute fitness tier of a card for a specific player archetype.

    S-tier: card's home archetype
    A-tier: adjacent archetype sharing card's primary resonance
    B-tier: archetypes sharing card's secondary resonance; also generic
    C/F-tier: distant archetypes
    """
    if card_arch_idx < 0:
        # Generic card
        return Tier.B

    if card_arch_idx == player_arch_idx:
        return Tier.S

    dist = circle_distance(card_arch_idx, player_arch_idx)

    card_primary = ARCHETYPES[card_arch_idx][1]
    player_primary = ARCHETYPES[player_arch_idx][1]
    card_secondary = ARCHETYPES[card_arch_idx][2]
    player_secondary = ARCHETYPES[player_arch_idx][2]

    if dist == 1:
        # Adjacent: A-tier if they share primary resonance
        if card_primary == player_primary:
            return Tier.A
        # Adjacent but share through secondary
        return Tier.B
    elif dist == 2:
        card_res = {card_primary, card_secondary}
        player_res = {player_primary, player_secondary}
        if card_res & player_res:
            return Tier.B
        return Tier.C
    elif dist == 3:
        return Tier.C
    else:
        return Tier.F


@dataclass
class SimCard:
    id: int
    symbols: list  # list of Resonance, 0-3 elements, [] = generic
    archetype_idx: int  # index into ARCHETYPES, -1 for generic
    # For bridge cards, this is the primary archetype; bridge_arch_idx is second
    bridge_arch_idx: int = -1  # if >= 0, card belongs to two archetypes
    fitness: dict = field(default_factory=dict)  # arch_idx -> Tier

    @property
    def primary_resonance(self) -> Optional[Resonance]:
        return self.symbols[0] if self.symbols else None

    @property
    def is_generic(self) -> bool:
        return len(self.symbols) == 0

    @property
    def is_bridge(self) -> bool:
        return self.bridge_arch_idx >= 0


def compute_bridge_fitness(card, player_arch_idx: int) -> Tier:
    """For bridge cards, fitness is S for either home archetype, A for primary-adjacent."""
    if card.archetype_idx == player_arch_idx or card.bridge_arch_idx == player_arch_idx:
        return Tier.S

    # Check adjacency to either home archetype
    for home_idx in [card.archetype_idx, card.bridge_arch_idx]:
        dist = circle_distance(home_idx, player_arch_idx)
        if dist == 1:
            home_primary = ARCHETYPES[home_idx][1]
            player_primary = ARCHETYPES[player_arch_idx][1]
            if home_primary == player_primary:
                return Tier.A

    # Check B-tier through any resonance overlap
    card_res = set()
    for home_idx in [card.archetype_idx, card.bridge_arch_idx]:
        card_res.add(ARCHETYPES[home_idx][1])
        card_res.add(ARCHETYPES[home_idx][2])
    player_res = {ARCHETYPES[player_arch_idx][1], ARCHETYPES[player_arch_idx][2]}
    if card_res & player_res:
        return Tier.B

    return Tier.C


def assign_fitness(card: SimCard):
    """Assign fitness tiers for all archetypes."""
    for j in range(NUM_ARCHETYPES):
        if card.is_generic:
            card.fitness[j] = Tier.B
        elif card.is_bridge:
            card.fitness[j] = compute_bridge_fitness(card, j)
        else:
            card.fitness[j] = compute_fitness(card.archetype_idx, j)


# ---------------------------------------------------------------------------
# Pool generation for each breakdown model
# ---------------------------------------------------------------------------

def make_symbols_for_archetype(arch_idx: int, num_symbols: int,
                               rng: random.Random = None) -> list:
    """Generate a symbol list for a card in the given archetype.

    Default distribution for multi-symbol cards:
    - 2-sym: 2/3 [P,S], 1/3 [P,P]
    - 3-sym: 1/3 [P,P,S], 1/3 [P,S,S], 1/3 [P,S,P]
    """
    primary = ARCHETYPES[arch_idx][1]
    secondary = ARCHETYPES[arch_idx][2]
    r = rng or random

    if num_symbols == 1:
        return [primary]
    elif num_symbols == 2:
        roll = r.random()
        if roll < 0.33:
            return [primary, primary]
        else:
            return [primary, secondary]
    else:  # 3
        roll = r.random()
        if roll < 0.33:
            return [primary, primary, secondary]
        elif roll < 0.67:
            return [primary, secondary, secondary]
        else:
            return [primary, secondary, primary]


def generate_pool_equal_small_generic(total: int = 360, rng=None) -> list[SimCard]:
    """Model 1: Equal cards per archetype + ~10% generic pool.

    36 generic, 324 archetype cards = 40-41 per archetype.
    Default symbol distribution: 20% 1-sym, 55% 2-sym, 25% 3-sym.
    """
    r = rng or random
    cards = []
    card_id = 0
    num_generic = 36  # 10%

    # Generic cards
    for _ in range(num_generic):
        card = SimCard(id=card_id, symbols=[], archetype_idx=-1)
        cards.append(card)
        card_id += 1

    # Archetype cards: equal distribution
    non_generic = total - num_generic
    per_archetype = non_generic // NUM_ARCHETYPES  # 40

    for arch_idx in range(NUM_ARCHETYPES):
        n1 = round(per_archetype * 0.20)  # 8
        n3 = round(per_archetype * 0.25)  # 10
        n2 = per_archetype - n1 - n3      # 22

        for _ in range(n1):
            syms = make_symbols_for_archetype(arch_idx, 1, r)
            cards.append(SimCard(id=card_id, symbols=syms, archetype_idx=arch_idx))
            card_id += 1
        for _ in range(n2):
            syms = make_symbols_for_archetype(arch_idx, 2, r)
            cards.append(SimCard(id=card_id, symbols=syms, archetype_idx=arch_idx))
            card_id += 1
        for _ in range(n3):
            syms = make_symbols_for_archetype(arch_idx, 3, r)
            cards.append(SimCard(id=card_id, symbols=syms, archetype_idx=arch_idx))
            card_id += 1

    # Fill remaining slots if rounding caused a shortfall
    while len(cards) < total:
        arch_idx = r.randint(0, NUM_ARCHETYPES - 1)
        syms = make_symbols_for_archetype(arch_idx, 2, r)
        cards.append(SimCard(id=card_id, symbols=syms, archetype_idx=arch_idx))
        card_id += 1

    for card in cards:
        assign_fitness(card)
    return cards


def generate_pool_equal_large_generic(total: int = 360, rng=None) -> list[SimCard]:
    """Model 2: Equal cards per archetype + ~25% generic pool.

    90 generic, 270 archetype = 33-34 per archetype.
    """
    r = rng or random
    cards = []
    card_id = 0
    num_generic = 90  # 25%

    for _ in range(num_generic):
        cards.append(SimCard(id=card_id, symbols=[], archetype_idx=-1))
        card_id += 1

    non_generic = total - num_generic
    per_archetype = non_generic // NUM_ARCHETYPES  # 33

    for arch_idx in range(NUM_ARCHETYPES):
        n1 = round(per_archetype * 0.20)
        n3 = round(per_archetype * 0.25)
        n2 = per_archetype - n1 - n3

        for _ in range(n1):
            syms = make_symbols_for_archetype(arch_idx, 1, r)
            cards.append(SimCard(id=card_id, symbols=syms, archetype_idx=arch_idx))
            card_id += 1
        for _ in range(n2):
            syms = make_symbols_for_archetype(arch_idx, 2, r)
            cards.append(SimCard(id=card_id, symbols=syms, archetype_idx=arch_idx))
            card_id += 1
        for _ in range(n3):
            syms = make_symbols_for_archetype(arch_idx, 3, r)
            cards.append(SimCard(id=card_id, symbols=syms, archetype_idx=arch_idx))
            card_id += 1

    while len(cards) < total:
        arch_idx = r.randint(0, NUM_ARCHETYPES - 1)
        syms = make_symbols_for_archetype(arch_idx, 2, r)
        cards.append(SimCard(id=card_id, symbols=syms, archetype_idx=arch_idx))
        card_id += 1

    for card in cards:
        assign_fitness(card)
    return cards


def generate_pool_bridge(total: int = 360, rng=None) -> list[SimCard]:
    """Model 3: Equal + explicit bridge card category.

    36 generic (~10%), 48 bridge cards (6 per adjacent pair = 8 pairs * 6),
    276 archetype cards = ~34 per archetype.

    Bridge cards belong to two adjacent archetypes and carry symbols from both.
    They are S-tier for both home archetypes.
    """
    r = rng or random
    cards = []
    card_id = 0
    num_generic = 36
    bridge_per_pair = 6  # 6 bridge cards per adjacent pair

    # Generic cards
    for _ in range(num_generic):
        cards.append(SimCard(id=card_id, symbols=[], archetype_idx=-1))
        card_id += 1

    # Bridge cards: 8 adjacent pairs on the circle
    for arch_idx in range(NUM_ARCHETYPES):
        next_idx = (arch_idx + 1) % NUM_ARCHETYPES
        arch_primary = ARCHETYPES[arch_idx][1]
        next_primary = ARCHETYPES[next_idx][1]

        for _ in range(bridge_per_pair):
            # Bridge cards carry symbols from both archetypes
            # They alternate between having primary from one or the other
            roll = r.random()
            if roll < 0.5:
                # Primary from first archetype
                syms = [arch_primary, next_primary]
            else:
                # Primary from second archetype
                syms = [next_primary, arch_primary]

            card = SimCard(
                id=card_id,
                symbols=syms,
                archetype_idx=arch_idx,
                bridge_arch_idx=next_idx,
            )
            cards.append(card)
            card_id += 1

    # Remaining archetype cards
    num_bridge_total = bridge_per_pair * NUM_ARCHETYPES  # 48
    non_generic_non_bridge = total - num_generic - num_bridge_total  # 276
    per_archetype = non_generic_non_bridge // NUM_ARCHETYPES  # 34

    for arch_idx in range(NUM_ARCHETYPES):
        n1 = round(per_archetype * 0.20)
        n3 = round(per_archetype * 0.25)
        n2 = per_archetype - n1 - n3

        for _ in range(n1):
            syms = make_symbols_for_archetype(arch_idx, 1, r)
            cards.append(SimCard(id=card_id, symbols=syms, archetype_idx=arch_idx))
            card_id += 1
        for _ in range(n2):
            syms = make_symbols_for_archetype(arch_idx, 2, r)
            cards.append(SimCard(id=card_id, symbols=syms, archetype_idx=arch_idx))
            card_id += 1
        for _ in range(n3):
            syms = make_symbols_for_archetype(arch_idx, 3, r)
            cards.append(SimCard(id=card_id, symbols=syms, archetype_idx=arch_idx))
            card_id += 1

    while len(cards) < total:
        arch_idx = r.randint(0, NUM_ARCHETYPES - 1)
        syms = make_symbols_for_archetype(arch_idx, 2, r)
        cards.append(SimCard(id=card_id, symbols=syms, archetype_idx=arch_idx))
        card_id += 1

    for card in cards:
        assign_fitness(card)
    return cards


def generate_pool_asymmetric(total: int = 360, rng=None) -> list[SimCard]:
    """Model 4: Asymmetric archetype sizes.

    36 generic (~10%). Archetypes get different sizes:
    - 2 large (55 cards each): Warriors, Storm
    - 2 medium-large (45 each): Sacrifice, Blink
    - 2 medium (35 each): Flash, Self-Mill
    - 2 small (24 each): Self-Discard, Ramp
    Total non-generic = 324.
    """
    r = rng or random
    cards = []
    card_id = 0
    num_generic = 36

    for _ in range(num_generic):
        cards.append(SimCard(id=card_id, symbols=[], archetype_idx=-1))
        card_id += 1

    # Asymmetric sizes: index -> count
    arch_sizes = {
        0: 35,   # Flash (medium)
        1: 45,   # Blink (medium-large)
        2: 55,   # Storm (large)
        3: 24,   # Self-Discard (small)
        4: 35,   # Self-Mill (medium)
        5: 45,   # Sacrifice (medium-large)
        6: 55,   # Warriors (large)
        7: 24,   # Ramp (small)
    }
    # Verify: sum = 318; remaining = 360 - 36 - 318 = 6 distributed later
    assert sum(arch_sizes.values()) <= total - num_generic

    for arch_idx, count in arch_sizes.items():
        n1 = round(count * 0.20)
        n3 = round(count * 0.25)
        n2 = count - n1 - n3

        for _ in range(n1):
            syms = make_symbols_for_archetype(arch_idx, 1, r)
            cards.append(SimCard(id=card_id, symbols=syms, archetype_idx=arch_idx))
            card_id += 1
        for _ in range(n2):
            syms = make_symbols_for_archetype(arch_idx, 2, r)
            cards.append(SimCard(id=card_id, symbols=syms, archetype_idx=arch_idx))
            card_id += 1
        for _ in range(n3):
            syms = make_symbols_for_archetype(arch_idx, 3, r)
            cards.append(SimCard(id=card_id, symbols=syms, archetype_idx=arch_idx))
            card_id += 1

    while len(cards) < total:
        arch_idx = r.randint(0, NUM_ARCHETYPES - 1)
        syms = make_symbols_for_archetype(arch_idx, 2, r)
        cards.append(SimCard(id=card_id, symbols=syms, archetype_idx=arch_idx))
        card_id += 1

    for card in cards:
        assign_fitness(card)
    return cards


def generate_pool_mono_symbol(total: int = 360, rng=None) -> list[SimCard]:
    """Model 5: Mono-symbol only (all archetype cards have just [Primary]).

    36 generic, 324 archetype = 40-41 per archetype.
    Every archetype card has exactly 1 symbol: its primary resonance.
    """
    r = rng or random
    cards = []
    card_id = 0
    num_generic = 36

    for _ in range(num_generic):
        cards.append(SimCard(id=card_id, symbols=[], archetype_idx=-1))
        card_id += 1

    non_generic = total - num_generic
    per_archetype = non_generic // NUM_ARCHETYPES

    for arch_idx in range(NUM_ARCHETYPES):
        primary = ARCHETYPES[arch_idx][1]
        for _ in range(per_archetype):
            cards.append(SimCard(
                id=card_id,
                symbols=[primary],
                archetype_idx=arch_idx,
            ))
            card_id += 1

    while len(cards) < total:
        arch_idx = r.randint(0, NUM_ARCHETYPES - 1)
        primary = ARCHETYPES[arch_idx][1]
        cards.append(SimCard(id=card_id, symbols=[primary], archetype_idx=arch_idx))
        card_id += 1

    for card in cards:
        assign_fitness(card)
    return cards


# ---------------------------------------------------------------------------
# Pack Widening v3 Algorithm
# ---------------------------------------------------------------------------

@dataclass
class DraftState:
    """State of one draft in progress using Pack Widening v3."""
    tokens: dict = field(default_factory=lambda: {r: 0 for r in Resonance})
    picks: list = field(default_factory=list)
    target_archetype: int = -1
    spend_count: int = 0
    total_packs: int = 0

    # Per-pick tracking
    pick_sa_counts: list = field(default_factory=list)  # SA per pack at each pick
    pick_pack_sizes: list = field(default_factory=list)
    spend_picks: list = field(default_factory=list)  # which picks involved spending
    pick_diversity: list = field(default_factory=list)  # unique archs with S/A per pack
    pick_off_arch: list = field(default_factory=list)  # C/F cards per pack
    pick_sa_stdev_data: list = field(default_factory=list)  # raw SA counts for picks 6+

    # Bonus card tracking
    bonus_sa_hits: int = 0
    bonus_total: int = 0


def is_sa(card: SimCard, arch_idx: int) -> bool:
    tier = card.fitness.get(arch_idx, Tier.F)
    return tier in (Tier.S, Tier.A)


def is_s(card: SimCard, arch_idx: int) -> bool:
    return card.fitness.get(arch_idx, Tier.F) == Tier.S


def is_cf(card: SimCard, arch_idx: int) -> bool:
    tier = card.fitness.get(arch_idx, Tier.F)
    return tier in (Tier.C, Tier.F)


def choose_spend_resonance(state: DraftState) -> Optional[Resonance]:
    """Choose which resonance to spend on, if any.

    Committed player spends on their primary resonance when able.
    """
    if state.target_archetype < 0:
        return None

    target_primary = ARCHETYPES[state.target_archetype][1]
    if state.tokens[target_primary] >= 3:
        return target_primary

    # Also try secondary
    target_secondary = ARCHETYPES[state.target_archetype][2]
    if state.tokens[target_secondary] >= 3:
        return target_secondary

    return None


def generate_pack(state: DraftState, pool: list[SimCard],
                  pool_by_primary: dict, rng=None) -> tuple[list[SimCard], Optional[Resonance]]:
    """Generate a pack using Pack Widening v3.

    Returns (pack, spent_resonance).
    """
    r = rng or random

    # Step a: Check if player can and wants to spend
    spend_res = choose_spend_resonance(state)
    if spend_res is not None:
        state.tokens[spend_res] -= 3
        state.spend_count += 1

    # Step b: Draw 4 cards uniformly at random
    pack = [r.choice(pool) for _ in range(4)]

    # Step c: If spent, draw 1 bonus card from primary-resonance pool
    if spend_res is not None:
        candidates = pool_by_primary.get(spend_res, pool)
        if candidates:
            bonus_card = r.choice(candidates)
            pack.append(bonus_card)

    state.total_packs += 1
    return pack, spend_res


def earn_tokens(state: DraftState, card: SimCard):
    """Add tokens after picking a card."""
    if not card.symbols:
        return  # Generic cards earn nothing

    for i, sym in enumerate(card.symbols):
        if i == 0:
            state.tokens[sym] += 2  # Primary
        else:
            state.tokens[sym] += 1  # Secondary/Tertiary


def pick_card_committed(pack: list[SimCard], state: DraftState, pick_num: int,
                        rng=None) -> SimCard:
    """Archetype-committed player picks the best card for their archetype.

    First 2 picks: random (not yet committed).
    After: best fitness tier, then random among ties.
    """
    r = rng or random

    if pick_num <= 2:
        return r.choice(pack)

    arch = state.target_archetype
    tier_priority = [Tier.S, Tier.A, Tier.B, Tier.C, Tier.F]
    for tier in tier_priority:
        candidates = [c for c in pack if c.fitness.get(arch, Tier.F) == tier]
        if candidates:
            return r.choice(candidates)

    return r.choice(pack)


# ---------------------------------------------------------------------------
# Build pool index
# ---------------------------------------------------------------------------

def build_pool_by_primary(pool: list[SimCard]) -> dict:
    """Index cards by primary resonance for bonus card draws."""
    by_primary = defaultdict(list)
    for card in pool:
        if card.primary_resonance is not None:
            by_primary[card.primary_resonance].append(card)
    return dict(by_primary)


# ---------------------------------------------------------------------------
# Simulate one draft
# ---------------------------------------------------------------------------

def simulate_draft(pool: list[SimCard], pool_by_primary: dict,
                   target_arch: int, num_picks: int = 30,
                   rng=None) -> DraftState:
    """Run one full draft for an archetype-committed player."""
    r = rng or random
    state = DraftState()
    state.target_archetype = target_arch

    for pick_num in range(1, num_picks + 1):
        pack, spend_res = generate_pack(state, pool, pool_by_primary, r)

        # Measure pack before picking
        sa_count = sum(1 for c in pack if is_sa(c, target_arch))
        state.pick_sa_counts.append(sa_count)
        state.pick_pack_sizes.append(len(pack))

        if spend_res is not None:
            state.spend_picks.append(pick_num)
            # Track bonus card hit rate (last card in pack is the bonus)
            bonus = pack[-1]
            state.bonus_total += 1
            if is_sa(bonus, target_arch):
                state.bonus_sa_hits += 1

        # Measure diversity: unique archetypes with S/A cards
        archs_with_sa = set()
        for card in pack:
            for a in range(NUM_ARCHETYPES):
                if is_sa(card, a):
                    archs_with_sa.add(a)
        state.pick_diversity.append(len(archs_with_sa))

        # Measure off-archetype (C/F) cards
        cf_count = sum(1 for c in pack if is_cf(c, target_arch))
        state.pick_off_arch.append(cf_count)

        # Pick
        card = pick_card_committed(pack, state, pick_num, r)
        earn_tokens(state, card)
        state.picks.append(card)

    return state


# ---------------------------------------------------------------------------
# Bridge strategy simulation
# ---------------------------------------------------------------------------

def simulate_bridge_draft(pool: list[SimCard], pool_by_primary: dict,
                          arch1: int, arch2: int, num_picks: int = 30,
                          rng=None) -> dict:
    """Simulate a player committed to two adjacent archetypes.

    They spend on the shared primary resonance and count S/A hits for BOTH.
    """
    r = rng or random
    state = DraftState()
    # Target the first archetype for spending decisions
    state.target_archetype = arch1

    both_sa_packs = 0
    total_late_packs = 0
    sa_for_arch1 = 0
    sa_for_arch2 = 0
    total_late_cards = 0

    for pick_num in range(1, num_picks + 1):
        pack, spend_res = generate_pack(state, pool, pool_by_primary, r)

        if pick_num >= 6:
            # Count S/A for each archetype in the pack
            sa1 = sum(1 for c in pack if is_sa(c, arch1))
            sa2 = sum(1 for c in pack if is_sa(c, arch2))
            if sa1 >= 1 and sa2 >= 1:
                both_sa_packs += 1
            total_late_packs += 1
            sa_for_arch1 += sa1
            sa_for_arch2 += sa2
            total_late_cards += len(pack)

        # Pick the card with best combined fitness
        def combined_score(c):
            t1 = c.fitness.get(arch1, Tier.F)
            t2 = c.fitness.get(arch2, Tier.F)
            score_map = {Tier.S: 4, Tier.A: 3, Tier.B: 2, Tier.C: 1, Tier.F: 0}
            return score_map[t1] + score_map[t2]

        if pick_num <= 2:
            card = r.choice(pack)
        else:
            best_score = max(combined_score(c) for c in pack)
            candidates = [c for c in pack if combined_score(c) == best_score]
            card = r.choice(candidates)

        earn_tokens(state, card)
        state.picks.append(card)

    both_pct = (both_sa_packs / total_late_packs * 100) if total_late_packs > 0 else 0
    avg_sa1 = sa_for_arch1 / total_late_packs if total_late_packs > 0 else 0
    avg_sa2 = sa_for_arch2 / total_late_packs if total_late_packs > 0 else 0

    return {
        "both_sa_pct": both_pct,
        "avg_sa_arch1": avg_sa1,
        "avg_sa_arch2": avg_sa2,
    }


# ---------------------------------------------------------------------------
# Metrics collection
# ---------------------------------------------------------------------------

@dataclass
class BreakdownMetrics:
    name: str

    # Standard targets
    early_diversity: float = 0.0        # picks 1-5: unique archs with S/A per pack
    early_sa_emerging: float = 0.0      # picks 1-5: S/A for emerging arch per pack
    late_sa: float = 0.0                # picks 6+: S/A per pack avg
    late_off_arch: float = 0.0          # picks 6+: C/F cards per pack avg
    convergence_pick: int = 30          # first pick with avg SA >= 2
    deck_sa_pct: float = 0.0           # fraction of final deck that is S/A
    card_overlap_pct: float = 0.0       # run-to-run variety
    archetype_freq: dict = field(default_factory=dict)  # arch -> fraction of drafts
    sa_stddev_late: float = 0.0         # stddev of S/A per pack picks 6+

    # Agent 3 specific
    s_tier_pct: float = 0.0            # fraction of S-tier in final deck
    a_tier_pct: float = 0.0            # fraction of A-tier in final deck
    bridge_viability: float = 0.0       # % of late packs with S/A for both archetypes
    bridge_avg_sa1: float = 0.0
    bridge_avg_sa2: float = 0.0
    generic_token_impact: float = 0.0   # avg tokens/pick (lower with more generics)
    bonus_sa_hit_rate: float = 0.0      # fraction of bonus cards that are S/A
    spend_frequency: float = 0.0        # fraction of picks 6+ that involve spending
    first_spend_pick: float = 0.0       # avg first spend pick

    # Convergence curve
    convergence_curve: dict = field(default_factory=dict)  # pick -> avg S/A


def run_model(name: str, pool_gen_fn, num_drafts: int = 1000,
              num_picks: int = 30) -> BreakdownMetrics:
    """Run full simulation for one breakdown model."""
    rng = random.Random(42)
    pool = pool_gen_fn(rng=rng)
    pool_by_primary = build_pool_by_primary(pool)

    metrics = BreakdownMetrics(name=name)

    # Per-pick tracking across all drafts
    all_sa_by_pick = defaultdict(list)
    all_div_by_pick = defaultdict(list)
    all_off_by_pick = defaultdict(list)

    # Deck composition tracking
    all_s_fracs = []
    all_a_fracs = []
    all_sa_fracs = []

    # Token tracking
    all_tokens_per_pick = []

    # Spend tracking
    all_first_spend = []
    all_spend_fracs = []

    # Bonus card tracking
    total_bonus_sa = 0
    total_bonus = 0

    # Archetype frequency tracking
    arch_target_counts = defaultdict(int)

    # Card overlap tracking (for same target archetype)
    arch_card_sets = defaultdict(list)  # arch -> list of sets of card IDs

    for _ in range(num_drafts):
        target_arch = rng.randint(0, NUM_ARCHETYPES - 1)
        arch_target_counts[target_arch] += 1

        state = simulate_draft(pool, pool_by_primary, target_arch, num_picks, rng)

        # Record per-pick data
        for p in range(num_picks):
            pick_num = p + 1
            all_sa_by_pick[pick_num].append(state.pick_sa_counts[p])
            all_div_by_pick[pick_num].append(state.pick_diversity[p])
            all_off_by_pick[pick_num].append(state.pick_off_arch[p])

        # Deck composition
        s_count = sum(1 for c in state.picks if is_s(c, target_arch))
        a_count = sum(1 for c in state.picks if is_sa(c, target_arch) and not is_s(c, target_arch))
        sa_count = s_count + a_count
        all_s_fracs.append(s_count / num_picks)
        all_a_fracs.append(a_count / num_picks)
        all_sa_fracs.append(sa_count / num_picks)

        # Token earn rate
        total_tokens = sum(state.tokens.values())
        # Tokens earned = current tokens + tokens spent (3 per spend)
        total_earned = total_tokens + state.spend_count * 3
        all_tokens_per_pick.append(total_earned / num_picks)

        # Spend tracking
        if state.spend_picks:
            all_first_spend.append(state.spend_picks[0])
        late_picks = num_picks - 5  # picks 6-30
        late_spends = sum(1 for sp in state.spend_picks if sp >= 6)
        all_spend_fracs.append(late_spends / late_picks if late_picks > 0 else 0)

        # Bonus card tracking
        total_bonus_sa += state.bonus_sa_hits
        total_bonus += state.bonus_total

        # Card overlap
        card_ids = frozenset(c.id for c in state.picks)
        arch_card_sets[target_arch].append(card_ids)

    # Compute metrics
    # Early diversity (picks 1-5)
    early_div = []
    for p in range(1, 6):
        early_div.extend(all_div_by_pick[p])
    metrics.early_diversity = statistics.mean(early_div)

    # Early SA for emerging archetype (picks 1-5)
    early_sa = []
    for p in range(1, 6):
        early_sa.extend(all_sa_by_pick[p])
    metrics.early_sa_emerging = statistics.mean(early_sa)

    # Late SA (picks 6+)
    late_sa = []
    for p in range(6, num_picks + 1):
        late_sa.extend(all_sa_by_pick[p])
    metrics.late_sa = statistics.mean(late_sa)

    # Late off-archetype (picks 6+)
    late_off = []
    for p in range(6, num_picks + 1):
        late_off.extend(all_off_by_pick[p])
    metrics.late_off_arch = statistics.mean(late_off)

    # SA stddev for late picks
    metrics.sa_stddev_late = statistics.stdev(late_sa) if len(late_sa) > 1 else 0

    # Convergence curve
    for p in range(1, num_picks + 1):
        metrics.convergence_curve[p] = statistics.mean(all_sa_by_pick[p])

    # Convergence pick
    for p in range(1, num_picks + 1):
        if metrics.convergence_curve[p] >= 2.0:
            metrics.convergence_pick = p
            break

    # Deck composition
    metrics.deck_sa_pct = statistics.mean(all_sa_fracs) * 100
    metrics.s_tier_pct = statistics.mean(all_s_fracs) * 100
    metrics.a_tier_pct = statistics.mean(all_a_fracs) * 100

    # Token earn rate
    metrics.generic_token_impact = statistics.mean(all_tokens_per_pick)

    # Spend tracking
    metrics.first_spend_pick = statistics.mean(all_first_spend) if all_first_spend else 31
    metrics.spend_frequency = statistics.mean(all_spend_fracs)

    # Bonus card hit rate
    metrics.bonus_sa_hit_rate = (total_bonus_sa / total_bonus * 100) if total_bonus > 0 else 0

    # Archetype frequency
    total_drafts = num_drafts
    for a in range(NUM_ARCHETYPES):
        metrics.archetype_freq[a] = arch_target_counts[a] / total_drafts * 100

    # Card overlap: for each archetype, compute pairwise Jaccard overlap
    overlaps = []
    for a in range(NUM_ARCHETYPES):
        sets = arch_card_sets[a]
        for i in range(len(sets)):
            for j in range(i + 1, min(i + 20, len(sets))):  # Sample pairs
                if len(sets[i] | sets[j]) > 0:
                    jaccard = len(sets[i] & sets[j]) / len(sets[i] | sets[j])
                    overlaps.append(jaccard)
    metrics.card_overlap_pct = statistics.mean(overlaps) * 100 if overlaps else 0

    # Bridge strategy: test all 8 adjacent pairs
    bridge_results = []
    for arch_idx in range(NUM_ARCHETYPES):
        next_idx = (arch_idx + 1) % NUM_ARCHETYPES
        for _ in range(125):  # 125 * 8 = 1000 bridge drafts
            result = simulate_bridge_draft(pool, pool_by_primary,
                                           arch_idx, next_idx, num_picks, rng)
            bridge_results.append(result)

    metrics.bridge_viability = statistics.mean([r["both_sa_pct"] for r in bridge_results])
    metrics.bridge_avg_sa1 = statistics.mean([r["avg_sa_arch1"] for r in bridge_results])
    metrics.bridge_avg_sa2 = statistics.mean([r["avg_sa_arch2"] for r in bridge_results])

    return metrics


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

def print_results(all_metrics: list[BreakdownMetrics]):
    print("\n" + "=" * 130)
    print("ARCHETYPE BREAKDOWN SIMULATION — Pack Widening v3")
    print("=" * 130)

    # Table 1: Standard Targets
    print("\n### STANDARD TARGETS")
    print(f"{'Model':<35} {'EarlyDiv':>8} {'EarlySA':>8} {'LateSA':>8} {'LateOff':>8} {'Conv':>6} {'DeckSA%':>8} {'Overlap%':>9} {'SA_SD':>6}")
    print("-" * 106)
    for m in all_metrics:
        print(f"{m.name:<35} {m.early_diversity:>8.2f} {m.early_sa_emerging:>8.2f} {m.late_sa:>8.2f} {m.late_off_arch:>8.2f} {m.convergence_pick:>6} {m.deck_sa_pct:>7.1f}% {m.card_overlap_pct:>8.1f}% {m.sa_stddev_late:>6.2f}")

    # Table 2: S-tier vs A-tier and Bridge Strategy
    print("\n### S-TIER / A-TIER BREAKDOWN & BRIDGE VIABILITY")
    print(f"{'Model':<35} {'S-tier%':>8} {'A-tier%':>8} {'Bridge%':>8} {'Br.SA1':>7} {'Br.SA2':>7} {'BonusSA%':>9}")
    print("-" * 90)
    for m in all_metrics:
        print(f"{m.name:<35} {m.s_tier_pct:>7.1f}% {m.a_tier_pct:>7.1f}% {m.bridge_viability:>7.1f}% {m.bridge_avg_sa1:>7.2f} {m.bridge_avg_sa2:>7.2f} {m.bonus_sa_hit_rate:>8.1f}%")

    # Table 3: Token Economy & Generics Impact
    print("\n### TOKEN ECONOMY & GENERIC IMPACT")
    print(f"{'Model':<35} {'Tok/Pick':>9} {'1stSpend':>9} {'SpendFrq':>9}")
    print("-" * 68)
    for m in all_metrics:
        first_sp_str = f"{m.first_spend_pick:.1f}" if m.first_spend_pick < 31 else "Never"
        print(f"{m.name:<35} {m.generic_token_impact:>9.2f} {first_sp_str:>9} {m.spend_frequency*100:>8.1f}%")

    # Table 4: Convergence Curves
    print("\n### CONVERGENCE CURVES (S/A per pack at select picks)")
    header = f"{'Model':<35}"
    for p in [1, 3, 5, 7, 10, 15, 20, 25, 30]:
        header += f" {'P'+str(p):>5}"
    print(header)
    print("-" * len(header))
    for m in all_metrics:
        line = f"{m.name:<35}"
        for p in [1, 3, 5, 7, 10, 15, 20, 25, 30]:
            line += f" {m.convergence_curve.get(p, 0):>5.2f}"
        print(line)

    # Table 5: Archetype Frequency Balance
    print("\n### ARCHETYPE FREQUENCY (target archetype distribution, should be ~12.5% each)")
    header = f"{'Model':<35}"
    for a in ARCHETYPE_NAMES:
        header += f" {a[:6]:>7}"
    print(header)
    print("-" * len(header))
    for m in all_metrics:
        line = f"{m.name:<35}"
        for a in range(NUM_ARCHETYPES):
            line += f" {m.archetype_freq.get(a, 0):>6.1f}%"
        print(line)

    # Pool composition summary
    print("\n### POOL COMPOSITION SUMMARY")
    pool_configs = [
        ("Equal+SmallGeneric(10%)", 36, 324, "40-41 per arch", "None"),
        ("Equal+LargeGeneric(25%)", 90, 270, "33-34 per arch", "None"),
        ("Equal+BridgeCards", 36, 276, "34 per arch + 48 bridge", "48 bridge (6 per pair)"),
        ("Asymmetric", 36, 324, "24-55 per arch", "None"),
        ("Mono-Symbol Only", 36, 324, "40-41 per arch, all 1-sym", "None"),
    ]
    print(f"{'Model':<30} {'Generic':>8} {'Archetype':>10} {'Distribution':<30} {'Bridge':>20}")
    print("-" * 105)
    for name, gen, arch, dist, bridge in pool_configs:
        print(f"{name:<30} {gen:>8} {arch:>10} {dist:<30} {bridge:>20}")

    # Target comparison
    print("\n### TARGET COMPARISON")
    print(f"{'Metric':<50} {'Target':<15}", end="")
    for m in all_metrics:
        print(f" {m.name[:12]:>12}", end="")
    print()
    print("-" * (65 + 13 * len(all_metrics)))

    targets = [
        ("Picks 1-5: unique archs w/ S/A per pack", ">= 3", lambda m: f"{m.early_diversity:.1f}"),
        ("Picks 1-5: S/A for emerging arch per pack", "<= 2", lambda m: f"{m.early_sa_emerging:.2f}"),
        ("Picks 6+: S/A for committed arch per pack", ">= 2 avg", lambda m: f"{m.late_sa:.2f}"),
        ("Picks 6+: off-arch (C/F) per pack", ">= 0.5 avg", lambda m: f"{m.late_off_arch:.2f}"),
        ("Convergence pick", "5-8", lambda m: f"{m.convergence_pick}"),
        ("Deck archetype concentration", "60-90% S/A", lambda m: f"{m.deck_sa_pct:.1f}%"),
        ("Run-to-run variety", "< 40% overlap", lambda m: f"{m.card_overlap_pct:.1f}%"),
        ("StdDev S/A per pack (picks 6+)", ">= 0.8", lambda m: f"{m.sa_stddev_late:.2f}"),
    ]

    for metric_name, target, extractor in targets:
        print(f"{metric_name:<50} {target:<15}", end="")
        for m in all_metrics:
            print(f" {extractor(m):>12}", end="")
        print()


def main():
    random.seed(42)
    all_metrics = []

    models = [
        ("Equal + Small Generic (10%)", generate_pool_equal_small_generic),
        ("Equal + Large Generic (25%)", generate_pool_equal_large_generic),
        ("Equal + Bridge Cards", generate_pool_bridge),
        ("Asymmetric Sizes", generate_pool_asymmetric),
        ("Mono-Symbol Only", generate_pool_mono_symbol),
    ]

    for name, gen_fn in models:
        print(f"Running: {name}...")
        m = run_model(name, gen_fn, num_drafts=1000, num_picks=30)
        all_metrics.append(m)

    print_results(all_metrics)


if __name__ == "__main__":
    main()
