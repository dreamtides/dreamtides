"""
Model D Simulation: Variety-First Draft System

N=8 archetypes, 2 suppressed per run, adaptive weighted sampling with depletion,
semi-explicit starting signal. Optimized for run-to-run variety and signal reading.
"""

import random
import math
from collections import defaultdict
from dataclasses import dataclass, field
from enum import Enum
from typing import Optional

# ---------------------------------------------------------------------------
# Constants
# ---------------------------------------------------------------------------

NUM_ARCHETYPES = 8
TOTAL_UNIQUE_CARDS = 360
PICKS_PER_DRAFT = 30
PACK_SIZE = 4
NUM_DRAFTS = 1000
SUPPRESSED_PER_RUN = 2
DEPLETION_CHANCE = 0.40  # chance each unpicked card is removed from pool

# Rarity distribution
RARITY_DIST = {"common": 0.55, "uncommon": 0.25, "rare": 0.15, "legendary": 0.05}
RARITY_COPIES = {"common": 4, "uncommon": 3, "rare": 2, "legendary": 1}

# Weight ramp for committed archetype (pick_number -> multiplier)
def get_weight_multiplier(pick_num: int) -> float:
    if pick_num <= 5:
        return 1.0
    elif pick_num <= 10:
        return 5.0
    elif pick_num <= 20:
        return 6.0
    else:
        return 7.0

# Fitness tier values for scoring
TIER_VALUES = {"S": 5, "A": 4, "B": 2, "C": 1, "F": 0}


# ---------------------------------------------------------------------------
# Data structures
# ---------------------------------------------------------------------------

class Rarity(Enum):
    COMMON = "common"
    UNCOMMON = "uncommon"
    RARE = "rare"
    LEGENDARY = "legendary"


@dataclass
class SimCard:
    id: int
    rarity: Rarity
    power: float  # raw card strength 0-10
    archetype_fitness: dict  # archetype_id (int) -> tier (str: S/A/B/C/F)

    def fitness_in(self, arch: int) -> str:
        return self.archetype_fitness.get(arch, "F")

    def is_fitting(self, arch: int) -> bool:
        """S or A tier in the given archetype."""
        return self.fitness_in(arch) in ("S", "A")

    def best_archetype(self) -> int:
        """Return the archetype with highest fitness (break ties by id)."""
        best_arch = 0
        best_val = -1
        for a, t in self.archetype_fitness.items():
            v = TIER_VALUES[t]
            if v > best_val:
                best_val = v
                best_arch = a
        return best_arch

    def fitness_score(self, arch: int) -> float:
        """Numeric fitness combining tier value and power."""
        return TIER_VALUES[self.fitness_in(arch)] + self.power * 0.1


# ---------------------------------------------------------------------------
# Card pool generation
# ---------------------------------------------------------------------------

# Archetype neighbor topology: each archetype has 2-3 neighbors
# (clustered overlap for multi-archetype stars and splash cards)
NEIGHBORS = {
    0: [1, 7],
    1: [0, 2],
    2: [1, 3],
    3: [2, 4],
    4: [3, 5],
    5: [4, 6],
    6: [5, 7],
    7: [6, 0],
}


