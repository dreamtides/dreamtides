#!/usr/bin/env python3
"""
V4 Pool Synthesis Simulation for Pack Widening v3.

Reconciles findings from 5 parallel investigations:
  1. Symbol count distribution (50/35/15 best for save/spend rhythm)
  2. Rarity system (TCG standard, orthogonal to convergence)
  3. Archetype breakdown (equal + 10% generic + 48 bridge cards)
  4. Symbol patterns (Concentrated+Bridge mix for genuine choice)
  5. Algorithm parameters (Cost 5, Bonus 2, Weight 3 mandatory)

Tests the RECONCILED pool design against the V4 default baseline.
Runs 1200 drafts per configuration x 3 strategies x 3 configs = ~10,800 drafts.
"""

import random
import statistics
from dataclasses import dataclass, field
from enum import Enum
from collections import defaultdict
from typing import Optional


# ============================================================
# Core types
# ============================================================

class Resonance(Enum):
    EMBER = "Ember"
    STONE = "Stone"
    TIDE = "Tide"
    ZEPHYR = "Zephyr"

class Rarity(Enum):
    COMMON = "Common"
    UNCOMMON = "Uncommon"
    RARE = "Rare"
    LEGENDARY = "Legendary"

class Tier(Enum):
    S = "S"
    A = "A"
    B = "B"
    C = "C"
    F = "F"

POWER_RANGES = {
    Rarity.COMMON: (2.0, 5.0),
    Rarity.UNCOMMON: (4.0, 7.0),
    Rarity.RARE: (6.0, 9.0),
    Rarity.LEGENDARY: (8.0, 10.0),
}

# ============================================================
# Archetype circle
# ============================================================

ARCHETYPES = [
    ("Flash",        Resonance.ZEPHYR, Resonance.EMBER),   # 0
    ("Blink",        Resonance.EMBER,  Resonance.ZEPHYR),  # 1
    ("Storm",        Resonance.EMBER,  Resonance.STONE),   # 2
    ("SelfDiscard",  Resonance.STONE,  Resonance.EMBER),   # 3
    ("SelfMill",     Resonance.STONE,  Resonance.TIDE),    # 4
    ("Sacrifice",    Resonance.TIDE,   Resonance.STONE),   # 5
    ("Warriors",     Resonance.TIDE,   Resonance.ZEPHYR),  # 6
    ("Ramp",         Resonance.ZEPHYR, Resonance.TIDE),    # 7
]

ARCHETYPE_NAMES = [a[0] for a in ARCHETYPES]
NUM_ARCHETYPES = 8
NUM_PICKS = 30
TOTAL_CARDS = 360
NUM_GENERIC = 36


def circle_distance(i: int, j: int) -> int:
    d = abs(i - j)
    return min(d, NUM_ARCHETYPES - d)


def compute_fitness(card_arch_idx: int, player_arch_idx: int) -> Tier:
    """Compute archetype-level fitness tier."""
    if card_arch_idx < 0:
        return Tier.B  # generic
    if card_arch_idx == player_arch_idx:
        return Tier.S
    dist = circle_distance(card_arch_idx, player_arch_idx)
    if dist == 1:
        card_primary = ARCHETYPES[card_arch_idx][1]
        player_primary = ARCHETYPES[player_arch_idx][1]
        if card_primary == player_primary:
            return Tier.A
        return Tier.B
    elif dist == 2:
        card_res = {ARCHETYPES[card_arch_idx][1], ARCHETYPES[card_arch_idx][2]}
        player_res = {ARCHETYPES[player_arch_idx][1], ARCHETYPES[player_arch_idx][2]}
        if card_res & player_res:
            return Tier.B
        return Tier.C
    elif dist == 3:
        return Tier.C
    else:
        return Tier.F


def compute_bridge_fitness(home_a: int, home_b: int, player_arch_idx: int) -> Tier:
    """Fitness for bridge cards that belong to two adjacent archetypes."""
    if home_a == player_arch_idx or home_b == player_arch_idx:
        return Tier.S
    for home_idx in [home_a, home_b]:
        dist = circle_distance(home_idx, player_arch_idx)
        if dist == 1:
            if ARCHETYPES[home_idx][1] == ARCHETYPES[player_arch_idx][1]:
                return Tier.A
    card_res = set()
    for home_idx in [home_a, home_b]:
        card_res.add(ARCHETYPES[home_idx][1])
        card_res.add(ARCHETYPES[home_idx][2])
    player_res = {ARCHETYPES[player_arch_idx][1], ARCHETYPES[player_arch_idx][2]}
    if card_res & player_res:
        return Tier.B
    return Tier.C


# ============================================================
# Card
# ============================================================

@dataclass
class SimCard:
    id: int
    symbols: list       # list of Resonance, 0-3 elements
    archetype_idx: int  # -1 for generic
    bridge_idx: int     # -1 if not bridge, otherwise second archetype index
    rarity: Rarity
    power: float
    fitness: dict = field(default_factory=dict)  # arch_idx -> Tier

    @property
    def primary_resonance(self) -> Optional[Resonance]:
        return self.symbols[0] if self.symbols else None

    def is_sa(self, arch_idx: int) -> bool:
        return self.fitness.get(arch_idx, Tier.F) in (Tier.S, Tier.A)

    def is_cf(self, arch_idx: int) -> bool:
        return self.fitness.get(arch_idx, Tier.F) in (Tier.C, Tier.F)


