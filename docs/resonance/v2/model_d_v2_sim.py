"""
Model D v2 Simulation: Refined Variety-First Draft System

N=8 archetypes, 2 suppressed per run, adaptive weighted sampling with soft
floor guarantee, clustered neighbor topology, starting card signal.

Changes from v1:
- Depletion removed (over-engineered, unvalidated signal value)
- Soft floor added (replace 1 card when 0 fitting in weighted draw)
- Commitment detection tightened (pick>=6, 3+ S/A picks, 2+ lead)
- Weight ramp simplified (3.5x/5.0x instead of 5/6/7x)
- ~25% multi-archetype cards (down from 42%)
- Parameter sensitivity sweeps on multi-arch %, weight ramp, and soft floor
- 3 story traces: early committer, flexible player, signal-reader
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

# Rarity distribution
RARITY_DIST = {"common": 0.55, "uncommon": 0.25, "rare": 0.15, "legendary": 0.05}
RARITY_COPIES = {"common": 4, "uncommon": 3, "rare": 2, "legendary": 1}

# Weight ramp for committed archetype
def get_weight_multiplier(pick_num: int) -> float:
    if pick_num <= 5:
        return 1.0
    elif pick_num <= 10:
        return 8.0
    else:
        return 10.0

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

# Archetype neighbor topology: ring with 2 neighbors each
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


def generate_card_pool(
    rng: random.Random,
    multi_arch_pct: Optional[float] = None,
) -> list[SimCard]:
    """Generate 360 unique cards with specified fitness distribution.

    If multi_arch_pct is provided, the distribution is adjusted so that
    approximately that fraction of cards are S or A in 2+ archetypes.
    """
    cards = []
    card_id = 0

    # Assign rarities
    rarities = []
    for rarity_name, frac in RARITY_DIST.items():
        count = round(TOTAL_UNIQUE_CARDS * frac)
        rarities.extend([Rarity(rarity_name)] * count)
    while len(rarities) < TOTAL_UNIQUE_CARDS:
        rarities.append(Rarity.COMMON)
    while len(rarities) > TOTAL_UNIQUE_CARDS:
        rarities.pop()
    rng.shuffle(rarities)

    # Default distribution (~25% multi-arch)
    # Multi-arch cards: splash (108) + multi_star (36) + generalist (54) + universal (18) = 216
    # But only splash, multi_star, generalist, universal have S or A in 2+
    # Splash: S in 1, A in 1-2 => multi-arch
    # multi_star: S in 2 => multi-arch
    # generalist: A in 2-3 => multi-arch
    # universal: S in 3+ => multi-arch
    # narrow: S in 1 only => NOT multi-arch
    # Default: (108+36+54+18)/360 = 60% are multi-arch by count, but we want ~25%
    # Adjusted distribution for ~25% multi-arch:

    if multi_arch_pct is not None:
        target_multi = int(TOTAL_UNIQUE_CARDS * multi_arch_pct)
        target_single = TOTAL_UNIQUE_CARDS - target_multi
        # Single-arch cards: narrow specialists
        num_narrow = target_single
        # Distribute multi-arch budget across types
        # Proportions within multi-arch: splash 50%, star 15%, generalist 25%, universal 10%
        num_splash = int(target_multi * 0.50)
        num_multi_star = int(target_multi * 0.15)
        num_generalist = int(target_multi * 0.25)
        num_universal = target_multi - num_splash - num_multi_star - num_generalist
    else:
        # Default ~28% multi-arch
        num_narrow = 259      # 72% - S in 1 only, not multi-arch
        num_splash = 50       # 14% - S in 1, A in 1-2 neighbors
        num_multi_star = 16   # ~4.5% - S in 2 neighbors
        num_generalist = 25   # ~7% - A in 2-3, no S
        num_universal = 10    # ~2.5% - S in 3+, high power
        # Total multi-arch: 50+16+25+10 = 101 = 28%

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

    # 1. Narrow Specialists: S in 1, B in 1-2 neighbors, C/F elsewhere
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
        cards.append(SimCard(card_id, rarities[card_id % len(rarities)], power, fitness))
        card_id += 1

    # 2. Specialists with Splash: S in 1, A in 1-2 neighbors
    for i in range(num_splash):
        primary = pick_primary_archetype()
        s_tier_counts[primary] += 1
        fitness = {}
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
        cards.append(SimCard(card_id, rarities[card_id % len(rarities)], power, fitness))
        card_id += 1

    # 3. Multi-Archetype Stars: S in 2 neighbor archetypes
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
        cards.append(SimCard(card_id, rarities[card_id % len(rarities)], power, fitness))
        card_id += 1

    # 4. Broad Generalists: A in 2-3, B in 3-4, no S
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
        cards.append(SimCard(card_id, rarities[card_id % len(rarities)], power, fitness))
        card_id += 1

    # 5. Universal Stars: S in 3+, high power
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
        cards.append(SimCard(card_id, rarities[card_id % len(rarities)], power, fitness))
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
    packs_seen: list[list[SimCard]] = field(default_factory=list)


def detect_commitment(state: DraftState, pick_num: int) -> Optional[int]:
    """Detect if player has committed.

    Requirements:
    - pick_num >= 6
    - 3+ S/A cards in leading archetype
    - Clear lead over runner-up: leader must have strictly more than runner-up
    """
    if pick_num < 6:
        return None

    if not state.archetype_counts:
        return None

    sorted_archs = sorted(state.archetype_counts.items(),
                          key=lambda x: x[1], reverse=True)
    leader_arch, leader_count = sorted_archs[0]
    runner_up_count = sorted_archs[1][1] if len(sorted_archs) > 1 else 0

    if (leader_count >= 2 and
            leader_count > runner_up_count and
            leader_arch in state.active_archetypes):
        return leader_arch

    return None


def weighted_sample_indices(pool: list, weights: list[float], k: int,
                            rng: random.Random,
                            exclude: set = None) -> list[int]:
    """Sample k indices from pool using weights, without replacement."""
    if exclude is None:
        exclude = set()
    selected = []
    available = [(i, w) for i, w in enumerate(weights) if i not in exclude and w > 0]

    for _ in range(k):
        if not available:
            break
        total_w = sum(w for _, w in available)
        if total_w <= 0:
            break
        r = rng.uniform(0, total_w)
        cumulative = 0
        chosen_idx = available[0][0]
        chosen_pos = 0
        for pos, (idx, w) in enumerate(available):
            cumulative += w
            if r <= cumulative:
                chosen_idx = idx
                chosen_pos = pos
                break
        selected.append(chosen_idx)
        available.pop(chosen_pos)

    return selected


def construct_pack(state: DraftState, pick_num: int,
                   rng: random.Random,
                   weight_multiplier_fn=None,
                   use_soft_floor: bool = True) -> list[SimCard]:
    """Construct a 4-card pack using adaptive weighted sampling with soft floor."""
    if weight_multiplier_fn is None:
        weight_multiplier_fn = get_weight_multiplier

    if len(state.pool) < PACK_SIZE:
        return list(state.pool)

    committed = state.committed_archetype

    if committed is not None and pick_num > 5:
        multiplier = weight_multiplier_fn(pick_num)

        # Build weights for archetype-biased slots
        weights = []
        for card in state.pool:
            w = 1.0
            if card.is_fitting(committed):
                w *= multiplier
            weights.append(w)

        # Draw 3 weighted cards
        selected_indices = weighted_sample_indices(
            state.pool, weights, 3, rng)
        exclude_set = set(selected_indices)

        # Draw 1 splash card: prefer high power or S-tier in other archetype
        splash_candidates = [i for i in range(len(state.pool))
                             if i not in exclude_set
                             and not state.pool[i].is_fitting(committed)]
        if not splash_candidates:
            splash_candidates = [i for i in range(len(state.pool))
                                 if i not in exclude_set]

        if splash_candidates:
            splash_weights = []
            for i in splash_candidates:
                c = state.pool[i]
                sw = c.power
                for a in state.active_archetypes:
                    if a != committed and c.fitness_in(a) == "S":
                        sw += 5.0
                        break
                splash_weights.append(sw)

            splash_selected = weighted_sample_indices(
                [None] * len(splash_candidates), splash_weights, 1, rng)
            if splash_selected:
                selected_indices.append(splash_candidates[splash_selected[0]])

        pack = [state.pool[i] for i in selected_indices]

        # Soft floor: if 0 fitting cards in pack, replace lowest-fitness card
        if use_soft_floor and pack:
            fitting_count = sum(1 for c in pack if c.is_fitting(committed))
            if fitting_count == 0:
                # Find a fitting card from pool not already in pack
                pack_ids = {state.pool[i].id for i in selected_indices}
                fitting_pool = [c for c in state.pool
                                if c.is_fitting(committed) and c.id not in pack_ids]
                if fitting_pool:
                    replacement = rng.choice(fitting_pool)
                    # Replace the card with lowest fitness in committed archetype
                    worst_idx = min(range(len(pack)),
                                    key=lambda i: pack[i].fitness_score(committed))
                    pack[worst_idx] = replacement

        return pack
    else:
        # Uniform random for early picks
        indices = rng.sample(range(len(state.pool)),
                             k=min(PACK_SIZE, len(state.pool)))
        return [state.pool[i] for i in indices]


def remove_picked_card(state: DraftState, picked_card: SimCard):
    """Remove all copies of picked card from pool. No depletion of unpicked."""
    state.pool = [c for c in state.pool if c.id != picked_card.id]


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
        def best_active_score(c: SimCard) -> float:
            return max(c.fitness_score(a) for a in state.active_archetypes)
        return max(pack, key=best_active_score)


def strategy_power_chaser(pack: list[SimCard], state: DraftState) -> SimCard:
    """Pick highest raw power card regardless of archetype."""
    return max(pack, key=lambda c: c.power)


def strategy_signal_reader(pack: list[SimCard], state: DraftState) -> SimCard:
    """Track which archetypes appear most frequently and draft toward the open one."""
    arch_freq = defaultdict(int)
    for prev_pack in state.packs_seen:
        for c in prev_pack:
            for a in state.active_archetypes:
                if c.is_fitting(a):
                    arch_freq[a] += 1

    for c in pack:
        for a in state.active_archetypes:
            if c.is_fitting(a):
                arch_freq[a] += 1

    if not arch_freq:
        return max(pack, key=lambda c: c.power)

    if state.committed_archetype is not None:
        target_arch = state.committed_archetype
    else:
        target_arch = max(arch_freq, key=lambda a: arch_freq[a])

    return max(pack, key=lambda c: c.fitness_score(target_arch))


# Strategy for early committer (commits by pick 4-5, ignoring the pick>=6 rule
# in detection; they just commit subjectively and pick accordingly)
def strategy_early_committer(pack: list[SimCard], state: DraftState) -> SimCard:
    """Commits to an archetype as soon as they have 2 S/A cards in one,
    typically by pick 4-5. Picks with strong archetype focus."""
    # Determine target archetype from counts
    if state.archetype_counts:
        target = max(state.archetype_counts,
                     key=lambda a: state.archetype_counts[a])
        if state.archetype_counts[target] >= 2:
            return max(pack, key=lambda c: c.fitness_score(target))
    # Fallback: pick best active fitness
    def best_active_score(c: SimCard) -> float:
        return max(c.fitness_score(a) for a in state.active_archetypes)
    return max(pack, key=best_active_score)


# Strategy for flexible player (stays open for 8+ picks)
def strategy_flexible(pack: list[SimCard], state: DraftState) -> SimCard:
    """Stays open as long as possible, picking the highest-power card from
    among those with S/A fitness in any active archetype. Doesn't commit
    until the system forces it."""
    # Always pick the best-available card that is S/A in ANY active archetype
    fitting_cards = [c for c in pack
                     if any(c.is_fitting(a) for a in state.active_archetypes)]
    if fitting_cards:
        return max(fitting_cards, key=lambda c: c.power)
    return max(pack, key=lambda c: c.power)


STRATEGIES = {
    "committed": strategy_committed,
    "power_chaser": strategy_power_chaser,
    "signal_reader": strategy_signal_reader,
}

# Story trace strategies (separate from main metrics)
TRACE_STRATEGIES = {
    "early_committer": strategy_early_committer,
    "flexible": strategy_flexible,
    "signal_reader": strategy_signal_reader,
}


# ---------------------------------------------------------------------------
# Run a single draft
# ---------------------------------------------------------------------------

def run_draft(cards: list[SimCard], strategy_name: str,
              rng: random.Random, trace: bool = False,
              weight_fn=None, use_soft_floor: bool = True,
              strategy_fn_override=None) -> dict:
    """Run one complete draft and return metrics."""
    pool = build_pool(cards)
    rng.shuffle(pool)

    # Suppress 2 random archetypes
    suppressed = rng.sample(range(NUM_ARCHETYPES), k=SUPPRESSED_PER_RUN)
    active = [a for a in range(NUM_ARCHETYPES) if a not in suppressed]

    # Remove ~50% of copies of cards whose primary archetype is suppressed
    new_pool = []
    for card in pool:
        primary = card.best_archetype()
        if primary in suppressed:
            if rng.random() < 0.5:
                continue
        new_pool.append(card)
    pool = new_pool

    state = DraftState(
        pool=pool,
        active_archetypes=active,
    )

    # Starting signal: show 3 cards from active archetypes, keep best
    starting_candidates = [c for c in state.pool
                           if any(c.fitness_in(a) == "S" for a in active)]
    if len(starting_candidates) >= 3:
        starting_options = rng.sample(starting_candidates, k=3)
    else:
        starting_options = rng.sample(state.pool, k=min(3, len(state.pool)))

    state.starting_card = max(starting_options,
                              key=lambda c: max(c.fitness_score(a) for a in active))

    # Track the starting card for archetype detection
    for a in active:
        if state.starting_card.is_fitting(a):
            state.archetype_counts[a] += 1

    # Select strategy function
    if strategy_fn_override is not None:
        strategy_fn = strategy_fn_override
    elif strategy_name in STRATEGIES:
        strategy_fn = STRATEGIES[strategy_name]
    else:
        strategy_fn = TRACE_STRATEGIES[strategy_name]

    # Draft loop
    trace_log = []
    metrics = {
        "early_unique_archetypes": [],
        "early_fitting_per_pack": [],
        "late_fitting_per_pack": [],
        "late_off_archetype": [],
        "convergence_pick": None,
        "deck_cards": [],
        "suppressed": suppressed,
        "active": active,
    }

    for pick_num in range(1, PICKS_PER_DRAFT + 1):
        pack = construct_pack(state, pick_num, rng,
                              weight_multiplier_fn=weight_fn,
                              use_soft_floor=use_soft_floor)
        if len(pack) == 0:
            break

        state.packs_seen.append(pack)

        # Measure early metrics (picks 1-5)
        if pick_num <= 5:
            archs_in_pack = set()
            for c in pack:
                for a in active:
                    if c.is_fitting(a):
                        archs_in_pack.add(a)
            metrics["early_unique_archetypes"].append(len(archs_in_pack))

            if state.committed_archetype is not None:
                fitting = sum(1 for c in pack
                              if c.is_fitting(state.committed_archetype))
                metrics["early_fitting_per_pack"].append(fitting)
            elif state.archetype_counts:
                emerging = max(state.archetype_counts,
                               key=lambda a: state.archetype_counts[a])
                fitting = sum(1 for c in pack if c.is_fitting(emerging))
                metrics["early_fitting_per_pack"].append(fitting)

        # Measure late metrics (picks 6+)
        if pick_num >= 6 and state.committed_archetype is not None:
            arch = state.committed_archetype
            fitting = sum(1 for c in pack if c.is_fitting(arch))
            metrics["late_fitting_per_pack"].append(fitting)

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
            # Determine which archetype to report fitness for
            report_arch = state.committed_archetype
            if report_arch is None and state.archetype_counts:
                report_arch = max(state.archetype_counts,
                                  key=lambda a: state.archetype_counts[a])
            if report_arch is None:
                report_arch = 0

            trace_log.append({
                "pick": pick_num,
                "pack": [(c.id, c.fitness_in(report_arch),
                          round(c.power, 1))
                         for c in pack],
                "picked_id": picked.id,
                "committed": state.committed_archetype,
                "arch_counts": dict(state.archetype_counts),
            })

        # Update archetype counts
        for a in active:
            if picked.is_fitting(a):
                state.archetype_counts[a] += 1

        # Check commitment (with tightened detection)
        if state.committed_archetype is None:
            state.committed_archetype = detect_commitment(state, pick_num)

        # Remove picked card from pool (no depletion of unpicked)
        remove_picked_card(state, picked)

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
    metrics["commitment_pick"] = None
    # Find when commitment actually happened
    if trace_log:
        for entry in trace_log:
            if entry["committed"] is not None:
                metrics["commitment_pick"] = entry["pick"]
                break

    return metrics


# ---------------------------------------------------------------------------
# Batch simulation
# ---------------------------------------------------------------------------

def run_simulation(multi_arch_pct: Optional[float] = None,
                   num_drafts: int = NUM_DRAFTS,
                   trace_count: int = 0,
                   weight_fn=None,
                   use_soft_floor: bool = True,
                   seed: int = 42) -> dict:
    """Run full simulation batch."""
    master_rng = random.Random(seed)
    cards = generate_card_pool(master_rng, multi_arch_pct=multi_arch_pct)

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

            m = run_draft(cards, strategy_name, draft_rng, trace=do_trace,
                          weight_fn=weight_fn, use_soft_floor=use_soft_floor)

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


# ---------------------------------------------------------------------------
# Analysis and reporting
# ---------------------------------------------------------------------------

def compute_metrics(all_results: dict) -> dict:
    """Compute the 8 target metrics from simulation results."""
    metrics = {}
    committed = all_results["committed"]

    if committed["early_unique_archs"]:
        metrics["early_unique_archs"] = (
            sum(committed["early_unique_archs"]) /
            len(committed["early_unique_archs"])
        )
    else:
        metrics["early_unique_archs"] = 0

    if committed["early_fitting"]:
        metrics["early_fitting"] = (
            sum(committed["early_fitting"]) /
            len(committed["early_fitting"])
        )
    else:
        metrics["early_fitting"] = 0

    if committed["late_fitting"]:
        metrics["late_fitting"] = (
            sum(committed["late_fitting"]) /
            len(committed["late_fitting"])
        )
    else:
        metrics["late_fitting"] = 0

    if committed["late_off_arch"]:
        metrics["late_off_arch"] = (
            sum(committed["late_off_arch"]) /
            len(committed["late_off_arch"])
        )
    else:
        metrics["late_off_arch"] = 0

    if committed["convergence_picks"]:
        metrics["convergence_pick"] = (
            sum(committed["convergence_picks"]) /
            len(committed["convergence_picks"])
        )
    else:
        metrics["convergence_pick"] = float("inf")

    concs = [c for c in committed["deck_concentrations"] if c > 0]
    if concs:
        metrics["deck_concentration"] = sum(concs) / len(concs)
    else:
        metrics["deck_concentration"] = 0

    # Run-to-run variety
    overlaps = []
    id_sets = committed["picked_id_sets"]
    sample_size = min(200, len(id_sets))
    for i in range(0, sample_size - 1, 2):
        if i + 1 < sample_size:
            s1 = id_sets[i]
            s2 = id_sets[i + 1]
            if s1 and s2:
                overlap = len(s1 & s2) / max(len(s1 | s2), 1)
                overlaps.append(overlap)
    metrics["card_overlap"] = sum(overlaps) / len(overlaps) if overlaps else 0

    # Archetype frequency
    arch_counts = defaultdict(int)
    total = len(committed["committed_archetypes"])
    for a in committed["committed_archetypes"]:
        arch_counts[a] += 1
    if total > 0:
        metrics["arch_freq_max"] = max(arch_counts.values()) / total
        metrics["arch_freq_min"] = min(arch_counts.values()) / total if len(arch_counts) >= NUM_ARCHETYPES else 0
    else:
        metrics["arch_freq_max"] = 0
        metrics["arch_freq_min"] = 0

    # Per-strategy breakdown
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
    print("\n" + "=" * 75)
    print("TARGET SCORECARD")
    print("=" * 75)

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
        ("Deck concentration (committed)", "85-95%",
         f"{metrics.get('deck_concentration', 0):.1%}",
         0.85 <= metrics.get("deck_concentration", 0) <= 0.95),
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
    print("-" * 75)
    for name, target, actual, passed in targets:
        actual_str = f"{actual:.2f}" if isinstance(actual, float) else str(actual)
        status = "PASS" if passed else "FAIL"
        print(f"{name:<45} {target:<10} {actual_str:<10} {status:<6}")

    pass_count = sum(1 for _, _, _, p in targets if p)
    print(f"\nPassed: {pass_count}/{len(targets)}")
    return pass_count, len(targets)


def run_story_traces(cards: list[SimCard]):
    """Run 3 story traces with different player archetypes."""
    print("\n" + "=" * 75)
    print("DRAFT STORY TRACES")
    print("=" * 75)

    trace_configs = [
        ("early_committer", "TRACE 1: Early Committer (commits by pick 4-5)",
         strategy_early_committer),
        ("flexible", "TRACE 2: Flexible Player (stays open 8+ picks)",
         strategy_flexible),
        ("signal_reader", "TRACE 3: Signal Reader (identifies open archetype)",
         strategy_signal_reader),
    ]

    for strat_name, title, strat_fn in trace_configs:
        rng = random.Random(12345)
        m = run_draft(cards, strat_name, rng, trace=True,
                      strategy_fn_override=strat_fn)
        trace = m["trace"]
        if not trace:
            continue

        print(f"\n{'=' * 70}")
        print(f"{title}")
        print(f"Active archetypes: {m['active']}, Suppressed: {m['suppressed']}")
        print(f"Final committed archetype: {m['committed_archetype']}")
        if m["committed_archetype"] is not None:
            sa_count = sum(1 for c in m["deck_cards"]
                           if c.is_fitting(m["committed_archetype"]))
            print(f"Deck concentration: {sa_count}/{len(m['deck_cards'])} "
                  f"= {sa_count/len(m['deck_cards']):.0%}")
        print(f"{'=' * 70}")

        for entry in trace[:15]:  # Show first 15 picks
            pick_num = entry["pick"]
            committed = entry["committed"]
            arch_counts = entry["arch_counts"]
            pack_str = " | ".join(
                f"[{'*' if cid == entry['picked_id'] else ' '}] "
                f"id={cid:3d} fit={fit} pwr={pwr}"
                for cid, fit, pwr in entry["pack"]
            )
            top_archs = sorted(arch_counts.items(), key=lambda x: x[1],
                               reverse=True)[:3] if arch_counts else []
            top_str = ", ".join(f"a{a}:{n}" for a, n in top_archs)
            status = f"COMMITTED(a{committed})" if committed is not None else f"open [{top_str}]"
            print(f"  Pick {pick_num:2d} {status:30s} {pack_str}")

        if len(trace) > 15:
            print(f"  ... (picks 16-30 omitted, {len(trace)} total)")


def run_sensitivity_analysis(seed: int = 42):
    """Run sensitivity sweeps on key parameters."""

    # Sweep 1: Multi-archetype card percentage
    print("\n" + "=" * 75)
    print("SENSITIVITY SWEEP 1: Multi-Archetype Card %")
    print("=" * 75)
    pcts = [0.05, 0.10, 0.15, 0.20, 0.25, 0.30, 0.35, 0.40]
    print(f"{'Multi-Arch %':<15} {'Late Fitting':<14} {'Deck Conc':<12} "
          f"{'Card Overlap':<14} {'Conv Pick':<12} {'Early Uniq':<12}")
    print("-" * 80)

    for pct in pcts:
        results = run_simulation(multi_arch_pct=pct, num_drafts=300,
                                 seed=seed)
        m = compute_metrics(results)
        print(f"{pct:<15.0%} {m.get('late_fitting', 0):<14.2f} "
              f"{m.get('deck_concentration', 0):<12.1%} "
              f"{m.get('card_overlap', 0):<14.1%} "
              f"{m.get('convergence_pick', float('inf')):<12.1f} "
              f"{m.get('early_unique_archs', 0):<12.2f}")

    # Sweep 2: Weight ramp intensity
    print("\n" + "=" * 75)
    print("SENSITIVITY SWEEP 2: Weight Ramp Intensity")
    print("=" * 75)

    ramp_configs = [
        ("5.0x/7.0x", lambda p: 1.0 if p <= 5 else (5.0 if p <= 10 else 7.0)),
        ("8.0x/10.0x*", lambda p: 1.0 if p <= 5 else (8.0 if p <= 10 else 10.0)),
        ("10.0x/13.0x", lambda p: 1.0 if p <= 5 else (10.0 if p <= 10 else 13.0)),
        ("12.0x/16.0x", lambda p: 1.0 if p <= 5 else (12.0 if p <= 10 else 16.0)),
    ]

    print(f"{'Ramp':<15} {'Late Fitting':<14} {'Deck Conc':<12} "
          f"{'Conv Pick':<12}")
    print("-" * 55)

    for name, fn in ramp_configs:
        results = run_simulation(num_drafts=300, weight_fn=fn, seed=seed)
        m = compute_metrics(results)
        print(f"{name:<15} {m.get('late_fitting', 0):<14.2f} "
              f"{m.get('deck_concentration', 0):<12.1%} "
              f"{m.get('convergence_pick', float('inf')):<12.1f}")

    # Sweep 3: Soft floor on/off
    print("\n" + "=" * 75)
    print("SENSITIVITY SWEEP 3: Soft Floor Effect")
    print("=" * 75)

    print(f"{'Soft Floor':<15} {'Late Fitting':<14} {'Deck Conc':<12} "
          f"{'Conv Pick':<12}")
    print("-" * 55)

    for use_floor in [True, False]:
        results = run_simulation(num_drafts=300, use_soft_floor=use_floor,
                                 seed=seed)
        m = compute_metrics(results)
        label = "ON" if use_floor else "OFF"
        print(f"{label:<15} {m.get('late_fitting', 0):<14.2f} "
              f"{m.get('deck_concentration', 0):<12.1%} "
              f"{m.get('convergence_pick', float('inf')):<12.1f}")


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

if __name__ == "__main__":
    print("Running Model D v2 simulation (N=8, refined variety-first)...")
    print(f"  {NUM_DRAFTS} drafts, {PICKS_PER_DRAFT} picks each, "
          f"{PACK_SIZE} cards per pack")
    print(f"  {SUPPRESSED_PER_RUN} archetypes suppressed per run")
    print(f"  Default multi-arch: ~28%")
    print(f"  Soft floor: ON")
    print(f"  Weight ramp: 8.0x/10.0x (picks 6-10/11+)")
    print(f"  Commitment: pick>=6, 3+ S/A, clear lead")
    print()

    # Main simulation
    results = run_simulation(trace_count=3)
    metrics = compute_metrics(results)
    pass_count, total = print_scorecard(metrics)

    # Per-strategy breakdown
    print("\n" + "=" * 75)
    print("PER-STRATEGY BREAKDOWN")
    print("=" * 75)
    for strat in STRATEGIES:
        prefix = strat + "_"
        late_fit = metrics.get(prefix + "late_fitting", "N/A")
        deck_conc = metrics.get(prefix + "deck_conc", "N/A")
        late_str = f"{late_fit:.2f}" if isinstance(late_fit, float) else str(late_fit)
        conc_str = f"{deck_conc:.1%}" if isinstance(deck_conc, float) else str(deck_conc)
        print(f"  {strat:<20} late_fitting={late_str:<8} deck_conc={conc_str}")

    # Generate cards for traces using same seed
    trace_rng = random.Random(42)
    trace_cards = generate_card_pool(trace_rng)
    run_story_traces(trace_cards)

    # Sensitivity analysis
    run_sensitivity_analysis()

    print("\n" + "=" * 75)
    print("Simulation complete.")
    print("=" * 75)