def generate_card_pool(rng: random.Random) -> list[SimCard]:
    """Generate 360 unique cards with specified fitness distribution."""
    cards = []
    card_id = 0

    # Assign rarities
    rarities = []
    for rarity_name, frac in RARITY_DIST.items():
        count = round(TOTAL_UNIQUE_CARDS * frac)
        rarities.extend([Rarity(rarity_name)] * count)
    # Adjust to exactly 360
    while len(rarities) < TOTAL_UNIQUE_CARDS:
        rarities.append(Rarity.COMMON)
    while len(rarities) > TOTAL_UNIQUE_CARDS:
        rarities.pop()
    rng.shuffle(rarities)

    # Distribution targets
    num_narrow = 144       # 40% - S in 1, B in 1-2, C/F elsewhere
    num_splash = 108       # 30% - S in 1, A in 1-2, B in 1-2
    num_multi_star = 43    # 12% - S in 2, B in 2-3
    num_generalist = 47    # 13% - A in 2-3, B in 3-4, S in 0
    num_universal = 18     # 5%  - S in 3+, high power

    # Track S-tier assignments per archetype for balance
    s_tier_counts = defaultdict(int)

    def pick_primary_archetype() -> int:
        """Pick archetype with fewest S-tier cards so far."""
        min_count = min(s_tier_counts.get(a, 0) for a in range(NUM_ARCHETYPES))
        candidates = [a for a in range(NUM_ARCHETYPES)
                      if s_tier_counts.get(a, 0) == min_count]
        return rng.choice(candidates)

    def pick_neighbor(primary: int) -> int:
        return rng.choice(NEIGHBORS[primary])

    def pick_non_neighbor(primary: int) -> int:
        non_neighbors = [a for a in range(NUM_ARCHETYPES)
                         if a != primary and a not in NEIGHBORS[primary]]
        return rng.choice(non_neighbors) if non_neighbors else rng.choice(
            [a for a in range(NUM_ARCHETYPES) if a != primary])

    # 1. Narrow Specialists
    for i in range(num_narrow):
        primary = pick_primary_archetype()
        s_tier_counts[primary] += 1
        fitness = {}
        for a in range(NUM_ARCHETYPES):
            if a == primary:
                fitness[a] = "S"
            elif a in NEIGHBORS[primary] and rng.random() < 0.3:
                fitness[a] = "B"
            elif rng.random() < 0.15:
                fitness[a] = "C"
            else:
                fitness[a] = "F"
        power = rng.uniform(3.0, 6.0)
        cards.append(SimCard(card_id, rarities[card_id], power, fitness))
        card_id += 1

    # 2. Specialists with Splash
    for i in range(num_splash):
        primary = pick_primary_archetype()
        s_tier_counts[primary] += 1
        fitness = {}
        # A-tier in 1-2 neighbors
        a_targets = rng.sample(NEIGHBORS[primary],
                               k=min(rng.randint(1, 2), len(NEIGHBORS[primary])))
        for a in range(NUM_ARCHETYPES):
            if a == primary:
                fitness[a] = "S"
            elif a in a_targets:
                fitness[a] = "A"
            elif a in NEIGHBORS[primary] or rng.random() < 0.2:
                fitness[a] = "B"
            else:
                fitness[a] = rng.choice(["C", "F"])
        power = rng.uniform(4.0, 7.0)
        cards.append(SimCard(card_id, rarities[card_id], power, fitness))
        card_id += 1

    # 3. Multi-Archetype Stars
    for i in range(num_multi_star):
        primary = pick_primary_archetype()
        s_tier_counts[primary] += 1
        secondary = pick_neighbor(primary)
        s_tier_counts[secondary] += 1
        fitness = {}
        for a in range(NUM_ARCHETYPES):
            if a == primary or a == secondary:
                fitness[a] = "S"
            elif a in NEIGHBORS[primary] or a in NEIGHBORS[secondary]:
                fitness[a] = "B" if rng.random() < 0.5 else "C"
            else:
                fitness[a] = rng.choice(["C", "F"])
        power = rng.uniform(5.0, 8.0)
        cards.append(SimCard(card_id, rarities[card_id], power, fitness))
        card_id += 1

    # 4. Broad Generalists
    for i in range(num_generalist):
        fitness = {}
        a_count = rng.randint(2, 3)
        b_count = rng.randint(3, 4)
        a_targets = rng.sample(range(NUM_ARCHETYPES), k=a_count)
        remaining = [a for a in range(NUM_ARCHETYPES) if a not in a_targets]
        b_targets = rng.sample(remaining, k=min(b_count, len(remaining)))
        for a in range(NUM_ARCHETYPES):
            if a in a_targets:
                fitness[a] = "A"
            elif a in b_targets:
                fitness[a] = "B"
            else:
                fitness[a] = "C"
        power = rng.uniform(5.0, 7.5)
        cards.append(SimCard(card_id, rarities[card_id], power, fitness))
        card_id += 1

    # 5. Universal Stars
    for i in range(num_universal):
        fitness = {}
        s_count = rng.randint(3, 5)
        s_targets = rng.sample(range(NUM_ARCHETYPES), k=s_count)
        for a in range(NUM_ARCHETYPES):
            if a in s_targets:
                fitness[a] = "S"
                s_tier_counts[a] += 1
            else:
                fitness[a] = rng.choice(["A", "B"])
        power = rng.uniform(7.0, 10.0)
        cards.append(SimCard(card_id, rarities[card_id], power, fitness))
        card_id += 1

    return cards