def assign_fitness(card: SimCard):
    """Assign fitness for all 8 archetypes."""
    for j in range(NUM_ARCHETYPES):
        if card.archetype_idx < 0:
            card.fitness[j] = Tier.B
        elif card.bridge_idx >= 0:
            card.fitness[j] = compute_bridge_fitness(card.archetype_idx, card.bridge_idx, j)
        else:
            card.fitness[j] = compute_fitness(card.archetype_idx, j)


# ============================================================
# Symbol pattern generation
# ============================================================

def resolve_pattern(pattern: list[str], primary: Resonance,
                    secondary: Resonance) -> list[Resonance]:
    """Resolve symbolic pattern ['P','S','O'] into actual resonances."""
    others = [r for r in Resonance if r != primary and r != secondary]
    result = []
    for sym in pattern:
        if sym == 'P':
            result.append(primary)
        elif sym == 'S':
            result.append(secondary)
        elif sym == 'O':
            result.append(random.choice(others))
    return result


# ============================================================
# Pool generation
# ============================================================

@dataclass
class PoolConfig:
    """Configuration for building a card pool."""
    name: str
    cards_per_archetype: int
    generic_count: int
    bridge_per_pair: int  # bridge cards per adjacent archetype pair

    # Symbol count distribution for non-bridge archetype cards: {1: frac, 2: frac, 3: frac}
    symbol_dist: dict

    # Pattern weights per symbol count
    # 1-sym patterns: {"P": weight, "S": weight}
    # 2-sym patterns: {"PP": weight, "PS": weight, "PO": weight}
    # 3-sym patterns: {"PPS": weight, "PPO": weight, "PPS": weight, "PSO": weight}
    pattern_weights_1: dict
    pattern_weights_2: dict
    pattern_weights_3: dict

    # Algorithm parameters
    spend_cost: int
    bonus_cards: int
    primary_weight: int

    # Rarity distribution
    rarity_dist: dict = field(default_factory=lambda: {
        Rarity.COMMON: 180, Rarity.UNCOMMON: 100,
        Rarity.RARE: 60, Rarity.LEGENDARY: 20,
    })


def build_pool(config: PoolConfig) -> list[SimCard]:
    """Build the complete card pool from a PoolConfig."""
    cards: list[SimCard] = []
    card_id = 0

    # 1. Generic cards (no symbols)
    for _ in range(config.generic_count):
        cards.append(SimCard(
            id=card_id, symbols=[], archetype_idx=-1, bridge_idx=-1,
            rarity=Rarity.COMMON, power=0.0,
        ))
        card_id += 1

    # 2. Bridge cards (6 per adjacent pair, 8 pairs = 48 total if bridge_per_pair=6)
    if config.bridge_per_pair > 0:
        for arch_idx in range(NUM_ARCHETYPES):
            next_idx = (arch_idx + 1) % NUM_ARCHETYPES
            arch_pri = ARCHETYPES[arch_idx][1]
            next_pri = ARCHETYPES[next_idx][1]
            for i in range(config.bridge_per_pair):
                # Alternate primary resonance between the two archetypes
                if i % 2 == 0:
                    syms = [arch_pri, next_pri]
                else:
                    syms = [next_pri, arch_pri]
                cards.append(SimCard(
                    id=card_id, symbols=syms, archetype_idx=arch_idx,
                    bridge_idx=next_idx, rarity=Rarity.COMMON, power=0.0,
                ))
                card_id += 1

    # 3. Regular archetype cards
    for arch_idx in range(NUM_ARCHETYPES):
        pri = ARCHETYPES[arch_idx][1]
        sec = ARCHETYPES[arch_idx][2]
        n_total = config.cards_per_archetype

        # Allocate by symbol count
        n1 = round(n_total * config.symbol_dist.get(1, 0))
        n3 = round(n_total * config.symbol_dist.get(3, 0))
        n2 = n_total - n1 - n3

        # 1-symbol cards
        pats_1, weights_1 = _normalize_patterns(config.pattern_weights_1)
        for _ in range(n1):
            pat = random.choices(pats_1, weights=weights_1, k=1)[0]
            syms = resolve_pattern(list(pat), pri, sec)
            cards.append(SimCard(
                id=card_id, symbols=syms, archetype_idx=arch_idx,
                bridge_idx=-1, rarity=Rarity.COMMON, power=0.0,
            ))
            card_id += 1

        # 2-symbol cards
        pats_2, weights_2 = _normalize_patterns(config.pattern_weights_2)
        for _ in range(n2):
            pat = random.choices(pats_2, weights=weights_2, k=1)[0]
            syms = resolve_pattern(list(pat), pri, sec)
            cards.append(SimCard(
                id=card_id, symbols=syms, archetype_idx=arch_idx,
                bridge_idx=-1, rarity=Rarity.COMMON, power=0.0,
            ))
            card_id += 1

        # 3-symbol cards
        pats_3, weights_3 = _normalize_patterns(config.pattern_weights_3)
        for _ in range(n3):
            pat = random.choices(pats_3, weights=weights_3, k=1)[0]
            syms = resolve_pattern(list(pat), pri, sec)
            cards.append(SimCard(
                id=card_id, symbols=syms, archetype_idx=arch_idx,
                bridge_idx=-1, rarity=Rarity.COMMON, power=0.0,
            ))
            card_id += 1

    # Pad or trim to TOTAL_CARDS
    while len(cards) < TOTAL_CARDS:
        arch_idx = random.randint(0, NUM_ARCHETYPES - 1)
        pri = ARCHETYPES[arch_idx][1]
        sec = ARCHETYPES[arch_idx][2]
        syms = resolve_pattern(["P", "S"], pri, sec)
        cards.append(SimCard(
            id=card_id, symbols=syms, archetype_idx=arch_idx,
            bridge_idx=-1, rarity=Rarity.COMMON, power=0.0,
        ))
        card_id += 1

    if len(cards) > TOTAL_CARDS:
        cards = cards[:TOTAL_CARDS]

    # Assign rarity and power
    _assign_rarities(cards, config.rarity_dist)

    # Compute fitness for all cards
    for card in cards:
        assign_fitness(card)

    return cards