def build_pool(cards: list[SimCard]) -> list[SimCard]:
    """Expand unique cards into pool entries based on rarity copy counts."""
    pool = []
    for card in cards:
        copies = RARITY_COPIES[card.rarity.value]
        for _ in range(copies):
            pool.append(card)
    return pool


# ---------------------------------------------------------------------------
# Draft simulation
# ---------------------------------------------------------------------------

@dataclass
class DraftState:
    pool: list[SimCard]
    picked: list[SimCard] = field(default_factory=list)
    committed_archetype: Optional[int] = None
    archetype_counts: dict = field(default_factory=lambda: defaultdict(int))
    active_archetypes: list[int] = field(default_factory=list)
    starting_card: Optional[SimCard] = None
    # Tracking
    packs_seen: list[list[SimCard]] = field(default_factory=list)


def detect_commitment(state: DraftState) -> Optional[int]:
    """Detect if player has committed (3+ S/A cards in one archetype)."""
    for arch, count in state.archetype_counts.items():
        if count >= 3 and arch in state.active_archetypes:
            return arch
    return None


def construct_pack(state: DraftState, pick_num: int,
                   rng: random.Random) -> list[SimCard]:
    """Construct a 4-card pack using adaptive weighted sampling."""
    if len(state.pool) < PACK_SIZE:
        return list(state.pool)

    weights = []
    committed = state.committed_archetype
    multiplier = get_weight_multiplier(pick_num)

    for card in state.pool:
        w = 1.0
        if committed is not None and pick_num > 5:
            if card.is_fitting(committed):
                w *= multiplier
        weights.append(w)

    # If committed, use 3 weighted archetype slots + 1 splash slot
    if committed is not None and pick_num > 5:
        pack = []
        pool_indices = list(range(len(state.pool)))

        # Draw 3 weighted cards (archetype-biased)
        for _ in range(3):
            if not pool_indices:
                break
            idx_weights = [(i, weights[i]) for i in pool_indices]
            total_w = sum(w for _, w in idx_weights)
            if total_w == 0:
                break
            r = rng.uniform(0, total_w)
            cumulative = 0
            chosen_idx = pool_indices[0]
            for idx, w in idx_weights:
                cumulative += w
                if r <= cumulative:
                    chosen_idx = idx
                    break
            pack.append(state.pool[chosen_idx])
            pool_indices.remove(chosen_idx)

        # Draw 1 splash card: prefer high power or S-tier in other archetype
        if pool_indices:
            splash_candidates = [i for i in pool_indices
                                 if not state.pool[i].is_fitting(committed)]
            if not splash_candidates:
                splash_candidates = pool_indices[:]

            splash_weights = []
            for i in splash_candidates:
                c = state.pool[i]
                sw = c.power  # base weight is raw power
                # Bonus if S-tier in any other active archetype
                for a in state.active_archetypes:
                    if a != committed and c.fitness_in(a) == "S":
                        sw += 5.0
                        break
                splash_weights.append(sw)
            total_sw = sum(splash_weights)
            if total_sw > 0:
                r = rng.uniform(0, total_sw)
                cumulative = 0
                chosen = splash_candidates[0]
                for i, sw in zip(splash_candidates, splash_weights):
                    cumulative += sw
                    if r <= cumulative:
                        chosen = i
                        break
                pack.append(state.pool[chosen])
            elif pool_indices:
                pack.append(state.pool[rng.choice(pool_indices)])

        return pack
    else:
        # Uniform random for early picks
        indices = rng.sample(range(len(state.pool)), k=min(PACK_SIZE, len(state.pool)))
        return [state.pool[i] for i in indices]