def _normalize_patterns(weights: dict) -> tuple[list, list]:
    """Normalize pattern weights and return (patterns, weights) lists."""
    if not weights:
        return (["P"], [1.0])
    pats = list(weights.keys())
    wts = list(weights.values())
    total = sum(wts)
    if total <= 0:
        return (["P"], [1.0])
    wts = [w / total for w in wts]
    return (pats, wts)


def _assign_rarities(cards: list[SimCard], rarity_dist: dict):
    """Assign rarities and power levels following TCG distribution."""
    total_cards = len(cards)
    rarity_total = sum(rarity_dist.values())
    slots = []
    for r, count in rarity_dist.items():
        n = round(count * total_cards / rarity_total)
        slots.extend([r] * n)
    # Pad/trim to match
    while len(slots) < total_cards:
        slots.append(Rarity.COMMON)
    slots = slots[:total_cards]
    random.shuffle(slots)
    for card, r in zip(cards, slots):
        card.rarity = r
        lo, hi = POWER_RANGES[r]
        card.power = random.uniform(lo, hi)


def build_primary_index(pool: list[SimCard]) -> dict:
    """Index cards by primary resonance for bonus card draws."""
    by_primary = defaultdict(list)
    for card in pool:
        if card.primary_resonance is not None:
            by_primary[card.primary_resonance].append(card)
    return dict(by_primary)


# ============================================================
# Pool configurations
# ============================================================

def make_v4_default_config() -> PoolConfig:
    """V4 Default baseline: 20/55/25 symbols, cost 3, bonus 1, weight 2, no bridge."""
    # With 36 generic and 0 bridge, 324 archetype cards = 40 per archetype (+4 padding)
    return PoolConfig(
        name="V4 Default (C3/B1/W2)",
        cards_per_archetype=40,
        generic_count=36,
        bridge_per_pair=0,
        symbol_dist={1: 0.20, 2: 0.55, 3: 0.25},
        pattern_weights_1={"P": 1.0},
        pattern_weights_2={"PS": 0.65, "PP": 0.20, "PO": 0.15},
        pattern_weights_3={"PPS": 0.40, "PSS": 0.30, "PSO": 0.30},
        spend_cost=3,
        bonus_cards=1,
        primary_weight=2,
    )


def make_reconciled_config() -> PoolConfig:
    """Reconciled design: 50/35/15 symbols, cost 5, bonus 2, weight 3, with bridge."""
    # 36 generic + 48 bridge = 84 non-regular. 360 - 84 = 276 archetype = 34 per arch (+4 padding)
    return PoolConfig(
        name="Reconciled (C5/B2/W3)",
        cards_per_archetype=34,
        generic_count=36,
        bridge_per_pair=6,
        symbol_dist={1: 0.50, 2: 0.35, 3: 0.15},
        # Agent 4's Config G (Concentrated+Bridge):
        # 1-sym: 100% [P]
        pattern_weights_1={"P": 1.0},
        # 2-sym: 45% [P,P], 35% [P,O], 20% [P,S]
        pattern_weights_2={"PP": 0.45, "PO": 0.35, "PS": 0.20},
        # 3-sym: 20% [P,P,P], 30% [P,P,O], 25% [P,P,S], 25% [P,S,O]
        pattern_weights_3={"PPP": 0.20, "PPO": 0.30, "PPS": 0.25, "PSO": 0.25},
        spend_cost=5,
        bonus_cards=2,
        primary_weight=3,
    )


def make_reconciled_alt_config() -> PoolConfig:
    """Alternative reconciled: same pool, cost 4, bonus 2, weight 2."""
    cfg = make_reconciled_config()
    cfg.name = "Reconciled Alt (C4/B2/W2)"
    cfg.spend_cost = 4
    cfg.primary_weight = 2
    return cfg


# ============================================================
# Pack Widening v3 Draft Algorithm
# ============================================================

@dataclass
class DraftState:
    tokens: dict = field(default_factory=lambda: {r: 0 for r in Resonance})
    picks: list = field(default_factory=list)
    target_archetype: int = -1
    spend_picks: list = field(default_factory=list)

    spend_cost: int = 3
    bonus_cards: int = 1
    primary_weight: int = 2


def earn_tokens(state: DraftState, card: SimCard):
    """Add tokens based on the picked card's symbols."""
    for i, sym in enumerate(card.symbols):
        if i == 0:
            state.tokens[sym] += state.primary_weight
        else:
            state.tokens[sym] += 1


def choose_spend_resonance(state: DraftState) -> Optional[Resonance]:
    """Committed player: spend on primary resonance if possible, then secondary."""
    if state.target_archetype < 0:
        return None
    target_pri = ARCHETYPES[state.target_archetype][1]
    if state.tokens[target_pri] >= state.spend_cost:
        return target_pri
    target_sec = ARCHETYPES[state.target_archetype][2]
    if state.tokens[target_sec] >= state.spend_cost:
        return target_sec
    return None


def choose_spend_power_chaser(state: DraftState) -> Optional[Resonance]:
    """Power chaser: spend on whichever resonance has the most tokens."""
    spendable = [r for r in Resonance if state.tokens[r] >= state.spend_cost]
    if not spendable:
        return None
    return max(spendable, key=lambda r: state.tokens[r])


def choose_spend_signal_reader(state: DraftState, best_res: Optional[Resonance]) -> Optional[Resonance]:
    """Signal reader: spend on the resonance they've determined is most open."""
    if best_res and state.tokens[best_res] >= state.spend_cost:
        return best_res
    # Fallback: highest available
    spendable = [r for r in Resonance if state.tokens[r] >= state.spend_cost]
    if spendable:
        return max(spendable, key=lambda r: state.tokens[r])
    return None


def generate_pack(state: DraftState, pool: list[SimCard],
                  primary_index: dict, spend_res: Optional[Resonance]) -> list[SimCard]:
    """Generate a pack. If spend_res is provided, deduct tokens and add bonus cards."""
    if spend_res is not None:
        state.tokens[spend_res] -= state.spend_cost
        state.spend_picks.append(len(state.picks) + 1)

    # Draw base pack (4 random)
    pack = [random.choice(pool) for _ in range(4)]

    # Draw bonus cards if spending
    if spend_res is not None and spend_res in primary_index:
        candidates = primary_index[spend_res]
        for _ in range(state.bonus_cards):
            pack.append(random.choice(candidates))

    return pack


# ============================================================
# Player strategies
# ============================================================

def pick_committed(pack: list[SimCard], arch_idx: int) -> SimCard:
    """Pick best S/A card, break ties by power."""
    tier_order = {Tier.S: 0, Tier.A: 1, Tier.B: 2, Tier.C: 3, Tier.F: 4}
    return min(pack, key=lambda c: (tier_order[c.fitness.get(arch_idx, Tier.F)], -c.power))


def pick_power_chaser(pack: list[SimCard]) -> SimCard:
    """Pick highest raw power."""
    return max(pack, key=lambda c: c.power)


def pick_signal_reader_early(pack: list[SimCard], tokens: dict) -> tuple[SimCard, Optional[Resonance]]:
    """Signal reader early: pick the card that contributes most to the dominant resonance in the pack."""
    res_count = {r: 0 for r in Resonance}
    for card in pack:
        for i, sym in enumerate(card.symbols):
            w = 2 if i == 0 else 1
            res_count[sym] += w
    combined = {r: tokens[r] + res_count[r] for r in Resonance}
    best_res = max(combined, key=lambda r: combined[r])

    def card_contribution(c: SimCard) -> int:
        return sum((2 if i == 0 else 1) for i, s in enumerate(c.symbols) if s == best_res)

    chosen = max(pack, key=lambda c: (card_contribution(c), c.power))
    return chosen, best_res


# ============================================================
# Single draft simulation
# ============================================================