def apply_depletion(state: DraftState, pack: list[SimCard],
                    picked_card: SimCard, rng: random.Random):
    """Remove picked card and probabilistically remove unpicked cards."""
    # Remove all copies of picked card from pool
    state.pool = [c for c in state.pool if c.id != picked_card.id]

    # Probabilistically remove unpicked cards
    unpicked_ids = {c.id for c in pack if c.id != picked_card.id}
    for uid in unpicked_ids:
        if rng.random() < DEPLETION_CHANCE:
            # Remove one copy of this card
            for i, c in enumerate(state.pool):
                if c.id == uid:
                    state.pool.pop(i)
                    break


# ---------------------------------------------------------------------------
# Player strategies
# ---------------------------------------------------------------------------

def strategy_committed(pack: list[SimCard], state: DraftState) -> SimCard:
    """Pick highest fitness in committed archetype; before commitment, pick
    highest single-archetype fitness among active archetypes."""
    if state.committed_archetype is not None:
        arch = state.committed_archetype
        return max(pack, key=lambda c: c.fitness_score(arch))
    else:
        # Pick the card with highest fitness in any single active archetype
        def best_active_score(c: SimCard) -> float:
            return max(c.fitness_score(a) for a in state.active_archetypes)
        return max(pack, key=best_active_score)


def strategy_power_chaser(pack: list[SimCard], state: DraftState) -> SimCard:
    """Pick highest raw power card regardless of archetype."""
    return max(pack, key=lambda c: c.power)


def strategy_signal_reader(pack: list[SimCard], state: DraftState) -> SimCard:
    """Track which archetypes appear most frequently and draft toward the open one."""
    # Count archetype appearances across all packs seen
    arch_freq = defaultdict(int)
    for prev_pack in state.packs_seen:
        for c in prev_pack:
            for a in state.active_archetypes:
                if c.is_fitting(a):
                    arch_freq[a] += 1

    # Also count current pack
    for c in pack:
        for a in state.active_archetypes:
            if c.is_fitting(a):
                arch_freq[a] += 1

    if not arch_freq:
        return max(pack, key=lambda c: c.power)

    # If committed, use committed archetype
    if state.committed_archetype is not None:
        target_arch = state.committed_archetype
    else:
        # Target the most-seen active archetype (it's most "open"/available)
        target_arch = max(arch_freq, key=lambda a: arch_freq[a])

    return max(pack, key=lambda c: c.fitness_score(target_arch))


STRATEGIES = {
    "committed": strategy_committed,
    "power_chaser": strategy_power_chaser,
    "signal_reader": strategy_signal_reader,
}


# ---------------------------------------------------------------------------
# Run a single draft
# ---------------------------------------------------------------------------