def run_single_draft(pool: list[SimCard], primary_index: dict,
                     config: PoolConfig, target_arch: int,
                     strategy: str = "committed") -> dict:
    """Run one 30-pick draft. Returns detailed per-pick data."""
    state = DraftState()
    state.target_archetype = target_arch
    state.spend_cost = config.spend_cost
    state.bonus_cards = config.bonus_cards
    state.primary_weight = config.primary_weight

    pick_data = []
    signal_best_res: Optional[Resonance] = None

    for pick_num in range(1, NUM_PICKS + 1):
        # Determine spending
        if strategy == "committed":
            spend_res = choose_spend_resonance(state)
        elif strategy == "power":
            spend_res = choose_spend_power_chaser(state)
        elif strategy == "signal":
            if pick_num <= 5:
                spend_res = choose_spend_signal_reader(state, signal_best_res)
            else:
                # After commitment, spend like committed player
                spend_res = choose_spend_resonance(state)
        else:
            spend_res = None

        pack = generate_pack(state, pool, primary_index, spend_res)

        # Measure pack
        sa_count = sum(1 for c in pack if c.is_sa(target_arch))
        cf_count = sum(1 for c in pack if c.is_cf(target_arch))

        # Unique archetypes with S/A in pack
        archs_with_sa = set()
        for card in pack:
            for a in range(NUM_ARCHETYPES):
                if card.is_sa(a):
                    archs_with_sa.add(a)

        # Genuine choice: 2+ S/A cards with different token profiles
        sa_cards = [c for c in pack if c.is_sa(target_arch)]
        sa_profiles = set()
        for c in sa_cards:
            sa_profiles.add(tuple(c.symbols))
        genuine_choice = len(sa_profiles) >= 2

        # Pick a card
        if strategy == "committed":
            if pick_num <= 2:
                chosen = max(pack, key=lambda c: c.power)
            else:
                chosen = pick_committed(pack, target_arch)
        elif strategy == "power":
            chosen = pick_power_chaser(pack)
        elif strategy == "signal":
            if pick_num <= 5:
                chosen, signal_best_res = pick_signal_reader_early(pack, state.tokens)
            else:
                # Determine target archetype from signal
                # Find the archetype with most accumulated tokens as primary
                if signal_best_res:
                    # Find archetype with this as primary resonance
                    for a_idx in range(NUM_ARCHETYPES):
                        if ARCHETYPES[a_idx][1] == signal_best_res:
                            state.target_archetype = a_idx
                            target_arch = a_idx
                            break
                chosen = pick_committed(pack, target_arch)
        else:
            chosen = random.choice(pack)

        earn_tokens(state, chosen)
        state.picks.append(chosen)

        pick_data.append({
            "pick": pick_num,
            "sa_count": sa_count,
            "cf_count": cf_count,
            "archs_with_sa": len(archs_with_sa),
            "pack_size": len(pack),
            "spent": spend_res is not None,
            "genuine_choice": genuine_choice,
        })

    # Deck stats
    deck_sa = sum(1 for c in state.picks if c.is_sa(target_arch))
    deck_sa_pct = deck_sa / NUM_PICKS * 100

    return {
        "pick_data": pick_data,
        "state": state,
        "deck_sa_pct": deck_sa_pct,
        "target_arch": target_arch,
        "drafted_ids": set(c.id for c in state.picks),
        "tokens_final": dict(state.tokens),
        "total_spends": len(state.spend_picks),
        "first_spend": state.spend_picks[0] if state.spend_picks else None,
    }


# ============================================================
# Batch simulation
# ============================================================

@dataclass
class BatchResults:
    """Aggregated results from many drafts."""
    config_name: str
    strategy: str
    n_drafts: int

    # Standard targets
    early_diversity: float = 0.0      # picks 1-5: avg unique archs with S/A per pack
    early_sa: float = 0.0             # picks 1-5: avg S/A for emerging arch
    late_sa: float = 0.0              # picks 6+: avg S/A for committed arch
    late_cf: float = 0.0              # picks 6+: avg C/F off-archetype
    convergence_pick: float = 0.0     # avg first pick with 2+ S/A
    deck_concentration: float = 0.0   # avg % S/A in final deck
    card_overlap: float = 0.0         # % overlap between draft pairs
    arch_freq_min: float = 0.0
    arch_freq_max: float = 0.0
    sa_stddev_late: float = 0.0       # stddev of S/A per pack at picks 6+

    # Token economy
    tokens_per_pick: float = 0.0      # avg tokens earned per pick
    primary_tokens_per_pick: float = 0.0
    first_spend: float = 0.0          # avg first spend pick
    spend_frequency: float = 0.0      # fraction of picks 6+ with spending
    max_no_spend_streak: float = 0.0  # avg longest streak of non-spend picks 6+

    # Decision quality
    genuine_choice_rate: float = 0.0  # fraction of late packs with 2+ distinct S/A profiles

    # Convergence curve: pick -> avg S/A
    sa_curve: dict = field(default_factory=dict)

    # Phase averages
    sa_phase: dict = field(default_factory=dict)

    def passes_target(self) -> dict[str, tuple[str, bool]]:
        return {
            "Early diversity (>=3)": (f"{self.early_diversity:.2f}", self.early_diversity >= 3),
            "Early S/A (<=2)": (f"{self.early_sa:.2f}", self.early_sa <= 2),
            "Late S/A (>=2)": (f"{self.late_sa:.2f}", self.late_sa >= 2.0),
            "Late C/F (>=0.5)": (f"{self.late_cf:.2f}", self.late_cf >= 0.5),
            "Conv pick (5-8)": (f"{self.convergence_pick:.1f}", 5 <= self.convergence_pick <= 8),
            "Deck conc (60-90%)": (f"{self.deck_concentration:.1f}%", 60 <= self.deck_concentration <= 90),
            "Card overlap (<40%)": (f"{self.card_overlap:.1f}%", self.card_overlap < 40),
            "Arch freq (5-20%)": (
                f"{self.arch_freq_min:.1f}-{self.arch_freq_max:.1f}%",
                self.arch_freq_min >= 5 and self.arch_freq_max <= 20,
            ),
            "SA StdDev (>=0.8)": (f"{self.sa_stddev_late:.2f}", self.sa_stddev_late >= 0.8),
        }