def run_draft(cards: list[SimCard], strategy_name: str,
              rng: random.Random, trace: bool = False) -> dict:
    """Run one complete draft and return metrics."""
    pool = build_pool(cards)
    rng.shuffle(pool)

    # Suppress 2 random archetypes
    suppressed = rng.sample(range(NUM_ARCHETYPES), k=SUPPRESSED_PER_RUN)
    active = [a for a in range(NUM_ARCHETYPES) if a not in suppressed]

    # Remove ~60% of copies of suppressed archetype S-tier cards
    # (they still exist but are rarer)
    new_pool = []
    for card in pool:
        dominated_by_suppressed = all(
            card.fitness_in(a) in ("S",) for a in suppressed
            if card.fitness_in(a) == "S"
        ) and any(card.fitness_in(a) == "S" for a in suppressed)
        # Only suppress cards whose PRIMARY archetype is suppressed
        primary = card.best_archetype()
        if primary in suppressed:
            if rng.random() < 0.5:  # Remove 50% of copies
                continue
        new_pool.append(card)
    pool = new_pool

    state = DraftState(
        pool=pool,
        active_archetypes=active,
    )

    # Starting signal: show 3 cards biased toward multi-archetype stars in active archetypes
    starting_candidates = [c for c in state.pool
                           if any(c.fitness_in(a) == "S" for a in active)]
    if len(starting_candidates) >= 3:
        starting_options = rng.sample(starting_candidates, k=3)
    else:
        starting_options = rng.sample(state.pool, k=min(3, len(state.pool)))

    # Player keeps the card with highest fitness in any active archetype
    strategy_fn = STRATEGIES[strategy_name]
    state.starting_card = max(starting_options,
                              key=lambda c: max(c.fitness_score(a) for a in active))

    # Track the starting card as an initial pick for archetype detection
    for a in active:
        if state.starting_card.is_fitting(a):
            state.archetype_counts[a] += 1

    # Draft loop
    trace_log = []
    metrics = {
        "early_unique_archetypes": [],  # picks 1-5
        "early_fitting_per_pack": [],   # picks 1-5
        "late_fitting_per_pack": [],    # picks 6+
        "late_off_archetype": [],       # picks 6+
        "convergence_pick": None,
        "deck_cards": [],
        "suppressed": suppressed,
        "active": active,
    }

    for pick_num in range(1, PICKS_PER_DRAFT + 1):
        pack = construct_pack(state, pick_num, rng)
        if len(pack) == 0:
            break

        state.packs_seen.append(pack)

        # Measure early metrics (picks 1-5)
        if pick_num <= 5:
            # Unique archetypes represented (S or A tier in active archetypes)
            archs_in_pack = set()
            for c in pack:
                for a in active:
                    if c.is_fitting(a):
                        archs_in_pack.add(a)
            metrics["early_unique_archetypes"].append(len(archs_in_pack))

            # If player has emerging archetype, count fitting cards
            if state.committed_archetype is not None:
                fitting = sum(1 for c in pack
                              if c.is_fitting(state.committed_archetype))
                metrics["early_fitting_per_pack"].append(fitting)
            elif state.archetype_counts:
                # Use the leading archetype as "emerging"
                emerging = max(state.archetype_counts,
                               key=lambda a: state.archetype_counts[a])
                fitting = sum(1 for c in pack if c.is_fitting(emerging))
                metrics["early_fitting_per_pack"].append(fitting)

        # Measure late metrics (picks 6+)
        if pick_num >= 6 and state.committed_archetype is not None:
            arch = state.committed_archetype
            fitting = sum(1 for c in pack if c.is_fitting(arch))
            metrics["late_fitting_per_pack"].append(fitting)

            # Strong off-archetype: S-tier in different active archetype or power >= 7
            off_arch = sum(1 for c in pack
                           if not c.is_fitting(arch) and
                           (c.power >= 7.0 or
                            any(c.fitness_in(a) == "S"
                                for a in active if a != arch)))
            metrics["late_off_archetype"].append(off_arch)

        # Convergence detection
        if (state.committed_archetype is not None and
                metrics["convergence_pick"] is None):
            arch = state.committed_archetype
            fitting = sum(1 for c in pack if c.is_fitting(arch))
            if fitting >= 2:
                metrics["convergence_pick"] = pick_num

        # Player picks
        picked = strategy_fn(pack, state)
        state.picked.append(picked)
        metrics["deck_cards"].append(picked)

        if trace:
            trace_log.append({
                "pick": pick_num,
                "pack": [(c.id, c.fitness_in(state.committed_archetype
                          if state.committed_archetype is not None
                          else (max(state.archetype_counts,
                                    key=lambda a: state.archetype_counts[a])
                                if state.archetype_counts else 0)),
                          round(c.power, 1))
                         for c in pack],
                "picked_id": picked.id,
                "committed": state.committed_archetype,
            })

        # Update archetype counts
        for a in active:
            if picked.is_fitting(a):
                state.archetype_counts[a] += 1

        # Check commitment
        if state.committed_archetype is None:
            state.committed_archetype = detect_commitment(state)

        # Apply depletion
        apply_depletion(state, pack, picked, rng)

    metrics["trace"] = trace_log if trace else None

    # Deck archetype concentration
    if state.committed_archetype is not None:
        arch = state.committed_archetype
        sa_count = sum(1 for c in state.picked if c.is_fitting(arch))
        metrics["deck_concentration"] = sa_count / len(state.picked) if state.picked else 0
    else:
        metrics["deck_concentration"] = 0

    metrics["committed_archetype"] = state.committed_archetype
    metrics["picked_ids"] = set(c.id for c in state.picked)

    return metrics


# ---------------------------------------------------------------------------
# Batch simulation
# ---------------------------------------------------------------------------

def run_simulation(multi_arch_pct: Optional[float] = None,
                   num_drafts: int = NUM_DRAFTS,
                   trace_count: int = 3) -> dict:
    """Run full simulation batch. If multi_arch_pct is set, adjust the
    card pool's multi-archetype card percentage for sensitivity analysis."""

    master_rng = random.Random(42)
    cards = generate_card_pool(master_rng)

    # If doing sensitivity analysis, modify multi-archetype fraction
    if multi_arch_pct is not None:
        cards = adjust_multi_archetype(cards, multi_arch_pct, master_rng)

    all_results = {}

    for strategy_name in STRATEGIES:
        results = {
            "early_unique_archs": [],
            "early_fitting": [],
            "late_fitting": [],
            "late_off_arch": [],
            "convergence_picks": [],
            "deck_concentrations": [],
            "picked_id_sets": [],
            "committed_archetypes": [],
            "traces": [],
        }

        for draft_i in range(num_drafts):
            draft_rng = random.Random(master_rng.randint(0, 2**32))
            do_trace = draft_i < trace_count and strategy_name == "committed"

            m = run_draft(cards, strategy_name, draft_rng, trace=do_trace)

            if m["early_unique_archetypes"]:
                results["early_unique_archs"].extend(m["early_unique_archetypes"])
            if m["early_fitting_per_pack"]:
                results["early_fitting"].extend(m["early_fitting_per_pack"])
            if m["late_fitting_per_pack"]:
                results["late_fitting"].extend(m["late_fitting_per_pack"])
            if m["late_off_archetype"]:
                results["late_off_arch"].extend(m["late_off_archetype"])
            if m["convergence_pick"] is not None:
                results["convergence_picks"].append(m["convergence_pick"])
            results["deck_concentrations"].append(m["deck_concentration"])
            results["picked_id_sets"].append(m["picked_ids"])
            if m["committed_archetype"] is not None:
                results["committed_archetypes"].append(m["committed_archetype"])
            if m["trace"]:
                results["traces"].append(m["trace"])

        all_results[strategy_name] = results

    return all_results


def adjust_multi_archetype(cards: list[SimCard], target_pct: float,
                           rng: random.Random) -> list[SimCard]:
    """Adjust the pool so that target_pct of cards are S or A in 2+ archetypes."""
    # Count current multi-archetype cards
    def is_multi(c: SimCard) -> bool:
        sa_count = sum(1 for t in c.archetype_fitness.values() if t in ("S", "A"))
        return sa_count >= 2

    current_multi = [c for c in cards if is_multi(c)]
    current_single = [c for c in cards if not is_multi(c)]
    current_pct = len(current_multi) / len(cards)
    target_count = int(len(cards) * target_pct)

    if target_count > len(current_multi):
        # Need to convert some single-archetype cards to multi
        to_convert = target_count - len(current_multi)
        convert_candidates = rng.sample(current_single,
                                        k=min(to_convert, len(current_single)))
        for card in convert_candidates:
            # Find primary S-tier archetype
            primary = None
            for a, t in card.archetype_fitness.items():
                if t == "S":
                    primary = a
                    break
            if primary is not None:
                # Add A-tier in a neighbor
                neighbor = rng.choice(NEIGHBORS.get(primary, [primary]))
                card.archetype_fitness[neighbor] = "A"
            else:
                # Card has no S-tier; upgrade a B to A
                for a, t in card.archetype_fitness.items():
                    if t == "B":
                        card.archetype_fitness[a] = "A"
                        break
    elif target_count < len(current_multi):
        # Need to downgrade some multi-archetype cards
        to_downgrade = len(current_multi) - target_count
        downgrade_candidates = rng.sample(current_multi,
                                          k=min(to_downgrade, len(current_multi)))
        for card in downgrade_candidates:
            # Downgrade all A-tier to B-tier
            for a in list(card.archetype_fitness.keys()):
                if card.archetype_fitness[a] == "A":
                    card.archetype_fitness[a] = "B"

    return cards


# ---------------------------------------------------------------------------
# Analysis and reporting
# ---------------------------------------------------------------------------