def run_batch(pool: list[SimCard], primary_index: dict,
              config: PoolConfig, strategy: str,
              n_drafts: int = 1200) -> BatchResults:
    """Run n_drafts and aggregate all metrics."""
    results = BatchResults(config_name=config.name, strategy=strategy, n_drafts=n_drafts)

    all_pick_sa = defaultdict(list)
    all_pick_cf = defaultdict(list)
    all_pick_div = defaultdict(list)
    all_pick_gc = defaultdict(list)

    deck_sa_pcts = []
    first_spends = []
    arch_counts = defaultdict(int)
    all_drafted_ids = []

    total_tokens_earned = []
    primary_tokens_earned = []
    late_spend_counts = []
    late_pick_counts = []
    no_spend_streaks = []

    for _ in range(n_drafts):
        target_arch = random.randint(0, NUM_ARCHETYPES - 1)
        arch_counts[target_arch] += 1

        draft = run_single_draft(pool, primary_index, config, target_arch, strategy)
        deck_sa_pcts.append(draft["deck_sa_pct"])
        all_drafted_ids.append(draft["drafted_ids"])

        if draft["first_spend"] is not None:
            first_spends.append(draft["first_spend"])

        # Token economy
        final_tokens = draft["tokens_final"]
        spent_tokens = draft["total_spends"] * config.spend_cost
        total_earned = sum(final_tokens.values()) + spent_tokens
        total_tokens_earned.append(total_earned / NUM_PICKS)

        target_pri = ARCHETYPES[target_arch][1]
        pri_earned = final_tokens[target_pri] + sum(
            config.spend_cost for p in draft["state"].spend_picks
            if True  # approximate: all spends on primary
        )
        primary_tokens_earned.append(pri_earned / NUM_PICKS)

        # Late spend tracking
        late_spends = sum(1 for p in draft["state"].spend_picks if p >= 6)
        late_picks = NUM_PICKS - 5
        late_spend_counts.append(late_spends)
        late_pick_counts.append(late_picks)

        # Max non-spend streak in picks 6+
        spend_set = set(draft["state"].spend_picks)
        max_streak = 0
        current_streak = 0
        for p in range(6, NUM_PICKS + 1):
            if p not in spend_set:
                current_streak += 1
                max_streak = max(max_streak, current_streak)
            else:
                current_streak = 0
        no_spend_streaks.append(max_streak)

        for pd in draft["pick_data"]:
            pn = pd["pick"]
            all_pick_sa[pn].append(pd["sa_count"])
            all_pick_cf[pn].append(pd["cf_count"])
            all_pick_div[pn].append(pd["archs_with_sa"])
            all_pick_gc[pn].append(1 if pd["genuine_choice"] else 0)

    # Early metrics (picks 1-5)
    early_div, early_sa_vals = [], []
    for p in range(1, 6):
        early_div.extend(all_pick_div[p])
        early_sa_vals.extend(all_pick_sa[p])
    results.early_diversity = statistics.mean(early_div)
    results.early_sa = statistics.mean(early_sa_vals)

    # Late metrics (picks 6+)
    late_sa_vals, late_cf_vals, late_gc_vals = [], [], []
    for p in range(6, NUM_PICKS + 1):
        late_sa_vals.extend(all_pick_sa[p])
        late_cf_vals.extend(all_pick_cf[p])
        late_gc_vals.extend(all_pick_gc[p])
    results.late_sa = statistics.mean(late_sa_vals)
    results.late_cf = statistics.mean(late_cf_vals)
    results.sa_stddev_late = statistics.stdev(late_sa_vals) if len(late_sa_vals) > 1 else 0
    results.genuine_choice_rate = statistics.mean(late_gc_vals) * 100

    # Convergence curve
    for p in range(1, NUM_PICKS + 1):
        results.sa_curve[p] = statistics.mean(all_pick_sa[p])

    # Convergence pick (first pick where curve >= 2.0)
    conv_picks = []
    for draft_idx in range(n_drafts):
        found = False
        for p in range(1, NUM_PICKS + 1):
            # Check this draft's sa at this pick
            # We stored per-draft data in all_pick_sa[p][draft_idx]
            pass
        # Simpler: use per-draft first-time 2+ SA pack
    # Actually compute from per-draft data:
    for draft_idx in range(n_drafts):
        found_pick = NUM_PICKS + 1
        for p in range(1, NUM_PICKS + 1):
            if all_pick_sa[p][draft_idx] >= 2:
                found_pick = p
                break
        conv_picks.append(found_pick)
    results.convergence_pick = statistics.mean(conv_picks)

    # Deck concentration
    results.deck_concentration = statistics.mean(deck_sa_pcts)

    # Card overlap (Jaccard similarity between random pairs)
    overlaps = []
    sample = min(400, len(all_drafted_ids))
    indices = random.sample(range(len(all_drafted_ids)), sample)
    for i in range(0, len(indices) - 1, 2):
        s1 = all_drafted_ids[indices[i]]
        s2 = all_drafted_ids[indices[i + 1]]
        if s1 or s2:
            overlap = len(s1 & s2) / len(s1 | s2) * 100
            overlaps.append(overlap)
    results.card_overlap = statistics.mean(overlaps) if overlaps else 0

    # Archetype frequency
    total = sum(arch_counts.values())
    freqs = [arch_counts[a] / total * 100 for a in range(NUM_ARCHETYPES)]
    results.arch_freq_min = min(freqs)
    results.arch_freq_max = max(freqs)

    # Token economy
    results.tokens_per_pick = statistics.mean(total_tokens_earned)
    results.primary_tokens_per_pick = statistics.mean(primary_tokens_earned)
    results.first_spend = statistics.mean(first_spends) if first_spends else 31
    results.spend_frequency = sum(late_spend_counts) / sum(late_pick_counts) if sum(late_pick_counts) > 0 else 0
    results.max_no_spend_streak = statistics.mean(no_spend_streaks)

    # Phase averages
    for phase_name, start, end in [("p1_5", 1, 5), ("p6_10", 6, 10), ("p11_15", 11, 15),
                                     ("p16_20", 16, 20), ("p21_25", 21, 25), ("p26_30", 26, 30)]:
        vals = []
        for p in range(start, end + 1):
            vals.extend(all_pick_sa[p])
        results.sa_phase[phase_name] = statistics.mean(vals)

    return results