def compute_metrics(all_results: dict) -> dict:
    """Compute the 8 target metrics from simulation results."""
    metrics = {}

    # Use committed strategy for most metrics
    committed = all_results["committed"]

    # 1. Picks 1-5: unique archetypes per pack
    if committed["early_unique_archs"]:
        metrics["early_unique_archs"] = (
            sum(committed["early_unique_archs"]) /
            len(committed["early_unique_archs"])
        )
    else:
        metrics["early_unique_archs"] = 0

    # 2. Picks 1-5: fitting cards per pack (emerging archetype)
    if committed["early_fitting"]:
        metrics["early_fitting"] = (
            sum(committed["early_fitting"]) /
            len(committed["early_fitting"])
        )
    else:
        metrics["early_fitting"] = 0

    # 3. Picks 6+: fitting cards per pack
    if committed["late_fitting"]:
        metrics["late_fitting"] = (
            sum(committed["late_fitting"]) /
            len(committed["late_fitting"])
        )
    else:
        metrics["late_fitting"] = 0

    # 4. Picks 6+: off-archetype strong cards per pack
    if committed["late_off_arch"]:
        metrics["late_off_arch"] = (
            sum(committed["late_off_arch"]) /
            len(committed["late_off_arch"])
        )
    else:
        metrics["late_off_arch"] = 0

    # 5. Convergence pick
    if committed["convergence_picks"]:
        metrics["convergence_pick"] = (
            sum(committed["convergence_picks"]) /
            len(committed["convergence_picks"])
        )
    else:
        metrics["convergence_pick"] = float("inf")

    # 6. Deck concentration
    concs = [c for c in committed["deck_concentrations"] if c > 0]
    if concs:
        metrics["deck_concentration"] = sum(concs) / len(concs)
    else:
        metrics["deck_concentration"] = 0

    # 7. Run-to-run variety (card overlap between runs with same seed structure)
    overlaps = []
    id_sets = committed["picked_id_sets"]
    sample_size = min(200, len(id_sets))
    sample_indices = list(range(sample_size))
    for i in range(0, sample_size - 1, 2):
        if i + 1 < sample_size:
            s1 = id_sets[i]
            s2 = id_sets[i + 1]
            if s1 and s2:
                overlap = len(s1 & s2) / max(len(s1 | s2), 1)
                overlaps.append(overlap)
    metrics["card_overlap"] = sum(overlaps) / len(overlaps) if overlaps else 0

    # 8. Archetype frequency
    arch_counts = defaultdict(int)
    total = len(committed["committed_archetypes"])
    for a in committed["committed_archetypes"]:
        arch_counts[a] += 1
    if total > 0:
        metrics["arch_freq_max"] = max(arch_counts.values()) / total
        metrics["arch_freq_min"] = min(arch_counts.values()) / total if arch_counts else 0
    else:
        metrics["arch_freq_max"] = 0
        metrics["arch_freq_min"] = 0

    # Also compute for all strategies
    for strat_name, strat_results in all_results.items():
        prefix = strat_name + "_"
        if strat_results["late_fitting"]:
            metrics[prefix + "late_fitting"] = (
                sum(strat_results["late_fitting"]) /
                len(strat_results["late_fitting"])
            )
        if strat_results["deck_concentrations"]:
            concs = [c for c in strat_results["deck_concentrations"] if c > 0]
            if concs:
                metrics[prefix + "deck_conc"] = sum(concs) / len(concs)

    return metrics


def print_scorecard(metrics: dict):
    """Print the target scorecard table."""
    print("\n" + "=" * 72)
    print("TARGET SCORECARD")
    print("=" * 72)

    targets = [
        ("Picks 1-5: unique archs per pack", ">= 3", metrics.get("early_unique_archs", 0),
         metrics.get("early_unique_archs", 0) >= 3),
        ("Picks 1-5: fitting cards per pack", "<= 2", metrics.get("early_fitting", 0),
         metrics.get("early_fitting", 0) <= 2),
        ("Picks 6+: fitting cards per pack", ">= 2", metrics.get("late_fitting", 0),
         metrics.get("late_fitting", 0) >= 2),
        ("Picks 6+: off-archetype per pack", ">= 0.5", metrics.get("late_off_arch", 0),
         metrics.get("late_off_arch", 0) >= 0.5),
        ("Convergence pick", "5-8", metrics.get("convergence_pick", 0),
         5 <= metrics.get("convergence_pick", 0) <= 8),
        ("Deck concentration", "60-80%",
         f"{metrics.get('deck_concentration', 0):.1%}",
         0.6 <= metrics.get("deck_concentration", 0) <= 0.8),
        ("Run-to-run card overlap", "< 40%",
         f"{metrics.get('card_overlap', 0):.1%}",
         metrics.get("card_overlap", 0) < 0.4),
        ("Archetype freq max", "<= 20%",
         f"{metrics.get('arch_freq_max', 0):.1%}",
         metrics.get("arch_freq_max", 0) <= 0.20),
        ("Archetype freq min", ">= 5%",
         f"{metrics.get('arch_freq_min', 0):.1%}",
         metrics.get("arch_freq_min", 0) >= 0.05),
    ]

    print(f"{'Metric':<45} {'Target':<10} {'Actual':<10} {'Pass?':<6}")
    print("-" * 72)
    for name, target, actual, passed in targets:
        actual_str = f"{actual:.2f}" if isinstance(actual, float) else str(actual)
        status = "PASS" if passed else "FAIL"
        print(f"{name:<45} {target:<10} {actual_str:<10} {status:<6}")

    pass_count = sum(1 for _, _, _, p in targets if p)
    print(f"\nPassed: {pass_count}/{len(targets)}")


def print_traces(all_results: dict):
    """Print detailed draft traces."""
    traces = all_results["committed"]["traces"]
    for i, trace in enumerate(traces[:3]):
        print(f"\n{'=' * 60}")
        print(f"DRAFT TRACE {i + 1} (committed strategy)")
        print(f"{'=' * 60}")
        for entry in trace:
            pick_num = entry["pick"]
            pack_info = entry["pack"]
            picked_id = entry["picked_id"]
            committed = entry["committed"]
            pack_str = " | ".join(
                f"[{'*' if cid == picked_id else ' '}] id={cid} fit={fit} pwr={pwr}"
                for cid, fit, pwr in pack_info
            )
            print(f"Pick {pick_num:2d} (arch={committed}): {pack_str}")


def run_sensitivity_analysis():
    """Vary multi-archetype card percentage and measure impact."""
    print("\n" + "=" * 72)
    print("MULTI-ARCHETYPE CARD SENSITIVITY ANALYSIS")
    print("=" * 72)

    pcts = [0.10, 0.20, 0.30, 0.42, 0.50]
    print(f"{'Multi-Arch %':<15} {'Late Fitting':<15} {'Deck Conc':<12} "
          f"{'Card Overlap':<15} {'Conv Pick':<12}")
    print("-" * 72)

    for pct in pcts:
        results = run_simulation(multi_arch_pct=pct, num_drafts=200, trace_count=0)
        m = compute_metrics(results)
        print(f"{pct:<15.0%} {m.get('late_fitting', 0):<15.2f} "
              f"{m.get('deck_concentration', 0):<12.1%} "
              f"{m.get('card_overlap', 0):<15.1%} "
              f"{m.get('convergence_pick', float('inf')):<12.1f}")


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

if __name__ == "__main__":
    print("Running Model D simulation (N=8, variety-first)...")
    print(f"  {NUM_DRAFTS} drafts, {PICKS_PER_DRAFT} picks each, "
          f"{PACK_SIZE} cards per pack")
    print(f"  {SUPPRESSED_PER_RUN} archetypes suppressed per run")
    print(f"  Depletion chance: {DEPLETION_CHANCE:.0%}")
    print()

    results = run_simulation()
    metrics = compute_metrics(results)

    print_scorecard(metrics)
    print_traces(results)

    # Per-strategy breakdown
    print("\n" + "=" * 72)
    print("PER-STRATEGY BREAKDOWN")
    print("=" * 72)
    for strat in STRATEGIES:
        prefix = strat + "_"
        late_fit = metrics.get(prefix + "late_fitting", "N/A")
        deck_conc = metrics.get(prefix + "deck_conc", "N/A")
        late_str = f"{late_fit:.2f}" if isinstance(late_fit, float) else str(late_fit)
        conc_str = f"{deck_conc:.1%}" if isinstance(deck_conc, float) else str(deck_conc)
        print(f"  {strat:<20} late_fitting={late_str:<8} deck_conc={conc_str}")

    run_sensitivity_analysis()

    print("\nSimulation complete.")