# ============================================================
# Output formatting
# ============================================================

def print_batch_results(res: BatchResults):
    """Print detailed results for one configuration + strategy."""
    print(f"\n{'='*75}")
    print(f"  {res.config_name} [{res.strategy}]  ({res.n_drafts} drafts)")
    print(f"{'='*75}")

    # Targets
    print("\n  --- Measurable Targets ---")
    pf = res.passes_target()
    passes = 0
    for metric, (val, passed) in pf.items():
        status = "PASS" if passed else "FAIL"
        if passed:
            passes += 1
        print(f"    {metric:30s}  {val:>15s}  {status}")
    print(f"    {'Targets passed':30s}  {passes}/{len(pf)}")

    # Token economy
    print("\n  --- Token Economy ---")
    print(f"    Tokens/pick:         {res.tokens_per_pick:.2f}")
    print(f"    Primary tokens/pick: {res.primary_tokens_per_pick:.2f}")
    print(f"    First spend pick:    {res.first_spend:.1f}")
    print(f"    Spend frequency (6+):{res.spend_frequency:.1%}")
    print(f"    Max no-spend streak: {res.max_no_spend_streak:.1f}")
    print(f"    Genuine choice rate: {res.genuine_choice_rate:.1f}%")

    # Convergence curve
    print("\n  --- Convergence Curve (S/A per pack) ---")
    for phase, label in [("p1_5", "Picks 1-5"), ("p6_10", "Picks 6-10"),
                         ("p11_15", "Picks 11-15"), ("p16_20", "Picks 16-20"),
                         ("p21_25", "Picks 21-25"), ("p26_30", "Picks 26-30")]:
        print(f"    {label:15s}: {res.sa_phase.get(phase, 0):.2f}")
    trend = res.sa_phase.get("p26_30", 0) - res.sa_phase.get("p6_10", 0)
    print(f"    SA Trend (late - mid): {trend:+.2f}")


def print_comparison_table(all_results: dict[str, dict[str, BatchResults]]):
    """Print side-by-side comparison for committed strategy."""
    print("\n\n")
    print("#" * 90)
    print("  COMPARISON: COMMITTED STRATEGY")
    print("#" * 90)

    configs = list(all_results.keys())
    header = f"{'Metric':35s}"
    for cfg in configs:
        short = cfg[:20]
        header += f"  {short:>20s}"
    print(f"\n{header}")
    print("-" * len(header))

    def row(metric_name, extract_fn, fmt="{:.2f}"):
        line = f"{metric_name:35s}"
        for cfg in configs:
            res = all_results[cfg]["committed"]
            val = extract_fn(res)
            line += f"  {fmt.format(val):>20s}"
        print(line)

    row("Late S/A (target >= 2.0)", lambda r: r.late_sa)
    row("Early diversity (target >= 3)", lambda r: r.early_diversity)
    row("Early S/A (target <= 2)", lambda r: r.early_sa)
    row("Late C/F (target >= 0.5)", lambda r: r.late_cf)
    row("Convergence pick (target 5-8)", lambda r: r.convergence_pick, "{:.1f}")
    row("Deck concentration %", lambda r: r.deck_concentration, "{:.1f}%")
    row("Card overlap %", lambda r: r.card_overlap, "{:.1f}%")
    row("SA StdDev late (target >= 0.8)", lambda r: r.sa_stddev_late)
    row("Genuine choice rate %", lambda r: r.genuine_choice_rate, "{:.1f}%")
    row("Tokens/pick", lambda r: r.tokens_per_pick)
    row("First spend pick", lambda r: r.first_spend, "{:.1f}")
    row("Spend frequency (6+)", lambda r: r.spend_frequency, "{:.1%}")
    row("Max no-spend streak", lambda r: r.max_no_spend_streak, "{:.1f}")

    # Targets passed
    line = f"{'Targets passed':35s}"
    for cfg in configs:
        res = all_results[cfg]["committed"]
        passed = sum(1 for _, (_, ok) in res.passes_target().items() if ok)
        total = len(res.passes_target())
        line += f"  {f'{passed}/{total}':>20s}"
    print(line)


def print_strategy_comparison(all_results: dict[str, dict[str, BatchResults]]):
    """Print strategy comparison for the reconciled config."""
    print("\n\n")
    print("#" * 90)
    print("  STRATEGY COMPARISON")
    print("#" * 90)

    for cfg_name in all_results:
        print(f"\n  --- {cfg_name} ---")
        header = f"{'Metric':35s}"
        for strat in ["committed", "power", "signal"]:
            header += f"  {strat:>15s}"
        print(header)
        print("-" * len(header))

        def row(metric_name, extract_fn, fmt="{:.2f}"):
            line = f"{metric_name:35s}"
            for strat in ["committed", "power", "signal"]:
                res = all_results[cfg_name][strat]
                val = extract_fn(res)
                line += f"  {fmt.format(val):>15s}"
            print(line)

        row("Late S/A", lambda r: r.late_sa)
        row("Deck concentration %", lambda r: r.deck_concentration, "{:.1f}%")
        row("Spend frequency (6+)", lambda r: r.spend_frequency, "{:.1%}")
        row("Genuine choice %", lambda r: r.genuine_choice_rate, "{:.1f}%")
        row("Convergence pick", lambda r: r.convergence_pick, "{:.1f}")


# ============================================================
# Main
# ============================================================

def pool_stats(pool: list[SimCard]) -> str:
    """Return a summary string of pool composition."""
    sym_counts = {0: 0, 1: 0, 2: 0, 3: 0}
    generic = 0
    bridge = 0
    for c in pool:
        sym_counts[len(c.symbols)] += 1
        if c.archetype_idx < 0:
            generic += 1
        if c.bridge_idx >= 0:
            bridge += 1
    return (f"  Total: {len(pool)} cards | Generic: {generic} | Bridge: {bridge}\n"
            f"  Symbols: 0-sym={sym_counts[0]}, 1-sym={sym_counts[1]}, "
            f"2-sym={sym_counts[2]}, 3-sym={sym_counts[3]}")


def main():
    random.seed(42)
    N_DRAFTS = 1200

    print("=" * 75)
    print("  V4 POOL SYNTHESIS SIMULATION")
    print("  Pack Widening v3 — Reconciled Pool Design")
    print("=" * 75)

    # Build configurations
    configs = [
        make_v4_default_config(),
        make_reconciled_config(),
        make_reconciled_alt_config(),
    ]

    strategies = ["committed", "power", "signal"]

    # Build pools and run
    all_results: dict[str, dict[str, BatchResults]] = {}

    for cfg in configs:
        print(f"\n--- Building pool: {cfg.name} ---")
        pool = build_pool(cfg)
        print(pool_stats(pool))
        primary_index = build_primary_index(pool)

        all_results[cfg.name] = {}
        for strat in strategies:
            print(f"  Running {cfg.name} / {strat} ({N_DRAFTS} drafts)...")
            res = run_batch(pool, primary_index, cfg, strat, N_DRAFTS)
            all_results[cfg.name][strat] = res

    # Print detailed results
    print("\n\n")
    print("#" * 75)
    print("  DETAILED RESULTS")
    print("#" * 75)

    for cfg_name in all_results:
        for strat in strategies:
            print_batch_results(all_results[cfg_name][strat])

    # Comparison tables
    print_comparison_table(all_results)
    print_strategy_comparison(all_results)

    # Three-act arc analysis for reconciled config
    print("\n\n")
    print("#" * 75)
    print("  THREE-ACT DRAFT ARC — RECONCILED (C5/B2/W3)")
    print("#" * 75)

    rec_res = all_results["Reconciled (C5/B2/W3)"]["committed"]
    print(f"\n  Act 1 — Exploration (picks 1-5):")
    print(f"    S/A per pack: {rec_res.sa_phase.get('p1_5', 0):.2f}")
    print(f"    First spend: pick {rec_res.first_spend:.1f}")
    print(f"    Unique archetypes per pack: {rec_res.early_diversity:.1f}")

    print(f"\n  Act 2 — Commitment (picks 6-15):")
    act2_sa = (rec_res.sa_phase.get('p6_10', 0) + rec_res.sa_phase.get('p11_15', 0)) / 2
    print(f"    S/A per pack: {act2_sa:.2f}")
    print(f"    Spend frequency: {rec_res.spend_frequency:.1%}")

    print(f"\n  Act 3 — Refinement (picks 16-30):")
    act3_sa = (rec_res.sa_phase.get('p16_20', 0) + rec_res.sa_phase.get('p21_25', 0) + rec_res.sa_phase.get('p26_30', 0)) / 3
    print(f"    S/A per pack: {act3_sa:.2f}")
    print(f"    Genuine choice rate: {rec_res.genuine_choice_rate:.1f}%")

    # Print fine-grained convergence curve for reconciled
    print(f"\n  --- Fine-grained convergence curve ---")
    picks_to_show = [1, 2, 3, 4, 5, 6, 8, 10, 12, 15, 18, 20, 25, 30]
    header = "    " + " ".join(f"P{p:>2d}" for p in picks_to_show)
    print(header)
    for cfg_name in all_results:
        res = all_results[cfg_name]["committed"]
        vals = " ".join(f"{res.sa_curve.get(p, 0):>4.1f}" for p in picks_to_show)
        print(f"    {cfg_name[:30]:30s} {vals}")

    print("\n\nDone.")


if __name__ == "__main__":
    main()
